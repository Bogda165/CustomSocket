// impl trait with sec

use std::marker::PhantomData;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::RwLock;
use CustomIterators::*;

#[derive(Debug)]
pub struct Packet {
    pub message_id: u16,
    pub total_packets: u16,
    pub packet_id: u16,
    pub data: Vec<u8>,
}

impl Packet {
    pub fn new(message_id: u16, total_packets: u16, packet_id: u16) -> Self {
        Packet {
            message_id,
            total_packets,
            packet_id,
            data: Vec::<u8>::new(),
        }
    }

    pub fn set_data (&mut self, data: Vec<u8>) {
        self.data = data;
    }

    pub fn vec_from_slice(slice: Vec<u8>, packet_size: u16, message_id: u16) -> Vec<Packet> {
        let size = slice.len();
        let total_packets: u16 = (size / packet_size as usize + if size % packet_size as usize != 0 { 1usize } else {0usize}) as u16 ;
        println!("total_packets: {}", total_packets);

        // THE OLD WAY ))))
        /*
        for packet_id in 0..total_packets {
            let slice = &slice[(packet_id * packet_size) as usize..(((packet_id + 1) * packet_size) as usize).min(size)];
            let mut packet = Packet::new(message_id, total_packets, packet_id);
            packet.set_data(slice.to_vec());
            packets_vec.push(packet);
        }
        */
        let slices = (0..total_packets)
            .map(|packet_id| {&slice[(packet_id * packet_size) as usize..(((packet_id + 1) * packet_size) as usize).min(size)]})
            .my_zip(
                (0..total_packets)
                    .map(|packet_id| {Packet::new(message_id, total_packets, packet_id)}),
        |slice, mut packet| {packet.set_data(slice.to_vec()); packet});


        let slices: Vec<_> = slices.collect();

        slices
    }

    pub fn data_from_vec(mut packets: Vec<Packet>) -> Vec<u8> {
        packets.sort_by(|packet1, packet2| {
            packet1.packet_id.cmp(&packet2.packet_id)
        });

        let data: Vec<_> = packets.into_iter()
            .flat_map(|packet| packet.data)
            .collect();
        data
    }

    pub fn serialize(&self) -> Vec<u8>{
        let mut buffer = Vec::new();
        buffer.extend(&self.message_id.to_be_bytes());
        buffer.extend(&self.total_packets.to_be_bytes());
        buffer.extend(&self.packet_id.to_be_bytes());

        buffer.extend(&self.data);

        buffer
    }

    pub fn deserialize(buffer: Vec<u8>) -> Packet {
        Packet {
            message_id: u16::from_be_bytes(buffer[0..2].try_into().unwrap()),
            total_packets: u16::from_be_bytes(buffer[2..4].try_into().unwrap()),
            packet_id: u16::from_be_bytes(buffer[4..6].try_into().unwrap()),
            data: buffer[6..].to_vec(),
        }
    }
}
