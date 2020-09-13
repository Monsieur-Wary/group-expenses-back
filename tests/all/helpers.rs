use crate::embedded_migrations;
use group_expenses::{configuration, infrastructure::repositories};
use std::env;

lazy_static! {
    static ref APP: TestApp = {
        let config = configuration::Settings::new().expect("Failed to read config.");
        let db_pool = configure_database(&config);
        // Setup the database
        embedded_migrations::run_with_output(
            &db_pool.get().expect("Failed to get a connection."),
            &mut std::io::stdout(),
        )
        .expect("Failed to run the migration.");
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port.");
        let port = listener.local_addr().unwrap().port();
        let address = format!("http://127.0.0.1:{}", port);

        let server =
            group_expenses::startup::run(listener, config, db_pool).expect("Failed to bind address.");
        tokio::spawn(server);

        TestApp { address }
    };
}

pub fn initialize() {
    env::set_var("HASH_SALT", "randomsalt");
    env::set_var(
        "DATABASE_URL",
        "postgres://postgres:password@localhost:5432/group-expenses",
    );
}

/// Spin up an instance of our application and returns its address (i.e. http://localhost:XXXX)
pub fn spawn_app() -> &'static TestApp {
    initialize();
    &*APP
}

fn configure_database(db_config: &configuration::Settings) -> repositories::PostgresPool {
    repositories::get_pool(db_config).expect("Failed to connect to Postgres.")
}

pub struct TestApp {
    pub address: String,
}
