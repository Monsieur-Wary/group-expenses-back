#[actix_rt::main]
async fn main() -> anyhow::Result<()> {
    env_logger::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let configuration = group_expenses::configuration::Settings::new()?;
    let db_pool = sqlx::PgPool::connect(&configuration.database().connection_string())
        .await
        .expect("Failed to connect to Postgres.");

    let address = format!("0.0.0.0:{}", &configuration.application_port());
    let listener = std::net::TcpListener::bind(address)?;
    group_expenses::startup::run(listener, configuration, db_pool)?.await?;

    Ok(())
}
