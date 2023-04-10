use paseto::{tokens::PasetoPublicKey, validate_public_token};
use ring::signature::Ed25519KeyPair;
use rocket::{
    fairing::AdHoc,
    http::Status,
    request::{self, FromRequest},
    Request, State,
};
use serde::{Deserialize, Serialize};
use tokio::sync::OnceCell;

use crate::api::Error;

static KP: OnceCell<Ed25519KeyPair> = OnceCell::const_new();

pub fn ed25519_keypair() -> AdHoc {
    async fn fairing(rocket: rocket::Rocket<rocket::Build>) -> rocket::Rocket<rocket::Build> {
        let public: String = rocket.figment().extract_inner("paseto.public").unwrap();
        let private: String = rocket.figment().extract_inner("paseto.private").unwrap();
        let public = hex::decode(public).unwrap();
        let private = hex::decode(private).unwrap();
        let kp = Ed25519KeyPair::from_seed_and_public_key(&private, &public).unwrap();
        KP.set(kp).unwrap();
        rocket
            .manage(PasetoPublicKey::ED25519KeyPair(KP.get().unwrap()))
            .manage(Keypair { public, private })
    }
    AdHoc::on_ignite("Paseto", fairing)
}

#[derive(Debug, Clone)]
pub struct Keypair {
    public: Vec<u8>,
    private: Vec<u8>,
}

impl Keypair {
    pub fn ed25519_keypair(&self) -> Ed25519KeyPair {
        Ed25519KeyPair::from_seed_and_public_key(&self.private, &self.public).unwrap()
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Token {
    pub jti: String,
    pub sub: String,
    pub aud: String,
    pub iss: String,
    pub iat: String,
    pub scopes: Option<Vec<String>>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Token {
    type Error = crate::api::Error;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let keys = request.headers().get("Authorization").collect::<Vec<_>>();
        match keys.len() {
            0 => {
                let cookies = request.cookies();
                let tok = cookies.get_private("token");
                match tok {
                    None => {
                        request::Outcome::Failure((Status::Unauthorized, Error::NoPasetoInRequest))
                    }
                    Some(cook) => {
                        let tok = cook.value().to_string();
                        let paseto_key = request.guard::<&State<PasetoPublicKey>>().await.unwrap();
                        match validate_public_token(
                            &tok,
                            None,
                            paseto_key,
                            &paseto::TimeBackend::Chrono,
                        ) {
                            Ok(val) => {
                                let tok: Token = serde_json::from_value(val).unwrap();
                                request::Outcome::Success(tok)
                            }
                            Err(why) => request::Outcome::Failure((
                                Status::Unauthorized,
                                Error::PasetoValidationError(why.to_string()),
                            )),
                        }
                    }
                }
            }
            1 => {
                let tok = keys[0];
                let paseto_key = request.guard::<&State<PasetoPublicKey>>().await.unwrap();
                match validate_public_token(tok, None, paseto_key, &paseto::TimeBackend::Chrono) {
                    Ok(val) => {
                        let tok: Token = serde_json::from_value(val).unwrap();
                        request::Outcome::Success(tok)
                    }
                    Err(why) => request::Outcome::Failure((
                        Status::Unauthorized,
                        Error::PasetoValidationError(why.to_string()),
                    )),
                }
            }
            _ => request::Outcome::Failure((Status::Unauthorized, Error::NoPasetoInRequest)),
        }
    }
}
