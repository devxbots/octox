use std::fmt::{Display, Formatter};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};

use axum::routing::get;
use axum::{Router, Server};

pub use self::error::Error;

mod error;

#[derive(Debug)]
pub struct Octox {
    address: SocketAddr,
    tcp_listener: Option<TcpListener>,
}

impl Octox {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn address(mut self, address: SocketAddr) -> Result<Self, Error> {
        self.address = address;
        self.tcp_listener = None;
        Ok(self)
    }

    pub fn listener(mut self, listener: TcpListener) -> Result<Self, Error> {
        self.address = listener.local_addr()?;
        self.tcp_listener = Some(listener);
        Ok(self)
    }

    pub async fn serve(self) -> Result<(), Error> {
        let app = Router::new().route("/", get(|| async { "Hello, World!" }));

        let listener = match self.tcp_listener {
            Some(listener) => listener,
            None => TcpListener::bind(self.address)?,
        };

        Server::from_tcp(listener)?
            .serve(app.into_make_service())
            .await?;

        Ok(())
    }
}

impl Display for Octox {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Octox {{}}")
    }
}

impl Default for Octox {
    fn default() -> Self {
        Self {
            address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 3000),
            tcp_listener: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::{SocketAddr, TcpListener};

    use super::{Error, Octox};

    #[test]
    fn new_returns_default_instance() {
        let octox = Octox::new();

        assert_eq!("127.0.0.1:3000", octox.address.to_string());
    }

    #[test]
    fn address_sets_address() -> Result<(), Error> {
        let octox = Octox::new();

        let octox = octox.address("127.0.0.1:8000".parse::<SocketAddr>().unwrap())?;

        assert_eq!("127.0.0.1:8000", octox.address.to_string());

        Ok(())
    }

    #[test]
    fn address_resets_listener() -> Result<(), Error> {
        let octox = Octox::new();

        let octox = octox.listener(TcpListener::bind(
            "0.0.0.0:0".parse::<SocketAddr>().unwrap(),
        )?)?;
        let octox = octox.address("127.0.0.1:8000".parse::<SocketAddr>().unwrap())?;

        assert_eq!("127.0.0.1:8000", octox.address.to_string());
        assert!(octox.tcp_listener.is_none());

        Ok(())
    }

    #[test]
    fn listener_resets_address() -> Result<(), Error> {
        let octox = Octox::new();

        let tcp_listener = TcpListener::bind("0.0.0.0:0".parse::<SocketAddr>().unwrap())?;
        let address = tcp_listener.local_addr()?;

        let octox = octox.listener(tcp_listener)?;

        assert_eq!(address, octox.address);

        Ok(())
    }
}
