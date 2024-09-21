#[cfg(test)]
mod tests {
    use CustomSocket_lib::data::Packet;
    use CustomSocket_lib::data::MyMessageId;
    use super::*;

    #[test]
    fn test1() {
        let packet = Packet::new(MyMessageId::new((127, 0, 0, 1), 8080), 10, 0);

        let packet_b = packet.serialize();

        assert_eq!(packet, Packet::deserialize(packet_b));
    }
}