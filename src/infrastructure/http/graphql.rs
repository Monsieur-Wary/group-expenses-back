use crate::infrastructure::{config, graphql, repositories};
use actix_web::{web, Error, HttpResponse, Result};
// use futures_util::future::{Ready, TryFutureExt};
use juniper::http;
use std::sync;

pub async fn handler(
    db_pool: web::Data<repositories::PostgresPool>,
    schema: web::Data<sync::Arc<graphql::Schema>>,
    data: web::Json<http::GraphQLRequest>,
    config: web::Data<config::Settings>,
) -> Result<HttpResponse, Error> {
    let ctx = graphql::Context::new(db_pool.get_ref().to_owned(), config.get_ref().clone());
    let res = web::block(move || {
        let res = data.execute(&schema, &ctx);
        Ok::<_, serde_json::error::Error>(serde_json::to_string(&res)?)
    })
    .await
    .map_err(Error::from)?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(res))
}

pub async fn graphiql(config: web::Data<config::Settings>) -> HttpResponse {
    let html = http::graphiql::graphiql_source(&format!("{}/graphql", config.base_url()));
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

// struct GraphQLAuthentication;
// impl FromRequest for GraphQLAuthentication {
//     type Error = Error;
//     type Future = Ready<Result<Self>>;
//     type Config = ();

//     fn from_request(req: &HttpRequest, payload: &mut web::Payload) -> Self::Future {
//             web::Json::<http::GraphQLRequest>::from_request(req, &mut payload.0)
//                 .map_ok(|r| {
//                     let req = r.into_inner();
//                 });
//     }
// }
