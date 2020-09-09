use crate::{configuration, routes};
use actix_web::{dev::Server, http, middleware, web, App, HttpServer};
use std::sync;

pub fn run(
    listener: std::net::TcpListener,
    config: configuration::Settings,
) -> std::result::Result<Server, std::io::Error> {
    env_logger::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let config = sync::Arc::new(config);
    let schema = sync::Arc::new(routes::create_schema());

    let server = HttpServer::new(move || {
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
            .data(sync::Arc::clone(&config))
            .data(sync::Arc::clone(&schema))
            .route("/health_check", web::get().to(routes::health_check))
            .service(
                web::resource("/graphql")
                    .route(web::post().to(routes::graphql))
                    .route(web::get().to(routes::graphiql)),
            )
    })
    .listen(listener)?
    .run();

    Ok(server)
}
