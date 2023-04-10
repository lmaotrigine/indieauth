use std::{fmt, marker::PhantomData, sync::Arc};

use base64::{engine::general_purpose::URL_SAFE_NO_PAD as BASE64_URL_SAFE_NO_PAD, Engine};
use rocket::{
    fairing::{AdHoc, Fairing},
    form::{Form, FromForm},
    http::{Cookie, CookieJar, SameSite, Status},
    request::{self, FromRequest},
    response::Redirect,
    Build, Ignite, Request, Rocket, Sentinel,
};
use serde::Deserialize;

use crate::api::{Error, Result};

use self::config::OAuthConfig;

mod adapter;
mod config;
pub mod pkce;

fn generate_state(rng: &mut impl rand::RngCore) -> Result<String> {
    let mut buf = [0; 128]; // 1024 bits
    rng.try_fill_bytes(&mut buf)
        .map_err(|_| Error::OAuth2("failed to generate random data".into()))?;
    Ok(BASE64_URL_SAFE_NO_PAD.encode(buf))
}

#[derive(Clone, Debug, PartialEq)]
pub enum TokenRequest {
    AuthorizationCode(String),
    RefreshToken(String),
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct TokenResponse<K> {
    data: TokenResponseData,
    #[serde(skip)]
    _k: PhantomData<fn() -> K>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
struct TokenResponseData {
    access_token: String,
    token_type: String,
    expires_in: Option<u64>,
    refresh_token: Option<String>,
    created_at: Option<u64>,
    scope: Option<String>,
}

impl<K> TokenResponse<K> {
    pub fn access_token(&self) -> &str {
        &self.data.access_token
    }

    pub fn refresh_token(&self) -> Option<&str> {
        self.data.refresh_token.as_deref()
    }
}

struct Shared<K> {
    adapter: adapter::Adapter,
    config: config::OAuthConfig,
    _k: PhantomData<fn() -> TokenResponse<K>>,
}

pub struct OAuth2<K>(Arc<Shared<K>>);

#[rocket::async_trait]
impl<'r, K: 'static> FromRequest<'r> for TokenResponse<K> {
    type Error = Error;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let oauth2 = request
            .rocket()
            .state::<Arc<Shared<K>>>()
            .expect("OAuth2 fairing was not attached for this key type!");
        let query = match request.uri().query() {
            Some(q) => q,
            None => {
                return request::Outcome::Failure((
                    Status::BadRequest,
                    Error::OAuth2("missing query string in request".into()),
                ));
            }
        };

        #[derive(FromForm)]
        struct CallbackQuery {
            code: String,
            state: String,
            scope: Option<String>,
        }
        let params = match Form::<CallbackQuery>::parse_encoded(&query) {
            Ok(p) => p,
            Err(e) => {
                tracing::warn!("Failed to parse OAuth2 query string: {e:?}");
                return request::Outcome::Failure((
                    Status::BadRequest,
                    Error::OAuth2(format!("{e:?}")),
                ));
            }
        };
        {
            let cookies = request
                .guard::<&CookieJar<'_>>()
                .await
                .expect("request cookies");
            match cookies.get_private("oauth2_state") {
                Some(ref cookie) if cookie.value() == params.state => {
                    cookies.remove(cookie.clone());
                }
                other => {
                    if other.is_some() {
                        tracing::warn!("the OAuth2 state returned from the server did not match the stored state.");
                    } else {
                        tracing::error!("The OAuth2 state cookie was missing. It may have been blocked by the client?");
                    }
                    return request::Outcome::Failure((Status::BadRequest, Error::OAuth2("The OAuth2 state returned from the server did not match the stored state".into())));
                }
            }
        }
        match oauth2
            .adapter
            .exchange_code(&oauth2.config, TokenRequest::AuthorizationCode(params.code))
            .await
        {
            Ok(mut token) => {
                if token.data.scope.is_none() && params.scope.is_some() {
                    token.data.scope = params.scope;
                }
                request::Outcome::Success(token)
            }
            Err(e) => {
                tracing::warn!("OAuth2 token exchange failed: {e}");
                request::Outcome::Failure((Status::BadRequest, e))
            }
        }
    }
}

