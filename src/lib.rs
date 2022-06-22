use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Read;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::ops::Deref;
use std::sync::Arc;

use anyhow::Context;
use axum::routing::{get, post};
use axum::{Extension, Router, Server};
use github_parts::github::app::AppId;
use github_parts::github::token::TokenFactory;
use github_parts::github::{GitHubHost, PrivateKey, WebhookSecret};
use parking_lot::Mutex;
use sentry_tower::{NewSentryLayer, SentryHttpLayer};
use tower_http::trace::TraceLayer;

use crate::routes::{health, webhook};

pub use self::error::Error;
pub use self::state::State;
pub use self::workflow::{Step, Transition, Workflow, WorkflowError};

mod auth;
mod error;
mod routes;
mod state;
mod workflow;

type SharedTokenFactory = Arc<Mutex<TokenFactory>>;
type WorkflowConstructor = fn(GitHubHost, AppId, PrivateKey) -> Box<dyn Workflow>;

#[derive(Debug)]
pub struct Octox {
    github_host: GitHubHost,
    app_id: Option<AppId>,
    private_key: Option<PrivateKey>,
    webhook_secret: Option<WebhookSecret>,
    socket_address: SocketAddr,
    tcp_listener: Option<TcpListener>,
    workflow: Option<WorkflowConstructor>,
}

impl Octox {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn workflow(mut self, workflow: WorkflowConstructor) -> Result<Self, Error> {
        self.workflow = Some(workflow);
        Ok(self)
    }

    pub fn github_host(mut self, github_host: String) -> Result<Self, Error> {
        self.github_host = GitHubHost::new(github_host);
        Ok(self)
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

    pub fn socket_address(mut self, address: SocketAddr) -> Result<Self, Error> {
        self.socket_address = address;
        self.tcp_listener = None;
        Ok(self)
    }

    pub fn tcp_listener(mut self, listener: TcpListener) -> Result<Self, Error> {
        self.socket_address = listener
            .local_addr()
            .context("failed to get socket address from TCP listener")?;

        self.tcp_listener = Some(listener);

        Ok(self)
    }

    pub async fn serve(self) -> Result<(), Error> {
        let app = Router::new()
            .route("/", post(webhook))
            .route("/health", get(health))
            .layer(TraceLayer::new_for_http())
            .layer(NewSentryLayer::new_from_top())
            .layer(SentryHttpLayer::with_transaction())
            .layer(self.github_host_extension()?)
            .layer(self.token_factory_extension()?)
            .layer(self.webhook_secret_extension()?)
            .layer(self.workflow_extension()?);

        let listener = match self.tcp_listener {
            Some(listener) => listener,
            None => {
                TcpListener::bind(self.socket_address).context("failed to bind socket address")?
            }
        };

        Server::from_tcp(listener)
            .context("failed to create HTTP server from TCP listener")?
            .serve(app.into_make_service())
            .await
            .context("failed to start HTTP server")?;

        Ok(())
    }

    fn workflow_extension(&self) -> Result<Extension<Arc<Box<dyn Workflow>>>, Error> {
        if let Some(workflow) = &self.workflow {
            let github_host = self.github_host.clone();
            let app_id = self.try_app_id()?;
            let private_key = self.try_private_key()?;

            let constructor = workflow.deref();
            let workflow = constructor(github_host, app_id, private_key);

            return Ok(Extension(Arc::new(workflow)));
        }

        Err(Error::Configuration("workflow must be set".into()))
    }

    fn github_host_extension(&self) -> Result<Extension<GitHubHost>, Error> {
        Ok(Extension(self.github_host.clone()))
    }

    fn token_factory_extension(&self) -> Result<Extension<SharedTokenFactory>, Error> {
        let github_host = self.github_host.clone();
        let app_id = self.try_app_id()?;
        let private_key = self.try_private_key()?;

        let factory = TokenFactory::new(github_host, app_id, private_key);
        let shared_factory = Arc::new(Mutex::new(factory));

        Ok(Extension(shared_factory))
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

    fn try_app_id(&self) -> Result<AppId, Error> {
        if let Some(app_id) = self.app_id {
            return Ok(app_id);
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
            return Ok(AppId::new(app_id));
        }

        Err(Error::Configuration("app id must be a number".into()))
    }

    fn try_private_key(&self) -> Result<PrivateKey, Error> {
        if let Some(private_key) = &self.private_key {
            return Ok(private_key.clone());
        }

        if let Ok(private_key) = std::env::var("OCTOX_PRIVATE_KEY") {
            return Ok(PrivateKey::new(private_key));
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
        File::open(private_key_path)
            .context("failed to open private key")?
            .read_to_string(&mut private_key)
            .context("failed to read private key")?;

        Ok(PrivateKey::new(private_key))
    }
}

impl Display for Octox {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Octox {{}}")
    }
}

impl Default for Octox {
    fn default() -> Self {
        let github_host = "https://api.github.com".into();
        let socket_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 3000);

        Self {
            github_host,
            app_id: None,
            private_key: None,
            webhook_secret: None,
            socket_address,
            tcp_listener: None,
            workflow: None,
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

        assert_eq!("127.0.0.1:3000", octox.socket_address.to_string());
    }

    #[test]
    fn github_host_sets_github_host() -> Result<(), Error> {
        let octox = Octox::new();

        let octox = octox.github_host("github_host".into())?;

        assert_eq!("github_host", octox.github_host.get());
        Ok(())
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

        let octox = octox.socket_address("127.0.0.1:8000".parse::<SocketAddr>().unwrap())?;

        assert_eq!("127.0.0.1:8000", octox.socket_address.to_string());
        Ok(())
    }

    #[test]
    fn address_resets_listener() -> Result<(), Error> {
        let octox = Octox::new();

        let octox = octox
            .tcp_listener(TcpListener::bind("0.0.0.0:0".parse::<SocketAddr>().unwrap()).unwrap())?;
        let octox = octox.socket_address("127.0.0.1:8000".parse::<SocketAddr>().unwrap())?;

        assert_eq!("127.0.0.1:8000", octox.socket_address.to_string());
        assert!(octox.tcp_listener.is_none());
        Ok(())
    }

    #[test]
    fn listener_resets_address() -> Result<(), Error> {
        let octox = Octox::new();

        let tcp_listener = TcpListener::bind("0.0.0.0:0".parse::<SocketAddr>().unwrap()).unwrap();
        let address = tcp_listener.local_addr().unwrap();

        let octox = octox.tcp_listener(tcp_listener)?;

        assert_eq!(address, octox.socket_address);
        Ok(())
    }
}
