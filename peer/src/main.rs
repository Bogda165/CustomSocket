use std::sync::Arc;
use tokio::io::{self, AsyncBufReadExt};
use tokio::task::JoinHandle;
use CustomSocket_lib::*;
use CustomSocket_lib::packet::Packet;

async fn send(socket: &mut CustomSocket) {
    println!("Sned");
    socket.send("127.0.0.1".to_string(), 8082, vec![80, 81, 82]).await.unwrap()
}

#[tokio::main]
async fn main() {

    let mut socket = CustomSocket::new("127.0.0.1".to_string(), 8082, SocketType::Recv);

    socket.connect().await.unwrap();

    let handler = tokio::spawn (async move {
        let mut buffer: Vec<u8> =  vec![0; 1024];
        loop {
            match socket.recv(&mut buffer).await {
                Ok(len) => {
                    println ! ("{:?}", Packet::deserialize((&buffer[..len]).to_vec()));
                }
                Err(_) => {
                    println ! ("FUUUCK");
                }
            }
        }
    });

    println!("Hello world");

    let mut socket2 = CustomSocket::new("127.0.0.1".to_string(), 8083, SocketType::Send);
    socket2.connect().await.unwrap();


    let console_handler= tokio::spawn(async move {
        let stdin = io::stdin();
        let mut reader = io::BufReader::new(stdin).lines();

        while let Some(line) = reader.next_line().await.unwrap() {
            send(&mut socket2).await;
        }
    });

    tokio::join!(handler, console_handler);


}
