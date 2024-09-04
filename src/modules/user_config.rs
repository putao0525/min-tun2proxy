use clap::{Arg, Command};
use tun::Configuration;

#[derive(Debug, PartialEq, Eq)]
pub enum ProxyType {
    Http,
    Socks5,
}

#[derive(Debug)]
pub struct UserConfig {
    pub proxy_type: ProxyType,
    proxy_addr: String,
    pub is_use_proxy_pool: bool,
    pub is_free_route_table: bool,
    pub tun_conf: Configuration,
}

impl UserConfig {
    pub fn get_proxy_addr(&self) -> String {
        self.proxy_addr.clone()
    }

    pub fn parse_params() -> Self {
        let matches = Command::new("min-tun2proxy")
            .version("0.0.1")
            .author("putao0525")
            .about("一个简单的vpn")
            .arg(
                Arg::new("proxy_addr")
                    .short('p')
                    .long("proxy_addr")
                    .required(true)
                    .value_name("ip:port")
                    .help("配置一个代理IP的地址,socks5地址,格式: -p 192.168.2:8090")
            )
            .get_matches();
        let p_addr = matches.get_one::<String>("proxy_addr").expect("必须配置代理IP").to_string();

        let mut config = Configuration::default();
        config.address((10, 0, 0, 1)); // 设置 TUN 设备的 IP 地址
        config.netmask((255, 255, 255, 0)); // 设置 TUN 设备的子网掩码
        config.mtu(1500); // 设置 TUN 设备的 MTU
        Self {
            proxy_type: ProxyType::Socks5,
            proxy_addr: p_addr,
            is_use_proxy_pool: false,
            is_free_route_table: true,
            tun_conf: config,
        }
    }
}