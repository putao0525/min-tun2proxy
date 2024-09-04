mod modules;
mod platform;

use std::io::{self, Read, Write};
use std::net::SocketAddr;
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::ipv6::Ipv6Packet;
use pnet::packet::Packet;
use pnet::packet::vlan::VlanPacket;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tun::platform::Device as TunDevice;
use tun::{Configuration, Device};
use tokio::net::TcpStream;
use tokio_socks::tcp::Socks5Stream;
use crate::modules::self_packet::SelfPacket;
use crate::modules::user_config::{ProxyType, UserConfig};
use crate::platform::macos::mac_table::MacRouteTable;
use crate::platform::RouteTable;

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut config = Configuration::default();
    config.address((10, 0, 0, 1))?; // 设置 TUN 设备的 IP 地址
    config.netmask((255, 255, 255, 0))?; // 设置 TUN 设备的子网掩码
    config.mtu(1500); // 设置 TUN 设备的 MTU
    let user_conf = UserConfig::default();
    //创建设备
    let mut dev = create_tun_dev(&config).expect("创建tun设备失败");
    //初始化路由表
    let route_table: Box<dyn RouteTable> = Box::new(MacRouteTable::default());
    route_table.init_route_table();
    // 读取和处理数据包
    let mut buf = [0u8; 1504];
    loop {
        let n = dev.read(&mut buf)?;
        println!("Read {} bytes", n);
        packet_route(&*buf, &user_conf);
    }
}

fn packet_route(buf: &[u8], user_conf: &UserConfig) {
    // 解析以太网帧
    if let Some(eth_packet) = EthernetPacket::new(buf) {
        match eth_packet.get_ethertype() {
            EtherTypes::Ipv4 => {
                if let Some(packet) = (eth_packet.payload()) {
                    let self_packet = SelfPacket::new_ipv4(packet);
                    let _ = forward_packet(self_packet, user_conf);
                }
            }
            EtherTypes::Ipv6 => {
                if let Some(packet) = Ipv6Packet::new(eth_packet.payload()) {
                    let self_packet = SelfPacket::new_ipv6(&packet);
                    let _ = forward_packet(self_packet, user_conf);
                }
            }
            EtherTypes::Vlan => {
                if let Some(vlan_packet) = VlanPacket::new(eth_packet.payload()) {
                    match vlan_packet.get_ethertype() {
                        EtherTypes::Ipv4 => {
                            if let Some(packet) = Ipv4Packet::new(vlan_packet.payload()) {
                                let self_packet = SelfPacket::new_ipv4(&packet);
                                let _ = forward_packet(self_packet, user_conf);
                            }
                        }
                        EtherTypes::Ipv6 => {
                            if let Some(packet) = Ipv6Packet::new(vlan_packet.payload()) {
                                let self_packet = SelfPacket::new_ipv6(&packet);
                                let _ = forward_packet(self_packet, user_conf);
                            }
                        }
                    }
                }
            }
        }
    }
}

///创建tun设备
fn create_tun_dev(config: &Configuration) -> anyhow::Result<TunDevice> {
    let mut dev = TunDevice::new(&config)?;
    let name = dev.name().to_string_lossy().into_owned();
    println!("TUN device created: {}", name);
    dev
}

// 根据目标地址和端口选择使用 HTTP 或 SOCKS5 代理进行转发
async fn forward_packet(self_packet: SelfPacket<'_>, user_config: &UserConfig) -> anyhow::Result<()> {
    if user_config.proxy_type == ProxyType::Http {
        forward_http(self_packet, user_config).await
    } else {
        forward_socks5(self_packet, user_config).await
    }
}

// 使用 HTTP 代理进行转发
async fn forward_http(self_packet: SelfPacket, user_config: &UserConfig) -> anyhow::Result<()> {
    !todo!()
}

// 使用 SOCKS5 代理进行转发
async fn forward_socks5(self_packet: SelfPacket, user_config: &UserConfig) -> anyhow::Result<()> {
    let proxy_addr = user_config.get_proxy_addr().parse::<SocketAddr>().unwrap();
    let target_addr = self_packet.get_target_addr().unwrap();
    // 连接到 SOCKS5 代理服务器
    if (user_config.is_use_proxy_pool) {
        // return Err("暂时不支持使用代理池子")
    }
    let stream = TcpStream::connect(proxy_addr).await?;
    let mut socks5_stream = Socks5Stream::connect_with_socket(stream, target_addr).await?;
    // let manager = Socks5ConnectionManager { proxy_addr: proxy_addr, target_addr: target_addr };
    // let pool = Pool::builder().build(manager).await?;
    let (mut client_reader, mut client_writer) = socks5_stream.split();
    let mut response_buffer = vec![0; 1500];
    // 从客户端读取数据并写入到 SOCKS5 代理
    let client_to_socks5 = async {
        loop {
            match client_reader.read(&mut response_buffer).await {
                Ok(0) => break, // 连接关闭
                Ok(n) => {
                    if let Err(e) = client_writer.write_all(&response_buffer[..n]).await {
                        eprintln!("Failed to write to SOCKS5: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Failed to read from client: {}", e);
                    break;
                }
            }
        }
    };
    // 从 SOCKS5 代理读取数据并写入到客户端
    let socks5_to_client = async {
        loop {
            match client_writer.read(&mut response_buffer).await {
                Ok(0) => break, // 连接关闭
                Ok(n) => {
                    if let Err(e) = client_reader.write_all(&response_buffer[..n]).await {
                        eprintln!("Failed to write to client: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Failed to read from SOCKS5: {}", e);
                    break;
                }
            }
        }
    };

    // 并行运行两个任务
    tokio::try_join!(client_to_socks5, socks5_to_client)?;
    Ok(())
}