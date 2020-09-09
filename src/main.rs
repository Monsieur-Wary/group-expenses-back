#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    env_logger::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let configuration =
        group_expenses::configuration::Settings::new().expect("Failed to read configuration.");
    let address = format!("127.0.0.1:{}", &configuration.application_port());
    let listener = std::net::TcpListener::bind(address)?;
    group_expenses::startup::run(listener, configuration)?.await
}
