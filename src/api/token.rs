use ::paseto::PasetoBuilder;
use chrono::Utc;
use diesel::prelude::*;
use rocket::{get, post, serde::json::Json, State};
use rusty_ulid::generate_ulid_string;
use tracing::instrument;

use crate::{models, paseto, schema, MainDatabase, APPLICATION_NAME};

use super::{Error, Result};

#[get("/token/info")]
pub async fn info(tok: paseto::Token) -> Json<paseto::Token> {
    Json(tok)
}

#[post("/token/mint?<aud>&<sub>")]
#[instrument(skip(kp, conn), err)]
pub async fn mint(
    conn: MainDatabase,
    tok: paseto::Token,
    kp: &State<paseto::Keypair>,
    aud: String,
    sub: String,
) -> Result<String> {
    let kp = kp.inner().ed25519_keypair();
    let now = Utc::now();
    let tok = models::Token {
        id: generate_ulid_string(),
        sub: sub.clone(),
        aud: aud.clone(),
        iat: now.to_rfc3339(),
        iss: APPLICATION_NAME.into(),
        exp: None,
        valid: None,
    };
    let clone = tok.clone();
    conn.run(move |c| {
        diesel::insert_into(schema::tokens::table)
            .values(&tok)
            .execute(c)
            .map_err(Error::Database)
    })
    .await?;
    PasetoBuilder::new()
        .set_ed25519_key(&kp)
        .set_issued_at(Some(now))
        .set_issuer(&format!("api call from {}", clone.sub))
        .set_audience(&aud)
        .set_jti(&clone.id)
        .set_subject(&sub)
        .build()
        .map_err(|why| {
            tracing::error!("can't make paseto: {why}");
            Error::PasetoCreationError(format!("{why}"))
        })
}
