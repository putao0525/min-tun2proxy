use tun::Configuration;

#[derive(Debug, PartialEq, Eq)]
pub enum ProxyType {
    Http,
    Socks5,
}

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
        let mut config = Configuration::default();
        config.address((10, 0, 0, 1)); // 设置 TUN 设备的 IP 地址
        config.netmask((255, 255, 255, 0)); // 设置 TUN 设备的子网掩码
        config.mtu(1500); // 设置 TUN 设备的 MTU
        Self {
            proxy_type: ProxyType::Socks5,
            proxy_addr: String::from("192.168.2:8090"),
            is_use_proxy_pool: false,
            is_free_route_table: true,
            tun_conf: config,
        }
    }
}