use secrecy::{ExposeSecret, SecretString};

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct AppId(u64);

impl AppId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn get(&self) -> u64 {
        self.0
    }
}

#[derive(Clone, Debug)]
pub struct PrivateKey(SecretString);

impl PrivateKey {
    pub fn new(private_key: String) -> Self {
        Self(SecretString::new(private_key))
    }

    pub fn get(&self) -> &str {
        self.0.expose_secret()
    }
}

#[derive(Clone, Debug)]
pub struct WebhookSecret(SecretString);

impl WebhookSecret {
    pub fn new(webhook_secret: String) -> Self {
        Self(SecretString::new(webhook_secret))
    }

    pub fn get(&self) -> &str {
        self.0.expose_secret()
    }
}

#[cfg(test)]
mod tests {
    use super::{AppId, PrivateKey, WebhookSecret};

    mod app_id {
        use super::AppId;

        #[test]
        fn app_id() {
            let app_id = AppId::new(1);
            assert_eq!(1, app_id.get());
        }

        #[test]
        fn trait_send() {
            fn assert_send<T: Send>() {}
            assert_send::<AppId>();
        }

        #[test]
        fn trait_sync() {
            fn assert_sync<T: Sync>() {}
            assert_sync::<AppId>();
        }
    }

    mod private_key {
        use super::PrivateKey;

        #[test]
        fn private_key() {
            let private_key = PrivateKey::new("private_key".into());
            assert_eq!("private_key", private_key.get());
        }

        #[test]
        fn trait_send() {
            fn assert_send<T: Send>() {}
            assert_send::<PrivateKey>();
        }

        #[test]
        fn trait_sync() {
            fn assert_sync<T: Sync>() {}
            assert_sync::<PrivateKey>();
        }
    }

    mod webhook_secret {
        use super::WebhookSecret;

        #[test]
        fn webhook_secret() {
            let webhook_secret = WebhookSecret::new("webhook_secret".into());
            assert_eq!("webhook_secret", webhook_secret.get());
        }

        #[test]
        fn trait_send() {
            fn assert_send<T: Send>() {}
            assert_send::<WebhookSecret>();
        }

        #[test]
        fn trait_sync() {
            fn assert_sync<T: Sync>() {}
            assert_sync::<WebhookSecret>();
        }
    }
}
