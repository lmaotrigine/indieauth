use super::{Error, Result};
use crate::{models, oauth::pkce, paseto::Token, schema, MainDatabase};
use askama::Template;
use diesel::prelude::*;
use rocket::{
    response::Redirect,
    serde::{json::Json, Serialize},
};

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct Me {
    pub me: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

#[allow(clippy::too_many_arguments)] // :/
#[rocket::get("/auth?<me>&<client_id>&<redirect_uri>&<state>&<response_type>&<code_challenge>&<code_challenge_method>")]
#[tracing::instrument(skip(db), err)]
pub async fn auth(
    db: MainDatabase,
    me: String,
    client_id: String,
    redirect_uri: String,
    state: String,
    response_type: String,
    code_challenge: String,
    code_challenge_method: String,
) -> Result<Authz> {
    let _ = db;
    match response_type.as_str() {
        "code" | "id" => {}
        _ => return Err(Error::WrongIndieAuthResponseType(response_type)),
    }
    if me.as_str() != "https://5ht2.me" {
        return Err(Error::NotFound);
    }
    if code_challenge_method.as_str() != "S256" {
        return Err(Error::WrongIndieAuthCodeChallengeMethod(
            code_challenge_method,
        ));
    }
    let _cid = client_id.clone();
    let code = rusty_ulid::generate_ulid_string();
    let _c = code.clone();
    db.run(move |c| {
        use schema::indieauth_codes::dsl::indieauth_codes;
        diesel::insert_into(indieauth_codes)
            .values(&models::IndieauthCode {
                code,
                client_id,
                redirect_uri,
                state,
                response_type,
                code_challenge,
                authorized: false,
            })
            .execute(c)
            .map_err(Error::Database)
    })
    .await?;
    Ok(Authz {
        client_id: _cid,
        code: _c,
        me,
    })
}

#[rocket::get("/auth/authorized?<code>")]
#[tracing::instrument(skip(db), err)]
pub async fn authorized(token: Token, db: MainDatabase, code: String) -> Result<Redirect> {
    let _c = code.clone();
    let iac = db
        .run(move |c| {
            use schema::indieauth_codes::dsl::indieauth_codes;
            let iac: Result<models::IndieauthCode> = indieauth_codes
                .find(&code)
                .get_result(c)
                .map_err(Error::Database);
            let iac = match iac {
                Ok(iac) => iac,
                Err(e) => return Err(e),
            };
            match diesel::update(indieauth_codes.find(&iac.code))
                .set(&models::UpdateIndieauthCodeAuthorized { authorized: true })
                .execute(c)
                .map_err(Error::Database)
            {
                Err(e) => Err(e),
                Ok(_) => Ok(iac),
            }
        })
        .await?;
    if iac.code != _c {
        return Err(Error::NotFound);
    }
    let u =
        reqwest::Url::parse_with_params(&iac.redirect_uri, &[("code", &_c), ("state", &iac.state)])
            .map_err(|_| Error::NotFound)?;
    Ok(Redirect::to(u.to_string()))
}

#[derive(Template)]
#[template(path = "authz.html")]
pub struct Authz {
    client_id: String,
    me: String,
    code: String,
}

#[rocket::post("/auth?<code>&<redirect_uri>&<client_id>&<code_verifier>", rank = 2)]
#[tracing::instrument(skip(db, code), err)]
pub async fn send_code(
    db: MainDatabase,
    client_id: String,
    redirect_uri: String,
    code: String,
    code_verifier: String,
) -> Result<Json<Me>> {
    db.run(move |c| {
        use schema::indieauth_codes::dsl;
        let _iac: Result<models::IndieauthCode> = dsl::indieauth_codes
            .find(&code)
            .get_result(c)
            .map_err(Error::Database);
        let iac = match _iac {
            Err(e) => return Err(e),
            Ok(_c) => _c,
        };
        if !iac.authorized {
            return Err(Error::NotFound);
        }
        if !pkce::verify(&code_verifier, &iac.code_challenge) {
            return Err(Error::InvalidCodeVerifier(code_verifier));
        }
        diesel::delete(dsl::indieauth_codes.filter(dsl::code.eq(code)))
            .execute(c)
            .map_err(Error::Database)
            .map(|_| ())
    })
    .await?;
    Ok(Json(Me {
        me: "https://5ht2.me".to_string(),
        access_token: None,
        scope: None,
    }))
}
