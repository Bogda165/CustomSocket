use std::sync::{Arc};
use tokio::time::Duration;
use tokio::sync::Notify;
use tokio::io::{self, AsyncBufReadExt};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use CustomSocket_lib::*;

async fn handler_fn(data: Vec<u8>) {
    println!("{:?}", data);
}

async fn timeout_handler(timeouts: Vec<String>) {
    for i in timeouts {
        println!("{}", i);
    }
}

#[tokio::main]
async fn main() {
    let shared_ready = Arc::new(Notify::new());
    let shared_mem =  Arc::new(Mutex::new(None));

    let mut socket = CustomSocket::new("127.0.0.1".to_string(), 8090, SocketType::Recv, shared_ready.clone(), shared_mem.clone());

    socket.connect().await.unwrap();

    let handler = tokio::spawn (
        async move {
            tokio::join!(
                socket.recv(),
                socket.timeout_checker(Arc::new(timeout_handler)),
            );
        }
    );


    let handler_recv = tokio::spawn(
        async move {
            loop {
                shared_ready.notified().await;
                println!("Notified");
                let mut shared_mem = shared_mem.clone();

                tokio::spawn(async move {
                    let mut share_mem = shared_mem.lock().await;
                    if let Some(_data) = share_mem.take() {
                        let (ip, data) = (_data.0, _data.1.buffer);
                        println!("{}", ip);
                        handler_fn(data).await;
                    }else {
                        println!("Unexpected!!!!!!!!!");
                        panic!("WTF!!!!!!!");
                    }
                });
            }
        }
    );

    tokio::join!(handler, handler_recv);

}
