/// Spin up an instance of our application
/// and returns its address (i.e. http://localhost:XXXX)
pub async fn spawn_app() -> TestApp {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port.");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let server = group_expenses::startup::run(listener).expect("Failed to bind address.");
    tokio::spawn(server);

    TestApp { address }
}

pub struct TestApp {
    pub address: String,
}
