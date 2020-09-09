use super::schema;
use crate::configuration;
use actix_web::{web, Error, HttpResponse};
use juniper::http;
use std::sync;

pub fn create_schema() -> schema::Schema {
    schema::Schema::new(schema::Query, schema::Mutation)
}

pub async fn graphql(
    schema: web::Data<sync::Arc<schema::Schema>>,
    data: web::Json<http::GraphQLRequest>,
) -> Result<HttpResponse, Error> {
    let ctx = schema::Context;
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

pub async fn graphiql(config: web::Data<sync::Arc<configuration::Settings>>) -> HttpResponse {
    let html = http::graphiql::graphiql_source(&format!("{}/graphql", &config.base_url()));
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}
