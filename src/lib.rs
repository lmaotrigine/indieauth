pub mod api;
pub mod frontend;
pub mod gitlab;
pub mod models;
pub mod oauth;
pub mod paseto;
pub mod rocket_trace;
pub mod schema;
pub mod wellknown;

use rocket_sync_db_pools::{
    database,
    diesel::{prelude::*, SqliteConnection},
};

pub const APPLICATION_NAME: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
    " +https://5ht2.me/.well-known/botinfo"
);

#[database("main_data")]
pub struct MainDatabase(SqliteConnection);

pub struct GitLab;

pub fn establish_connection() -> SqliteConnection {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {database_url}"))
}
