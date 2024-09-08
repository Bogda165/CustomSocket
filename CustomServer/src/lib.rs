use std::future::Future;
use std::io::Error;
use std::sync::Arc;
use std::sync::atomic::{AtomicU16, AtomicUsize};
use std::sync::atomic::Ordering::SeqCst;
use CustomSocket_lib::*;
use tokio::io;
use tokio::io::AsyncBufReadExt;
use tokio::sync::{Mutex, Notify};

async fn handler_fn_default(data: Vec<u8>) {
    println!("{:?}", std::str::from_utf8(&*data));
}

async fn timeout_handler_default(timeouts: Vec<String>) {
    for i in timeouts {
        println!("Timeout from: {}", i);
    }
}

static MESSAGE_ID: AtomicU16 = AtomicU16::new(0);

async fn handler_fn(data: Vec<u8>) {
    println!("{:?}", data);
}

pub struct CustomServer<H>
where
    H: TimeoutHandler + Send + Sync + 'static,
{
    socket_recv: Arc<CustomSocket>,
    socket_send: Arc<CustomSocket>,
    shared_ready: Arc<Notify>,
    shared_mem: Arc<Mutex<Option<(String, Data)>>>,
    pub timeout_handler: Arc<Mutex<H>>,
}


impl<H> CustomServer<H>
where
    H: TimeoutHandler + Send + Sync + 'static,

{
    pub async fn new(recv_addr: String, recv_port: u16, send_addr: String, send_port: u16, timeout_handler: H) -> Self {
        let shared_ready = Arc::new(Notify::new());
        let shared_mem = Arc::new(Mutex::new(None));
        let mut socket_recv = CustomSocket::new(recv_addr, recv_port, SocketType::Recv, shared_ready.clone(), shared_mem.clone());
        let mut socket_send = CustomSocket::new(send_addr, send_port, SocketType::Send, shared_ready.clone(), shared_mem.clone());

        socket_recv.connect().await.unwrap();
        socket_send.connect().await.unwrap();

        CustomServer {
            socket_recv: Arc::new(socket_recv),
            socket_send: Arc::new(socket_send),
            shared_ready,
            shared_mem,
            timeout_handler: Arc::new(Mutex::new(timeout_handler)),
        }
    }

    pub fn get_ss(&self) -> Arc<CustomSocket> {
        self.socket_send.clone()
    }

    pub fn set_timeout_handler(&mut self, th: H) {
        self.timeout_handler = Arc::new(Mutex::new(th));
    }

    pub async fn start(&self) {
        let socket_recv = self.socket_recv.clone();
        let timeout_handler = self.timeout_handler.clone();
        let recv_task = tokio::spawn({
            async move {
                tokio::join!(
                    socket_recv.recv(),
                    socket_recv.timeout_checker(timeout_handler),
            );
            }
        });


        let notify_task = tokio::spawn({
            let shared_ready = self.shared_ready.clone();
            let shared_mem = self.shared_mem.clone();
            async move {
                loop {
                    shared_ready.notified().await;
                    println!("Notified");
                    let mut shared_mem = shared_mem.lock().await;
                    if let Some((ip, data)) = shared_mem.take() {
                        println!("{}", ip);
                        tokio::spawn(handler_fn(data.buffer));
                    } else {
                        println!("Unexpected!!!!!!!!!");
                        panic!("WTF!!!!!!!");
                    }
                }
            }
        });
        tokio::join!(recv_task, notify_task);
    }

    pub async fn send(&self, addr: String, port: u16, buffer: Vec<u8>){
        MESSAGE_ID.fetch_add(1, SeqCst);
        self.socket_send.send(addr, port, buffer, MESSAGE_ID.load(SeqCst).clone()).await.unwrap()
    }
}
/*
#[tokio::main]
async fn main() {
    let mut server = Arc::new(CustomServer::new("127.0.0.1".to_string(), 8090, "127.0.0.1".to_string(), 8091).await);

    let _server = server.clone();
    let recv = _server.start();


    let send = tokio::spawn(async move {
        let stdin = io::stdin();
        let mut reader = io::BufReader::new(stdin).lines();

        while let Ok(Some(line)) = reader.next_line().await {
            println!("{}", line);
            server.send("127.0.0.1".to_string(), 8090, line.as_bytes().to_vec()).await;
        }
    });

    tokio::join!(recv, send);
}

*/