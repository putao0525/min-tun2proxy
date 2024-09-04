mod modules;
mod platform;
mod utils;

use std::env;
use std::io::{Read, Write};
use std::net::SocketAddr;
use std::sync::Arc;
use log::{error, info};
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::ipv6::Ipv6Packet;
use pnet::packet::Packet;
use pnet::packet::vlan::VlanPacket;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tun::platform::Device as TunDevice;
use tun::{Configuration, Device};
use tokio::net::TcpStream;
use tokio::{signal, task};
use tokio_socks::tcp::Socks5Stream;

use crate::modules::self_packet::SelfPacket;
use crate::modules::user_config::{ProxyType, UserConfig};
use crate::platform::macos::mac_table::MacRouteTable;
use crate::platform::RouteTable;
use crate::utils::log::init_log_once;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    ///日志初始化
    unsafe { env::set_var("RUST_LOG", "info"); }
    init_log_once();
    ///解析命令行参数
    let user_conf = UserConfig::parse_params();
    let route_table = Arc::new(MacRouteTable::default());
    ///初始化路由表
    route_table.init_route_table();
    if user_conf.is_free_route_table {
        //信息处理（这里主要是释放路由表资源)
        // let _ = free_resource(route_table);
    }
    ///创建设备
    let mut dev = create_tun_dev(&user_conf.tun_conf).expect("创建tun设备失败");
    ///数据包处理
    let _ = exe_packet(&mut dev, &user_conf);
    Ok(())
}

fn exe_packet(dev: &mut TunDevice, user_config: &UserConfig) -> anyhow::Result<()> {
    let mut buf = [0u8; 1504];
    loop {
        let n = dev.read(&mut buf)?;
        println!("Read {} bytes", n);
        packet_route(&buf[..n], user_config);
    }
}


fn packet_route(buf: &[u8], user_conf: &UserConfig) {
    // 解析以太网帧
    if let Some(eth_packet) = EthernetPacket::new(buf) {
        match eth_packet.get_ethertype() {
            EtherTypes::Ipv4 => {
                if let Some(packet) = Ipv4Packet::new(eth_packet.payload()) {
                    let self_packet = SelfPacket::new_ipv4(&packet);
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
    info!("TUN device created: {}", name);
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
async fn forward_http(self_packet: SelfPacket<'_>, user_config: &UserConfig) -> anyhow::Result<()> {
    !todo!()
}

// 使用 SOCKS5 代理进行转发
//todo 有问题
async fn forward_socks5(self_packet: SelfPacket<'_>, user_config: &UserConfig) -> anyhow::Result<()> {
    let proxy_addr = user_config.get_proxy_addr().parse::<SocketAddr>().unwrap();
    let target_addr = self_packet.get_target_addr().unwrap();

    // 连接到 SOCKS5 代理服务器
    if user_config.is_use_proxy_pool {
        error!("不支持代理ip池子");
        return Ok(());
    }

    let stream = TcpStream::connect(proxy_addr).await?;
    let socks5_stream = Socks5Stream::connect_with_socket(stream, target_addr).await?;
    let tcp_stream = socks5_stream.into_inner();
    let (mut client_reader, mut client_writer) = tcp_stream.into_split();

    // 创建一个新的 TcpStream 连接到目标服务器
    let target_stream = TcpStream::connect(target_addr).await?;
    let (mut target_reader, mut target_writer) = target_stream.into_split();

    // 从客户端读取数据并写入到目标服务器
    let client_to_target = transfer_data(&mut client_reader, &mut target_writer);

    // 从目标服务器读取数据并写入到客户端
    let target_to_client = transfer_data(&mut target_reader, &mut client_writer);

    tokio::try_join!(client_to_target, target_to_client)?;
    Ok(())
}

async fn transfer_data<R, W>(mut reader: R, mut writer: W) -> anyhow::Result<()>
    where
        R: AsyncReadExt + Unpin,
        W: AsyncWriteExt + Unpin,
{
    let mut buffer = vec![0; 1500];
    loop {
        match reader.read(&mut buffer).await {
            Ok(0) => break, // 连接关闭
            Ok(n) => {
                if let Err(e) = writer.write_all(&buffer[..n]).await {
                    error!("Failed to write: {}", e);
                    break;
                }
            }
            Err(e) => {
                error!("Failed to read: {}", e);
                break;
            }
        }
    }
    Ok(())
}