fn sentinel_abort<K: 'static>(rocket: &Rocket<Ignite>, wrapper: &str) -> bool {
    if rocket.state::<Arc<Shared<K>>>().is_some() {
        return false;
    }
    let type_name = std::any::type_name::<K>();
    tracing::error!(
        "{wrapper}<{type_name}> was used in a mounted route without attaching a matching fairing"
    );
    tracing::info!(
        "attach either OAuth2::<{type_name}>::fairing() or OAuth2::<{type_name}>::custom()"
    );
    true
}

impl<K: 'static> Sentinel for TokenResponse<K> {
    fn abort(rocket: &Rocket<Ignite>) -> bool {
        sentinel_abort::<K>(rocket, "TokenResponse")
    }
}

impl<K: 'static> OAuth2<K> {
    pub fn fairing(config_name: impl AsRef<str> + Send + 'static) -> impl Fairing {
        AdHoc::try_on_ignite("oauth2::fairing", |rocket| async move {
            let config =
                match config::OAuthConfig::from_figment(rocket.figment(), config_name.as_ref()) {
                    Ok(c) => c,
                    Err(e) => {
                        tracing::error!("invalid configuration: {e:?}");
                        return Err(rocket);
                    }
                };
            Ok(Self::_init(rocket, adapter::Adapter::default(), config))
        })
    }

    fn _init(
        rocket: Rocket<Build>,
        adapter: adapter::Adapter,
        config: config::OAuthConfig,
    ) -> Rocket<Build> {
        rocket.manage(Arc::new(Shared::<K> {
            adapter,
            config,
            _k: PhantomData,
        }))
    }

    pub fn custom(adapter: adapter::Adapter, config: OAuthConfig) -> impl Fairing {
        AdHoc::on_ignite("oauth2::custom", |rocket| async {
            Self::_init(rocket, adapter, config)
        })
    }

    pub fn get_redirect(&self, cookies: &CookieJar<'_>, scopes: &[&str]) -> Result<Redirect> {
        self.get_redirect_extras(cookies, scopes, &[])
    }

    fn get_redirect_extras(
        &self,
        cookies: &CookieJar<'_>,
        scopes: &[&str],
        extras: &[(&str, &str)],
    ) -> Result<Redirect> {
        let state = generate_state(&mut rand::thread_rng())?;
        let uri = self
            .0
            .adapter
            .authorization_url(&self.0.config, &state, scopes, extras)?;
        cookies.add_private(
            Cookie::build("oauth2_state", state)
                .same_site(SameSite::Lax)
                .finish(),
        );
        Ok(Redirect::to(uri))
    }

    pub async fn refresh(&self, refresh_token: &str) -> Result<TokenResponse<K>> {
        self.0
            .adapter
            .exchange_code(
                &self.0.config,
                TokenRequest::RefreshToken(refresh_token.to_string()),
            )
            .await
    }
}

#[rocket::async_trait]
impl<'r, K: 'static> FromRequest<'r> for OAuth2<K> {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        request::Outcome::Success(OAuth2(
            request
                .rocket()
                .state::<Arc<Shared<K>>>()
                .expect("OAuth2 fairing was not attached for this key type!")
                .clone(),
        ))
    }
}

impl<K: 'static> Sentinel for OAuth2<K> {
    fn abort(rocket: &Rocket<Ignite>) -> bool {
        sentinel_abort::<K>(rocket, "OAuth2")
    }
}

impl<C: fmt::Debug> fmt::Debug for OAuth2<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OAuth2")
            .field("adapter", &(..))
            .field("config", &self.0.config)
            .finish()
    }
}
