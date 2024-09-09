use crate::packet::Packet;

struct DataWithIp {
    ip: String,
    data: Data,
}
#[derive(Debug)]
pub struct Data {
    pub buffer: Vec<u8>,
    packet_a: i32,
    packet_size: u16,
}

impl Data {
    pub fn new(packet_a: i32, packet_size: u16) -> Data{
        //TODO change magic val packet_size
        let buffer = vec![0; (packet_a * packet_size as i32) as usize];

        Data {
            packet_a,
            buffer,
            packet_size,
        }
    }
    //return true if amount of packet is 0
    pub fn add(&mut self, packet: Packet) -> bool{
        let start = (packet.packet_id * self.packet_size) as usize;
        let end = ((packet.packet_id + 1) * self.packet_size) as usize;
        self.buffer.splice(start..end, packet.data);

        self.packet_a -= 1;

        self.packet_a == 0
    }
}