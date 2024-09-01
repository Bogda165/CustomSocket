use std::sync::{Arc};
use tokio::time::Duration;
use tokio::sync::Notify;
use tokio::io::{self, AsyncBufReadExt};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use CustomSocket_lib::*;
use CustomSocket_lib::packet::Packet;

async fn handler_fn(data: Vec<u8>) {
    println!("{:?}", data);
}

#[tokio::main]
async fn main() {
    let shared_ready = Arc::new(Notify::new());
    let shared_mem =  Arc::new(Mutex::new(Data::new(0, 0)));

    let mut socket = CustomSocket::new("127.0.0.1".to_string(), 8090, SocketType::Recv, shared_ready.clone(), shared_mem.clone());

    socket.connect().await.unwrap();

    let handler = tokio::spawn (
        async move {
            socket.recv().await
        }
    );

    let handler_recv = tokio::spawn(
        async move {
            loop {
                shared_ready.notified().await;
                println!("Notified");
                let mut shared_mem = shared_mem.clone();

                tokio::spawn(async move {let share_mem = shared_mem.lock().await;
                    let data = (*share_mem).buffer.clone();
                    handler_fn(data).await;});
            }
        }
    );

    tokio::join!(handler, handler_recv);

}
