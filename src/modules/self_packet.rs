use std::net::{IpAddr, SocketAddr};
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::ipv6::Ipv6Packet;
use pnet::packet::Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;

pub enum SelfPacket<'a> {
    Ipv4(&'a Ipv4Packet<'a>),
    Ipv6(&'a Ipv6Packet<'a>),
}

impl<'a> SelfPacket<'a> {
    pub fn new_ipv4(packet: &'a Ipv4Packet<'a>) -> Self {
        SelfPacket::Ipv4(packet)
    }

    pub fn new_ipv6(packet: &'a Ipv6Packet<'a>) -> Self {
        SelfPacket::Ipv6(packet)
    }

    pub fn custom_packet(&self) -> &[u8] {
        match self {
            SelfPacket::Ipv4(p) => p.packet(),
            SelfPacket::Ipv6(p) => p.packet(),
        }
    }

    pub fn get_target_addr(&self) -> Option<SocketAddr> {
        match self {
            SelfPacket::Ipv4(packet) => {
                let dest_ip = packet.get_destination();
                let dest_port = SelfPacket::get_transport_port(packet.payload())?;
                Some(SocketAddr::new(IpAddr::V4(dest_ip), dest_port))
            }
            SelfPacket::Ipv6(packet) => {
                let dest_ip = packet.get_destination();
                let dest_port = SelfPacket::get_transport_port(packet.payload())?;
                Some(SocketAddr::new(IpAddr::V6(dest_ip), dest_port))
            }
        }
    }
    fn get_transport_port(payload: &[u8]) -> Option<u16> {
        if let Some(tcp_packet) = TcpPacket::new(payload) {
            return Some(tcp_packet.get_destination());
        } else if let Some(udp_packet) = UdpPacket::new(payload) {
            return Some(udp_packet.get_destination());
        }
        None
    }
}
