pub mod packet;

use tokio::net::UdpSocket;
use std::sync::Arc;
use tokio::sync::{RwLock, RwLockWriteGuard};
use crate::packet::Packet;

// the struct of each packet -> message id, number of packet per message, packet id.
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
}

impl CustomSocket {
    pub fn new(socket_addr: String, port: u16, s_type: SocketType) -> Self {
        CustomSocket {
            socket_addr,
            port,
            s_type,
            socket: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn connect(&mut self) -> Result<(), ()>{
        let new_socket = UdpSocket::bind(format!("{}:{}", self.socket_addr, self.port)).await.unwrap();

        let mut tmp = self.socket.write().await;
        *tmp = Some(new_socket);

        Ok(())
    }

    pub async fn recv(&mut self, buffer: &mut Vec<u8>) -> Result<usize, String> {
        match self.s_type {
            SocketType::Recv => {
                let socket = self.socket.read().await;
                match *socket {
                    None => Err("Socket hasn't been created".to_string()),
                    Some(ref socket) => {
                        match socket.recv(buffer).await {
                            Ok(bytes_received) => Ok(bytes_received),
                            Err(e) => Err(e.to_string()),
                        }
                    }
                }
            }
            SocketType::Send => {
                Err("Uncorrect socket type".to_string())
            }
        }
    }

    pub async fn send(&self, addr: String, port: u16, buffer: Vec<u8>) -> Result<(), ()>{
        let _socket = self.socket.write().await;

        match * _socket{
            None => {
                Err(())
            }
            Some(ref _socket) => {
                let packets = Packet::vec_from_slice(buffer, 2, 0);
                for packet in packets {
                    packet.send(_socket, format!("{}:{}", addr, port).as_str()).await?;
                }
                Ok(())
            }
        }
        //TODO change later to use a custom sender crate!!!! IDEA is to create a custom handler, that will deal with hube packets of data!!!
    }


}
