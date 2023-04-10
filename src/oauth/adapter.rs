use reqwest::Url;
use rocket::http::{ext::IntoOwned, uri::Absolute};

use crate::api::{Error, Result};

use super::{config::OAuthConfig, TokenRequest, TokenResponse};

#[derive(Clone, Debug)]
pub struct Adapter {
    client: reqwest::Client,
}

impl Default for Adapter {
    fn default() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

impl Adapter {
    pub fn authorization_url(
        &self,
        config: &OAuthConfig,
        state: &str,
        scopes: &[&str],
        extra_params: &[(&str, &str)],
    ) -> Result<Absolute<'static>> {
        let auth_uri = config.provider().auth_uri();
        let mut url = Url::parse(auth_uri).map_err(|_| Error::InvalidUri(auth_uri.to_string()))?;
        url.query_pairs_mut()
            .append_pair("response_type", "code")
            .append_pair("client_id", config.client_id())
            .append_pair("state", state);
        if let Some(redirect_uri) = config.redirect_uri() {
            url.query_pairs_mut()
                .append_pair("redirect_uri", redirect_uri);
        }
        if !scopes.is_empty() {
            url.query_pairs_mut()
                .append_pair("scope", &scopes.join(" "));
        }
        for (name, value) in extra_params {
            match *name {
                "response_type" | "client_id" | "state" => continue,
                "redirect_uri" if config.redirect_uri().is_some() => continue,
                "scope" if !scopes.is_empty() => continue,
                _ => url.query_pairs_mut().append_pair(name, value),
            };
        }
        Ok(Absolute::parse(url.as_ref())
            .map_err(|_| Error::InvalidUri(url.to_string()))?
            .into_owned())
    }

    pub async fn exchange_code<K>(
        &self,
        config: &OAuthConfig,
        token: TokenRequest,
    ) -> Result<TokenResponse<K>> {
        let rb = self
            .client
            .post(config.provider().token_uri())
            .header(reqwest::header::ACCEPT, "application/json".to_string())
            .header(
                reqwest::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded".to_string(),
            );
        let req_str = {
            let mut ser = url::form_urlencoded::Serializer::new(String::new());
            match token {
                TokenRequest::AuthorizationCode(code) => {
                    ser.append_pair("grant_type", "authorization_code");
                    ser.append_pair("code", &code);
                    if let Some(redirect_uri) = config.redirect_uri() {
                        ser.append_pair("redirect_uri", redirect_uri);
                    }
                }
                TokenRequest::RefreshToken(token) => {
                    ser.append_pair("grant_type", "refresh_token");
                    ser.append_pair("refresh_token", &token);
                }
            }
            ser.append_pair("client_id", config.client_id());
            ser.append_pair("client_secret", config.client_secret());
            ser.finish()
        };
        let req = rb
            .body(req_str)
            .build()
            .map_err(|_| Error::ExchangeFailure)?;
        let resp = self
            .client
            .execute(req)
            .await
            .map_err(|_| Error::ExchangeFailure)?;
        if !resp.status().is_success() {
            return Err(Error::ExchangeError(resp.status().as_u16()));
        }
        let body = resp.bytes().await.map_err(|_| Error::ExchangeFailure)?;
        let tr: TokenResponse<K> = serde_json::from_slice(&body).map_err(Error::Json)?;
        Ok(tr)
    }
}
