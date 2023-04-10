use std::{borrow::Cow, fmt};

use rocket::figment::{Error, Figment};

pub struct OAuthConfig {
    provider: Provider,
    client_secret: String,
    client_id: String,
    redirect_uri: Option<String>,
}

impl OAuthConfig {
    fn new(
        provider: Provider,
        client_id: String,
        client_secret: String,
        redirect_uri: Option<String>,
    ) -> Self {
        OAuthConfig {
            provider,
            client_id,
            client_secret,
            redirect_uri,
        }
    }

    pub fn from_figment(figment: &Figment, name: &str) -> Result<Self, Error> {
        #[derive(rocket::serde::Deserialize)]
        struct Config {
            auth_uri: String,
            token_uri: String,
            client_id: String,
            client_secret: String,
            redirect_uri: Option<String>,
        }
        let conf: Config = figment.extract_inner(&format!("oauth.{name}"))?;
        let provider = Provider {
            auth_uri: conf.auth_uri.into(),
            token_uri: conf.token_uri.into(),
        };
        Ok(Self::new(
            provider,
            conf.client_id,
            conf.client_secret,
            conf.redirect_uri,
        ))
    }

    pub fn provider(&self) -> &Provider {
        &self.provider
    }

    pub fn client_id(&self) -> &str {
        &self.client_id
    }

    pub fn client_secret(&self) -> &str {
        &self.client_secret
    }

    pub fn redirect_uri(&self) -> Option<&str> {
        self.redirect_uri.as_deref()
    }
}

impl fmt::Debug for OAuthConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("OAuthConfig")
            .field("provider", &(..))
            .field("client_id", &self.client_id)
            .field("client_secret", &self.client_secret)
            .field("redirect_uri", &self.redirect_uri)
            .finish()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Provider {
    auth_uri: Cow<'static, str>,
    token_uri: Cow<'static, str>,
}

impl Provider {
    pub fn auth_uri(&self) -> &str {
        &self.auth_uri
    }

    pub fn token_uri(&self) -> &str {
        &self.token_uri
    }
}
