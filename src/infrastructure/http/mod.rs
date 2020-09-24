mod graphql;
mod ops;

use crate::infrastructure::{config, graphql as gql, repositories};
use actix_web::{dev::Server, http, middleware, web, App, HttpServer};
use std::sync;

pub fn run(
    listener: std::net::TcpListener,
    config: config::Settings,
    db_pool: repositories::PostgresPool,
) -> std::result::Result<Server, std::io::Error> {
    let schema = sync::Arc::new(gql::create_schema());

    let server = HttpServer::new(move || {
        App::new()
            .data(db_pool.clone())
            .app_data(config.clone())
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .wrap(
                actix_cors::Cors::new()
                    .allowed_methods(vec!["GET", "POST"])
                    .allowed_headers(vec![
                        http::header::AUTHORIZATION,
                        http::header::ACCEPT,
                        http::header::CONTENT_TYPE,
                        http::header::ORIGIN,
                    ])
                    .max_age(3600)
                    .supports_credentials()
                    .finish(),
            )
            .wrap(middleware::DefaultHeaders::default())
            .data(sync::Arc::clone(&schema))
            .route("/health_check", web::get().to(ops::health_check))
            .service(
                web::resource("/graphql")
                    .route(web::post().to(graphql::handler))
                    // FIXME: This route shouldn't be exposed on production
                    .route(web::get().to(graphql::graphiql)),
            )
    })
    .listen(listener)?
    .run();

    Ok(server)
}
