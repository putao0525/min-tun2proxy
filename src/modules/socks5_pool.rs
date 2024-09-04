use std::io;
use std::net::{SocketAddr};
use async_trait::async_trait;
use bb8::ManageConnection;
use tokio::net::TcpStream;
use tokio_socks::tcp::Socks5Stream;
use tokio::io::{AsyncWriteExt, AsyncReadExt};


pub struct Socks5ConnectionManager {
    pub proxy_addr: SocketAddr,
    pub target_addr: SocketAddr,
}

#[async_trait]
impl ManageConnection for Socks5ConnectionManager {
    type Connection = Socks5Stream<TcpStream>;
    type Error = io::Error;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        let stream = TcpStream::connect(self.proxy_addr).await?;
        let rt = Socks5Stream::connect_with_socket(stream, self.target_addr).await;
        Ok(rt.unwrap())
    }

    async fn is_valid(&self, conn: &mut Self::Connection) -> Result<(), Self::Error> {
        // 发送一个简单的请求来验证连接是否有效
        let request = b"";
        conn.write_all(request).await?;
        let mut response = [0; 1];
        conn.read_exact(&mut response).await?;
        Ok(())
    }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        false
    }
}