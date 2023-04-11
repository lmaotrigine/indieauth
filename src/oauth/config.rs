use std::{borrow::Cow, collections::HashMap, fmt};

use rocket::figment::{Error, Figment};

use crate::api::Result;

pub struct OAuthConfig {
    provider: Provider,
    client_secret: Option<String>, // if not present, use PKCE
    client_id: String,
    redirect_uri: Option<String>,
    states: HashMap<String, String>, // state -> code_verifier
}

impl OAuthConfig {
    fn new(
        provider: Provider,
        client_id: String,
        client_secret: Option<String>,
        redirect_uri: Option<String>,
    ) -> Self {
        OAuthConfig {
            provider,
            client_id,
            client_secret,
            redirect_uri,
            states: HashMap::new(),
        }
    }

    pub fn from_figment(figment: &Figment, name: &str) -> std::result::Result<Self, Error> {
        #[derive(rocket::serde::Deserialize)]
        struct Config {
            auth_uri: String,
            token_uri: String,
            client_id: String,
            client_secret: Option<String>,
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

    pub fn client_secret(&self) -> Option<&str> {
        self.client_secret.as_deref()
    }

    pub fn redirect_uri(&self) -> Option<&str> {
        self.redirect_uri.as_deref()
    }

    pub fn insert(&mut self, state: String, code_verifier: String) {
        self.states.insert(state, code_verifier);
    }

    pub fn take(&mut self, state: &str) -> Result<String> {
        self.states
            .remove(state)
            .ok_or(crate::api::Error::OAuth2("invalid state".into()))
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
