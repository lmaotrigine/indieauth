use color_eyre::eyre::{eyre, Result};
use diesel::prelude::*;
use diesel_migrations::{EmbeddedMigrations, HarnessWithOutput, MigrationHarness};
use indieauth::APPLICATION_NAME;
use tracing::info;
const EMBEDDED_MIGRATIONS: EmbeddedMigrations =
    diesel_migrations::embed_migrations!("./migrations");

pub fn establish_connection() -> Result<SqliteConnection> {
    let database_url = std::env::var("DATABASE_URL").unwrap_or("./file.db".to_string());
    SqliteConnection::establish(&database_url)
        .map_err(|why| eyre!("can't connect to {database_url}: {why}"))
}

fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();
    info!("{APPLICATION_NAME} migrator starting up");
    info!("running migrations");
    let mut connection = establish_connection()?;
    let mut harness = HarnessWithOutput::write_to_stdout(&mut connection);
    harness
        .run_pending_migrations(EMBEDDED_MIGRATIONS)
        .map_err(|why| eyre!("can't run migrations: {why}"))?;
    info!("migrations complete");
    Ok(())
}
