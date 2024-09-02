pub mod packet;

use std::collections::HashMap;

use tokio::net::UdpSocket;
use std::sync::{Arc, Condvar};
use tokio::sync::{RwLock, RwLockWriteGuard, Mutex, MutexGuard, Notify};
use crate::packet::Packet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread::sleep;
use std::time::Duration;
use lazy_static::lazy_static;
use rand::random;
//Custom Socket allow only one therad reciver!!!
// the struct of each packet -> message id, number of packet per message, packet id.

//each time new data is ready conditional variable will be notified, and data can be requested from customSocket!!!!

// Be carefull the recommended time between each send is more the 1 mills

static MAGIC_CONST: i32 = 10;

lazy_static! {
    static ref COUNTER: AtomicUsize = AtomicUsize::new(0);
}

pub fn increment_counter() {
    COUNTER.fetch_add(1, Ordering::SeqCst);
}

pub fn get_counter() -> usize {
    COUNTER.load(Ordering::SeqCst)
}
pub enum SocketType {
    Recv,
    Send,
}
// TODO here implement sending of a huge data and recieving
pub struct CustomSocket {
    socket_addr: String,
    port: u16,
    socket: Arc<RwLock<Option<UdpSocket>>>,
    s_type: SocketType,
    ready: Arc<Notify>,
    messages: Arc<Mutex<HashMap<u16, Data>>>,
    // add a vector, of data!
    pub share_mem: Arc<Mutex<Data>>,
}

#[derive(Debug)]
pub struct Data {
    pub buffer: Vec<u8>,
    packet_a: i32,
    packet_size: u16,
}

impl Data {
    pub fn new(packet_a: i32, packet_size: u16) -> Data{
        //TODO change magic val packet_size
        let buffer = vec![0; (packet_a * packet_size as i32) as usize];

        Data {
            packet_a,
            buffer,
            packet_size,
        }
    }
    //return true if amount of packet is 0
    pub fn add(&mut self, packet: Packet) -> bool{
        let start = (packet.packet_id * self.packet_size) as usize;
        let end = ((packet.packet_id + 1) * self.packet_size) as usize;
        self.buffer.splice(start..end, packet.data);

        self.packet_a -= 1;

        self.packet_a == 0
    }
}

impl CustomSocket {
    pub fn new(socket_addr: String, port: u16, s_type: SocketType, ready: Arc<Notify>, share_mem: Arc<Mutex<Data>>) -> Self {
        let messages: Arc<Mutex<HashMap<u16, Data>>> = Arc::new(Mutex::new(HashMap::new()));
        
        CustomSocket {
            socket_addr,
            port,
            s_type,
            socket: Arc::new(RwLock::new(None)),
            ready, 
            messages,
            share_mem,
        }
    }

    pub async fn connect(&mut self) -> Result<(), ()>{
        let new_socket = UdpSocket::bind(format!("{}:{}", self.socket_addr, self.port)).await.unwrap();

        let mut tmp = self.socket.write().await;
        *tmp = Some(new_socket);

        Ok(())
    }

    pub async fn recv(&mut self){

        loop {
            let mut buffer = vec![0u8; 1024];
            match self.s_type {
                SocketType::Recv => {
                    {
                        let socket = self.socket.read().await;
                        match *socket {
                            None => println!("There is no socket((( do not forget to .connect() it)"),
                            Some(ref socket) => {
                                println!("begin recv from raw socket");
                                match socket.recv(&mut buffer).await {
                                    // 1) parse a packet -done
                                    // 2) if there are not DATA obj in haspmap for message_id, create a new one, else use data.add function()
                                    // 3) if data.add return true, check if ready is false 3`), lock ready mutex, write data from data obj to arc mutex vector(so another thread can read it)
                                    //      notify ready(In another thread(outiside od CustomMutex check if ready is true read data and set ready to false))
                                    // 3`) throw an error can not happend!!!!
                                    Ok(buffer_size) => {
                                        println!("Received from raw socket!");
                                        buffer = buffer[..buffer_size].to_vec();
                                    }
                                    Err(_) => {
                                        println!("Error while receiving");
                                    }
                                }
                            }
                        };
                    }
                    println!("Handler invoked");

                    let handler_fut = handler(
                        buffer.to_vec(),
                        Arc::clone(&self.ready),
                        Arc::clone(&self.messages),
                        Arc::clone(&self.share_mem),
                    );

                    tokio::spawn(handler_fut);

                }
                SocketType::Send => {
                    println!("Not the right type!!!")
                }
            }
        }
    }

    async fn send_packet(packet: &mut Packet, sender: &UdpSocket, receiver: &str) -> Result<(), ()> {
        println!("{:?}", packet);
        let data = packet.serialize();

        sender.send_to(data.as_slice(), receiver).await.unwrap();
        Ok(())
    }

    pub async fn send(&self, addr: String, port: u16, buffer: Vec<u8>, message_id: u16) -> Result<(), ()>{
        let _socket = self.socket.write().await;

        match * _socket{
            None => {
                Err(())
            }
            Some(ref _socket) => {
                let packets = Packet::vec_from_slice(buffer, MAGIC_CONST as u16, message_id);
                println!("{:?}", packets);
                for mut packet in packets {
                   Self::send_packet(&mut packet, _socket, format!("{}:{}", addr, port).as_str()).await?;
                    println!("Send one packet -> {:?}", packet.data);
                }
                Ok(())
            }
        }
    //TODO change later to use a custom sender crate!!!! IDEA is to create a custom handler, that will deal with hube packets of data!!!
    }
}

async fn handler(
    buffer: Vec<u8>,
    ready: Arc<Notify>,
    messages: Arc<Mutex<HashMap<u16, Data>>>,
    share_mem: Arc<Mutex<Data>>,
) {
    let packet = Packet::deserialize(buffer);
    println!("{:?}", packet);
    let packet_a = packet.total_packets;
    println!("Received from raw socket: {:?}", packet.data);
    let message_id = packet.message_id;

    let mut messages = messages.lock().await;

    if !messages.contains_key(&packet.message_id) {
        messages.insert(message_id, Data::new(packet_a as i32, 10));
        println!("Created position in haspMap");
    }

    match messages.get_mut(&message_id).unwrap().add(packet) {
        true => {
            println!("Foud zero in hasp map");
            match messages.remove(&message_id) {
                None => {
                    unreachable!("Error wtf????");
                }
                Some(data) => {
                    {
                        let mut share_mem = share_mem.lock().await;
                        *share_mem = data;
                    }
                }
            }
            ready.notify_waiters();
            println!("Notify send");
        }
        false => {
            println!("remain {} packets to finish message", packet_a);
        }
    }

    println!("Handler finished");
}
