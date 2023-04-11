use diesel::prelude::*;
use rocket::{
    get,
    http::{Cookie, CookieJar},
    response::Redirect,
    serde::{Deserialize, Serialize},
    State,
};
use rusty_ulid::generate_ulid_string;
use tracing::instrument;

use crate::{
    api::{self, Error, Result},
    models,
    oauth::{OAuth2, TokenResponse},
    paseto::{Keypair, Token},
    schema::gitlab_tokens,
    GitLab, MainDatabase, APPLICATION_NAME,
};

const ALLOWED_USERS: [i32; 1] = [34];

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct User {
    // these are all we care about
    id: i32,
    name: String,
}

async fn user(token: String) -> Result<User> {
    let c = reqwest::Client::new();
    let r = c
        .get("https://git.5ht2.me/api/v4/user")
        .header("Authorization", format!("Bearer {token}"))
        .header("User-Agent", APPLICATION_NAME)
        .build()
        .map_err(|why| Error::OAuth2(format!("{why}")))?;
    let res = c
        .execute(r)
        .await
        .map_err(|why| Error::OAuth2(format!("{why}")))?;
    let bytes = res
        .bytes()
        .await
        .map_err(|why| Error::OAuth2(format!("{why}")))?;
    let u: User = serde_json::from_slice(&bytes).map_err(Error::Json)?;
    Ok(u)
}

#[get("/")]
#[instrument(skip(oauth2, cookies))]
pub async fn login(oauth2: OAuth2<GitLab>, cookies: &CookieJar<'_>) -> Redirect {
    oauth2.get_redirect(cookies, &["read_user"]).await.unwrap()
}

#[get("/callback")]
#[instrument(skip(db, kp, token, cookies), err)]
pub async fn callback(
    db: MainDatabase,
    kp: &State<Keypair>,
    token: TokenResponse<GitLab>,
    cookies: &CookieJar<'_>,
) -> Result<String> {
    let tok = token.access_token().to_string();
    let refresh_token = token.refresh_token().unwrap().to_string();
    let gitlab_user = user(tok.clone())
        .await
        .map_err(|why| Error::OAuth2(format!("{why}")))?;
    if !ALLOWED_USERS.contains(&gitlab_user.id) {
        return Err(Error::OAuth2(
            "I'm sorry Dave, I'm afraid I can't do that.".into(),
        ));
    }
    let tok = models::GitlabToken {
        id: generate_ulid_string(),
        user_id: gitlab_user.id,
        access_token: tok,
        refresh_token,
    };
    db.run(move |c| {
        diesel::insert_into(gitlab_tokens::table)
            .values(&tok)
            .execute(c)
    })
    .await
    .map_err(Error::Database)
    .map_err(|e| Error::OAuth2(format!("{e}")))?;
    let tok = api::token::mint(
        db,
        Token::default(),
        kp,
        "https://5ht2.me".into(),
        gitlab_user.name,
    )
    .await
    .map_err(|why| Error::OAuth2(format!("{why}")))?;
    cookies.add_private(Cookie::build("token", tok.clone()).path("/").finish());
    Ok(tok)
}
