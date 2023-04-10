use diesel::prelude::*;

use crate::schema::*;

#[derive(Queryable, Insertable, Clone)]
#[diesel(table_name = tokens)]
pub struct Token {
    pub id: String,
    pub sub: String,
    pub aud: String,
    pub iss: String,
    pub iat: String,
    pub exp: Option<i32>,
    pub valid: Option<i32>,
}

#[derive(Queryable, Debug, Clone, Insertable)]
#[diesel(table_name = indieauth_codes)]
pub struct IndieauthCode {
    pub code: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub state: String,
    pub response_type: String,
    pub code_challenge: String,
    pub authorized: bool,
}

#[derive(AsChangeset)]
#[diesel(table_name = indieauth_codes)]
pub struct UpdateIndieauthCodeAuthorized {
    pub authorized: bool,
}

#[derive(Queryable, Debug, Clone, Insertable)]
#[diesel(table_name = gitlab_tokens)]
pub struct GitlabToken {
    pub id: String,
    pub user_id: i32,
    pub access_token: String,
    pub refresh_token: String,
}
