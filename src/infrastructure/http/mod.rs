mod graphql;
mod ops;

use crate::infrastructure::{config, graphql as gql, repositories};
use actix_web::{dev::Server, http, middleware, web, App, HttpServer};

pub fn run(
    listener: std::net::TcpListener,
    config: config::Settings,
    db_pool: repositories::PostgresPool,
) -> std::result::Result<Server, std::io::Error> {
    let config = web::Data::new(config);
    let db_pool = web::Data::new(db_pool);
    let schema = web::Data::new(gql::create_schema());

    let server = HttpServer::new(move || {
        App::new()
            .app_data(db_pool.clone())
            .app_data(config.clone())
            .app_data(schema.clone())
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .wrap(
                actix_cors::Cors::default()
                    .allowed_methods(vec!["GET", "POST"])
                    .allowed_headers(vec![
                        http::header::AUTHORIZATION,
                        http::header::ACCEPT,
                        http::header::CONTENT_TYPE,
                        http::header::ORIGIN,
                    ])
                    .max_age(3600)
                    .supports_credentials()
                    .allow_any_origin(),
            )
            .wrap(middleware::DefaultHeaders::default())
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
