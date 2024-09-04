#[derive(Debug, PartialEq, Eq)]
pub enum ProxyType {
    Http,
    Socks5,
}

pub struct UserConfig {
    pub proxy_type: ProxyType,
    proxy_addr: String,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            proxy_type: ProxyType::Socks5,
            proxy_addr: String::from("192.168.2:8090"),
        }
    }
}

impl UserConfig {
    pub fn get_proxy_addr(&self) -> String {
        self.proxy_addr.clone()
    }
}