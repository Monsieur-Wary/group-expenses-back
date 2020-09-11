use crate::{configuration, routes};
use actix_web::{dev::Server, http, middleware, web, App, HttpServer};
use std::sync;

pub fn run(
    listener: std::net::TcpListener,
    config: configuration::Settings,
    db_pool: sqlx::PgPool,
) -> std::result::Result<Server, std::io::Error> {
    let config = sync::Arc::new(config);
    let schema = sync::Arc::new(routes::create_schema());
    let db_pool = web::Data::new(db_pool);

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
            .app_data(web::Data::clone(&db_pool))
            .route("/health_check", web::get().to(routes::health_check))
            .service(
                web::resource("/graphql")
                    .route(web::post().to(routes::graphql))
                    // FIXME: This route shouldn't be exposed on production
                    .route(web::get().to(routes::graphiql)),
            )
    })
    .listen(listener)?
    .run();

    Ok(server)
}
