use crate::infrastructure::{config, graphql, repositories};
use actix_web::{dev, error, web, Error, FromRequest, HttpRequest, HttpResponse, Result};
use futures_util::future::{FutureExt, LocalBoxFuture};
use graphql_parser::query;
use juniper::{http, DefaultScalarValue, InputValue, ScalarValue};
use serde::{Deserialize, Serialize};
use std::sync;

pub async fn handler(
    db_pool: web::Data<repositories::PostgresPool>,
    schema: web::Data<sync::Arc<graphql::Schema>>,
    req: GraphQLRequest,
    config: web::Data<config::Settings>,
) -> Result<HttpResponse> {
    let req: http::GraphQLRequest = req.into();
    let ctx = graphql::Context {
        db_pool: db_pool.get_ref().to_owned(),
        config: config.get_ref().clone(),
    };
    let res = web::block(move || {
        let res = req.execute(&schema, &ctx);
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

// Copy of juniper's
#[derive(Deserialize, Debug)]
pub struct GraphQLRequest<S = DefaultScalarValue>
where
    S: ScalarValue,
{
    query: String,
    #[serde(rename = "operationName")]
    operation_name: Option<String>,
    #[serde(bound(deserialize = "InputValue<S>: Deserialize<'de> + Serialize"))]
    variables: Option<InputValue<S>>,
}

impl From<GraphQLRequest> for http::GraphQLRequest {
    fn from(req: GraphQLRequest) -> Self {
        http::GraphQLRequest::new(req.query, req.operation_name, req.variables)
    }
}

impl FromRequest for GraphQLRequest {
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self>>;
    type Config = ();

    fn from_request(req: &HttpRequest, payload: &mut dev::Payload) -> Self::Future {
        web::Json::<GraphQLRequest>::from_request(req, payload)
            .map(|r| {
                log::debug!(" serialization result: {:?}", r);
                r.and_then(|j| {
                    let req = j.into_inner();
                    graphql_parser::parse_query::<&str>(req.query.as_str())
                        .map_err(|e| error::ErrorBadRequest(e))
                        .map(extract_graphql_operation)
                        .and_then(|op| {
                            if AUTH_EXCEPTION_GRAPHQL_OPERATIONS.contains(&op.as_str()) {
                                Ok(req)
                            } else {
                                Err(error::ErrorUnauthorized("Unauthorized"))
                            }
                        })
                })
            })
            .boxed_local()
    }
}

const AUTH_EXCEPTION_GRAPHQL_OPERATIONS: [&str; 2] = ["signup", "login"];

fn extract_graphql_operation<'a>(ast: query::Document<'a, &'a str>) -> String {
    ast.definitions
        .into_iter()
        .filter_map(|d| match d {
            query::Definition::Operation(o) => Some(o),
            _ => None,
        })
        .filter_map(|o| match o {
            query::OperationDefinition::Query(q) => Some(q.selection_set.items),
            query::OperationDefinition::Mutation(m) => Some(m.selection_set.items),
            _ => None,
        })
        .flat_map(|v| {
            v.into_iter().filter_map(|s| match s {
                query::Selection::Field(f) => Some(f.name),
                _ => None,
            })
        })
        .collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_extract_the_graphql_operation() {
        let ast = graphql_parser::parse_query::<&str>("query MyQuery { login { token } }").unwrap();
        assert_eq!("login", extract_graphql_operation(ast));

        let ast =
            graphql_parser::parse_query::<&str>("mutation MyMutation { addPerson { person } }")
                .unwrap();
        assert_eq!("addPerson", extract_graphql_operation(ast));
    }
}
