use std::marker::PhantomData;
use std::path::Iter;
use std::sync::Arc;
use std::thread::spawn;
use std::time::Duration;
use tokio::io::{self, AsyncBufReadExt};
use tokio::sync::{Mutex, Notify};
use tokio::task::JoinHandle;
use CustomSocket_lib::*;
use CustomSocket_lib::packet::Packet;

async fn send(socket: &mut CustomSocket) {
    println!("Sned");
    socket.send("127.0.0.1".to_string(), 8090, (1..23).collect(), 0).await.unwrap()
}

async fn send2(socket: &mut CustomSocket) {
    println!("Sned");
    socket.send("127.0.0.1".to_string(), 8090, (24..50).collect(), 0).await.unwrap()
}

async fn send3(socket: &mut CustomSocket) {
    println!("Sned");
    socket.send("127.0.0.1".to_string(), 8090, (51..100).collect(), 0).await.unwrap()
}

async fn send4(socket: &mut CustomSocket) {
    println!("Sned");
    socket.send("127.0.0.1".to_string(), 8090, (1..15).collect(), 0).await.unwrap()
}

async fn send5(socket: &mut CustomSocket) {
    println!("Sned");
    socket.send("127.0.0.1".to_string(), 8090, (4..55).collect(), 0).await.unwrap()
}

async fn send6(socket: &mut CustomSocket) {
    println!("Sned");
    socket.send("127.0.0.1".to_string(), 8090, (8..100).collect(), 0).await.unwrap()
}

#[tokio::main]
async fn main() {
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(1)).await;
        let shared_ready2 = Arc::new(Notify::new());
        let mut socket2 = CustomSocket::new("127.0.0.1".to_string(), 8085, SocketType::Send, shared_ready2.clone(), Arc::new(Mutex::new(Data::new(0, 0))));
        socket2.connect().await.unwrap();

        send2(&mut socket2).await;}
    );

    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(2)).await;
        let shared_ready2 = Arc::new(Notify::new());
        let mut socket2 = CustomSocket::new("127.0.0.1".to_string(), 8084, SocketType::Send, shared_ready2.clone(), Arc::new(Mutex::new(Data::new(0, 0))));
        socket2.connect().await.unwrap();

        send4(&mut socket2).await;});

    tokio::time::sleep(Duration::from_millis(3)).await;
    let shared_ready3 = Arc::new(Notify::new());
    let mut socket3 = CustomSocket::new("127.0.0.1".to_string(), 8086, SocketType::Send, shared_ready3.clone(), Arc::new(Mutex::new(Data::new(0, 0))));
    socket3.connect().await.unwrap();

    send5(&mut socket3).await;
}
