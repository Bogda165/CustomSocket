use std::marker::PhantomData;
use std::os::unix::raw::off_t;
use std::path::Iter;
use std::sync::Arc;
use std::thread::spawn;
use std::time::Duration;
use tokio::io::{self, AsyncBufReadExt};
use tokio::sync::{Mutex, Notify};
use tokio::task::JoinHandle;
use CustomSocket_lib::*;
use CustomSocket_lib::packet::Packet;
use futures::future::join_all;

#[tokio::main]
async fn main() {
    let mut futures: Vec<_> = vec![];
    for i in 0..100 {
        let future = async move {
            let shared_ready = Arc::new(Notify::new());
            let mut socket = CustomSocket::new("127.0.0.1".to_string(), 8091 + i, SocketType::Send, shared_ready.clone(), Arc::new(Mutex::new(None)));
            socket.connect().await.unwrap();

            println!("Sned");
            socket.send("127.0.0.1".to_string(), 8090, (1..50).collect(), 0).await.unwrap()
        };

        futures.push(future);
    }

    join_all(futures).await;

    let shared_ready = Arc::new(Notify::new());
    let mut socket = CustomSocket::new("127.0.0.1".to_string(), 8091, SocketType::Send, shared_ready.clone(), Arc::new(Mutex::new(None)));
    socket.connect().await.unwrap();

    println!("Sned");
    socket.send("127.0.0.1".to_string(), 8090, (1..50).collect(), 13).await.unwrap();
    println!("WOOOW");
}