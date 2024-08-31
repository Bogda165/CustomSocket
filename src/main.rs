use std::marker::PhantomData;
use std::path::Iter;
use std::sync::Arc;
use tokio::io::{self, AsyncBufReadExt};
use tokio::task::JoinHandle;
use CustomSocket_lib::*;
use CustomSocket_lib::packet::Packet;

async fn send(socket: &mut CustomSocket) {
    println!("Sned");
    socket.send("127.0.0.1".to_string(), 8081, vec![80, 81, 82]).await.unwrap()
}

#[tokio::main]
async fn main() {
    /*
    let mut socket = CustomSocket::new("127.0.0.1".to_string(), 8082, SocketType::Recv);

    socket.connect().await.unwrap();

    let handler = tokio::spawn (async move {
        let mut buffer =  [0u8; 1024];
        loop {
            match socket.recv(&mut buffer).await {
                Ok(len) => {
                    println ! ("{:?}", & buffer[..len]);
                }
                Err(_) => {
                    println ! ("FUUUCK");
                }
            }
        }
    });

    println!("Hello world");

    let mut socket2 = CustomSocket::new("127.0.0.1".to_string(), 8084, SocketType::Send);
    socket2.connect().await.unwrap();


    let console_handler= tokio::spawn(async move {
        let stdin = io::stdin();
        let mut reader = io::BufReader::new(stdin).lines();

        while let Some(line) = reader.next_line().await.unwrap() {
            send(&mut socket2).await;
        }
    });

    tokio::join!(handler, console_handler);

     */
    let vec: Vec<u8> = (0..=255).collect();

    let vector = Packet::vec_from_slice(vec, 10, 4);

    for packet in vector.iter().clone() {
        println!("{:?}", packet);
    }

    println!("===========");

    let vector2 = Packet::data_from_vec(vector);
    println!("{:?}", vector2);

    println!("===========");

    let mut packet = Packet::new(1, 5, 6);
    packet.set_data(vec![1, 2, 3, 4, 5]);

    let mut buffer = packet.serialize();
    let _packet = Packet::deserialize(buffer);
    println!("{:?}", _packet);
}
