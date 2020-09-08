use crate::routes;
use actix_web::{dev::Server, http, middleware, web, App, HttpServer};

pub fn run(listener: std::net::TcpListener) -> std::result::Result<Server, std::io::Error> {
    env_logger::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let server = HttpServer::new(|| {
        App::new()
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .wrap(
                actix_cors::Cors::new()
                    .allowed_methods(vec!["GET", "POST"])
                    .allowed_headers(vec![
                        http::header::AUTHORIZATION,
                        http::header::ACCEPT,
                        http::header::CONTENT_TYPE,
                    ])
                    .max_age(3600)
                    .finish(),
            )
            .wrap(middleware::DefaultHeaders::default())
            .route("/health_check", web::get().to(routes::health_check))
    })
    .listen(listener)?
    .run();

    Ok(server)
}
