mod receive_handler;

use std::future::Future;
use std::sync::Arc;
use std::sync::atomic::{AtomicU16};
use std::sync::atomic::Ordering::SeqCst;
use CustomSocket_lib::*;
use tokio::sync::{Mutex, Notify};
use CustomSocket_lib::data::Data;
use CustomSocket_lib::timeout_handler::TimeoutHandler;

async fn handler_fn(data: Vec<u8>) {
    println!("{:?}", std::str::from_utf8(&*data));
}

static MESSAGE_ID: AtomicU16 = AtomicU16::new(0);

trait RecvHandler {
    fn on_recv(&self, data: Vec<u8>) -> impl Future<Output = ()> + Send + Sync;
}

pub struct DefaultRecvHandler {
    all_data: Arc<Mutex<Vec<Vec<u8>>>>,
}

impl DefaultRecvHandler {
    pub fn new() -> Self {
        DefaultRecvHandler {
            all_data: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn show_all_messages(&self) {
        let all_data = self.all_data.clone();
        let data_g = self.all_data.lock().await;
        let text = data_g.iter().map(|mes_u8| {std::str::from_utf8(&*mes_u8).unwrap()}).collect::<Vec<&str>>().join(" ");
        println!("{}", text);
    }
}

impl RecvHandler for DefaultRecvHandler {
    fn on_recv(&self, data: Vec<u8>) -> impl Future<Output=()> + Send + Sync {
        let all_data = self.all_data.clone();
        async move {
            let mut all_data_g = all_data.lock().await;
            all_data_g.push(data.clone());
            println!("{:?}", std::str::from_utf8(&*data));
        }
    }
}

pub struct CustomServer<TH, RH>
where
    TH: TimeoutHandler + Send + Sync + 'static,
    RH: RecvHandler + Send + Sync + 'static,
{
    socket_recv: Arc<CustomSocket>,
    socket_send: Arc<CustomSocket>,
    shared_ready: Arc<Notify>,
    shared_mem: Arc<Mutex<Option<(String, Data)>>>,
    pub timeout_handler: Arc<Mutex<TH>>,
    pub receive_handler: Arc<RH>,
}


impl<TH, RH> CustomServer<TH, RH>
where
    TH: TimeoutHandler + Send + Sync + 'static,
    RH: RecvHandler + Send + Sync + 'static,
{
    pub async fn new(recv_addr: String, recv_port: u16, send_addr: String, send_port: u16, timeout_handler: TH, receive_handler: RH) -> Self {
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
            receive_handler: Arc::new(receive_handler),
        }
    }

    pub fn get_ss(&self) -> Arc<CustomSocket> {
        self.socket_send.clone()
    }

    pub fn set_receive_handler(&mut self, rh: RH) {
        self.receive_handler = Arc::new(rh);
    }

    pub fn set_timeout_handler(&mut self, th: TH) {
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
            let rh =  self.receive_handler.clone();
            async move {
                loop {
                    let rh =  rh.clone();
                    shared_ready.notified().await;
                    println!("Notified");
                    let mut shared_mem = shared_mem.lock().await;
                    if let Some((ip, data)) = shared_mem.take() {
                        println!("{}", ip);
                        //add passing a receiving handler
                        //let rh = self.receive_handler.clone();
                        tokio::spawn(async move {rh.on_recv(data.buffer).await;});
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