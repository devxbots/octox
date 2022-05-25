use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Read;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};

use axum::routing::get;
use axum::{Extension, Router, Server};

pub use self::error::Error;
pub use self::github::{AppId, PrivateKey, WebhookSecret};

mod error;
mod github;

#[derive(Debug)]
pub struct Octox {
    app_id: Option<AppId>,
    private_key: Option<PrivateKey>,
    webhook_secret: Option<WebhookSecret>,
    address: SocketAddr,
    tcp_listener: Option<TcpListener>,
}

impl Octox {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn app_id(mut self, app_id: u64) -> Result<Self, Error> {
        self.app_id = Some(AppId::new(app_id));
        Ok(self)
    }

    pub fn private_key(mut self, private_key: &str) -> Result<Self, Error> {
        self.private_key = Some(PrivateKey::new(private_key.into()));
        Ok(self)
    }

    pub fn webhook_secret(mut self, webhook_secret: &str) -> Result<Self, Error> {
        self.webhook_secret = Some(WebhookSecret::new(webhook_secret.into()));
        Ok(self)
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
        let app = Router::new()
            .route("/", get(|| async { "Hello, World!" }))
            .layer(self.app_id_extension()?)
            .layer(self.private_key_extension()?)
            .layer(self.webhook_secret_extension()?);

        let listener = match self.tcp_listener {
            Some(listener) => listener,
            None => TcpListener::bind(self.address)?,
        };

        Server::from_tcp(listener)?
            .serve(app.into_make_service())
            .await?;

        Ok(())
    }

    fn app_id_extension(&self) -> Result<Extension<AppId>, Error> {
        if let Some(app_id) = self.app_id {
            return Ok(Extension(app_id));
        }

        let app_id_from_env = match std::env::var("OCTOX_APP_ID") {
            Ok(app_id) => app_id,
            Err(_) => {
                return Err(Error::Configuration(
                    "app id must be set either manually or as an environment variable".into(),
                ))
            }
        };

        if let Ok(app_id) = app_id_from_env.parse::<u64>() {
            return Ok(Extension(AppId::new(app_id)));
        }

        Err(Error::Configuration("app id must be a number".into()))
    }

    fn private_key_extension(&self) -> Result<Extension<PrivateKey>, Error> {
        if let Some(private_key) = &self.private_key {
            return Ok(Extension(private_key.clone()));
        }

        if let Ok(private_key) = std::env::var("OCTOX_PRIVATE_KEY") {
            return Ok(Extension(PrivateKey::new(private_key)));
        };

        let private_key_path = match std::env::var("OCTOX_PRIVATE_KEY_PATH") {
            Ok(path) => path,
            Err(_) => {
                return Err(Error::Configuration(
                    "private key must be set either manually or as an environment variable".into(),
                ))
            }
        };

        let mut private_key = String::new();
        File::open(private_key_path)?.read_to_string(&mut private_key)?;

        Ok(Extension(PrivateKey::new(private_key)))
    }

    fn webhook_secret_extension(&self) -> Result<Extension<WebhookSecret>, Error> {
        if let Some(webhook_secret) = &self.webhook_secret {
            return Ok(Extension(webhook_secret.clone()));
        }

        if let Ok(webhook_secret) = std::env::var("OCTOX_WEBHOOK_SECRET") {
            return Ok(Extension(WebhookSecret::new(webhook_secret)));
        }

        Err(Error::Configuration(
            "webhook secret must be set either manually or as an environment variable".into(),
        ))
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
            app_id: None,
            private_key: None,
            webhook_secret: None,
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
    fn app_id_sets_app_id() -> Result<(), Error> {
        let octox = Octox::new();

        let octox = octox.app_id(1)?;

        assert!(octox.app_id.is_some());
        Ok(())
    }

    #[test]
    fn private_key_sets_private_key() -> Result<(), Error> {
        let octox = Octox::new();

        let octox = octox.private_key("private_key")?;

        assert!(octox.private_key.is_some());
        Ok(())
    }

    #[test]
    fn webhook_secet_sets_webhook_secret() -> Result<(), Error> {
        let octox = Octox::new();

        let octox = octox.webhook_secret("webhook_secret")?;

        assert!(octox.webhook_secret.is_some());
        Ok(())
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
