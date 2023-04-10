use rocket::{http::Status, response::Responder};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("internal database error: {0}")]
    Database(#[from] diesel::result::Error),
    #[error("not found")]
    NotFound,
    #[error("paseto creation error: {0}")]
    PasetoCreationError(String),
    #[error("paseto validation error: {0}")]
    PasetoValidationError(String),
    #[error("no paseto in request")]
    NoPasetoInRequest,
    #[error("wrong indieauth response type: {0}")]
    WrongIndieAuthResponseType(String),
    #[error("Invalid code verifier: {0}")]
    InvalidCodeVerifier(String),
    #[error("code challenge method {0} not supported. Must be S256")]
    WrongIndieAuthCodeChallengeMethod(String),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Invalid URI: '{0}'")]
    InvalidUri(String),
    #[error("failed to exchange token")]
    ExchangeFailure,
    #[error("token exchange returned non-success status code: {0}")]
    ExchangeError(u16),
    #[error("OAuth2 error: {0}")]
    OAuth2(String),
}

pub type Result<T = ()> = std::result::Result<T, Error>;

impl<'r, 'o: 'r> Responder<'r, 'o> for Error {
    fn respond_to(self, _: &'r rocket::Request<'_>) -> rocket::response::Result<'o> {
        match self {
            Error::NotFound => Err(Status::NotFound),
            Error::WrongIndieAuthResponseType(_) => Err(Status::BadRequest),
            Error::WrongIndieAuthCodeChallengeMethod(_) | Error::InvalidCodeVerifier(_) => {
                Err(Status::BadRequest)
            }
            _ => Err(Status::InternalServerError),
        }
    }
}
