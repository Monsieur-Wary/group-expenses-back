#[macro_use]
extern crate diesel_migrations;

use group_expenses::infrastructure::repositories;

embed_migrations!();

#[actix_rt::main]
async fn main() -> anyhow::Result<()> {
    env_logger::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let configuration = group_expenses::configuration::Settings::new()?;
    let db_pool = repositories::get_pool(&configuration).expect("Failed to connect to Postgres.");

    // Setup the database
    embedded_migrations::run_with_output(&db_pool.get()?, &mut std::io::stdout())?;

    let address = format!("0.0.0.0:{}", &configuration.application_port());
    let listener = std::net::TcpListener::bind(address)?;
    group_expenses::startup::run(listener, configuration, db_pool)?.await?;

    Ok(())
}
