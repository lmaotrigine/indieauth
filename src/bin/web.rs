use color_eyre::eyre::Result;
use indieauth::{
    api, gitlab, oauth::OAuth2, paseto, rocket_trace::RequestId, wellknown, GitLab, MainDatabase,
    APPLICATION_NAME,
};
use tracing::info;

#[rocket::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();
    info!("{APPLICATION_NAME} starting up");
    let allowed_origins = rocket_cors::AllowedOrigins::some_exact(&[
        "https://5ht2.me",
        "https://api.5ht2.me",
        "http://localhost:7777",
        "http://localhost:7778",
    ]);
    let cors = rocket_cors::CorsOptions {
        allowed_origins,
        allowed_methods: vec![rocket::http::Method::Get, rocket::http::Method::Post]
            .into_iter()
            .map(From::from)
            .collect(),
        allowed_headers: rocket_cors::AllowedHeaders::some(&["Authorization", "Accept"]),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()?;
    rocket::build()
        .attach(cors)
        .attach(indieauth::rocket_trace::static_files())
        .attach(indieauth::frontend::fairing())
        .attach(MainDatabase::fairing())
        .attach(RequestId {})
        .attach(paseto::ed25519_keypair())
        .attach(OAuth2::<GitLab>::fairing("gitlab"))
        .mount(
            "/login/gitlab",
            rocket::routes![gitlab::callback, gitlab::login],
        )
        .mount(
            "/",
            rocket::routes![wellknown::botinfo, wellknown::robots, wellknown::security],
        )
        .mount(
            "/api",
            rocket::routes![
                api::indieauth::auth,
                api::indieauth::authorized,
                api::indieauth::send_code,
                api::token::info,
                api::token::mint,
            ],
        )
        .ignite()
        .await?
        .launch()
        .await?;
    Ok(())
}
