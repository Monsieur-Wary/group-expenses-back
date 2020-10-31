use crate::infrastructure::{config, graphql, repositories, security};
use actix_web::{dev, error, web, Error, FromRequest, HttpRequest, HttpResponse, Result};
use futures_util::future::{FutureExt, LocalBoxFuture};
use graphql_parser::query;
use juniper::{http, DefaultScalarValue, InputValue, ScalarValue};
use serde::{Deserialize, Serialize};

pub async fn handler(
    db_pool: web::Data<repositories::PostgresPool>,
    schema: web::Data<graphql::Schema>,
    req: GraphQLAuthentication,
) -> Result<HttpResponse> {
    let config = req.config();
    let viewer = req.viewer();
    let ctx = graphql::Context {
        db_pool: db_pool.get_ref().to_owned(),
        config,
        viewer,
    };

    let res = web::block(move || {
        let res = req.graphql().execute(&schema, &ctx);
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

pub struct GraphQLAuthentication {
    gql: http::GraphQLRequest,
    config: config::Settings,
    viewer: security::Viewer,
}

impl GraphQLAuthentication {
    fn new(http: HttpRequest, gql: GraphQLRequest) -> Result<Self> {
        let config = http
            .app_data::<web::Data<config::Settings>>()
            .expect("Couldn't extract settings")
            .as_ref()
            .clone();

        graphql_parser::parse_query::<&str>(gql.query.as_str())
            .map_err(error::ErrorBadRequest)
            .map(|ast| extract_graphql_operation(ast, gql.operation_name.clone()))
            .and_then(|op| {
                let gql = http::GraphQLRequest::new(gql.query, gql.operation_name, gql.variables);

                if GRAPHQL_OPERATIONS_AUTH_EXCEPTION.contains(&op.as_str()) {
                    log::debug!("GraphQL requet - exception for {}", op);
                    return Ok(GraphQLAuthentication {
                        gql,
                        config,
                        viewer: security::Viewer::default(),
                    });
                }

                extract_and_check_token(&http)
                    .map(|viewer| {
                        log::debug!("GraphQL request - user authorized for {}", op);

                        GraphQLAuthentication {
                            gql,
                            config,
                            viewer,
                        }
                    })
                    .map_err(|e| {
                        log::debug!("GraphQL request - user unauthorized for {}", op);
                        e
                    })
            })
    }

    pub fn graphql(&self) -> &http::GraphQLRequest {
        &self.gql
    }

    pub fn config(&self) -> config::Settings {
        self.config.clone()
    }

    pub fn viewer(&self) -> security::Viewer {
        self.viewer.clone()
    }
}

impl FromRequest for GraphQLAuthentication {
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self>>;
    type Config = ();

    fn from_request(req: &HttpRequest, payload: &mut dev::Payload) -> Self::Future {
        let req = req.clone();
        web::Json::<GraphQLRequest>::from_request(&req, payload)
            .map(|r| r.and_then(|j| GraphQLAuthentication::new(req, j.into_inner())))
            .boxed_local()
    }
}

const GRAPHQL_OPERATIONS_AUTH_EXCEPTION: [&str; 3] = ["signup", "login", "__schema"];

fn extract_graphql_operation<'a>(
    ast: query::Document<'a, &'a str>,
    operation_name: Option<String>,
) -> String {
    ast.definitions
        .into_iter()
        .filter_map(|d| match d {
            query::Definition::Operation(o) => Some(o),
            _ => None,
        })
        .filter_map(|o| match o {
            query::OperationDefinition::Query(q) => {
                if operation_name.is_none() || q.name.as_deref() == operation_name.as_deref() {
                    Some(q.selection_set.items)
                } else {
                    None
                }
            }
            query::OperationDefinition::Mutation(m) => {
                if operation_name.is_none() || m.name.as_deref() == operation_name.as_deref() {
                    Some(m.selection_set.items)
                } else {
                    None
                }
            }
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

fn extract_and_check_token(req: &HttpRequest) -> Result<security::Viewer> {
    let secret_key = req
        .app_data::<web::Data<config::Settings>>()
        .expect("Couldn't extract settings")
        .as_ref()
        .security()
        .secret_key();

    let extracted = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| {
            let re = regex::Regex::new(r"^Bearer (.+)$").unwrap();
            re.captures(s).and_then(|c| c.get(1)).map(|m| m.as_str())
        });

    match extracted {
        None => Err(error::ErrorUnauthorized("Unauthorized")),
        Some(t) => security::verify_token(t, secret_key).map_err(error::ErrorInternalServerError),
    }
}

// Copy of juniper's
#[derive(Deserialize, Debug, Clone)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_extract_the_graphql_operation() {
        let ast =
            graphql_parser::parse_query::<&str>("query MyQuery { viewer { token } }").unwrap();
        assert_eq!("viewer", extract_graphql_operation(ast, None));

        let ast =
            graphql_parser::parse_query::<&str>("mutation MyMutation { addPerson { person } }")
                .unwrap();
        assert_eq!(
            "addPerson",
            extract_graphql_operation(ast, Some("MyMutation".to_string()))
        );

        let ast = graphql_parser::parse_query::<&str>(
            r#"
            query MyQuery {
                viewer {
                    token
                }
            }
            mutation MyMutation {
                addPerson {
                    person
                }
            }
            "#,
        )
        .unwrap();
        assert_eq!(
            "addPerson",
            extract_graphql_operation(ast, Some("MyMutation".to_string()))
        );
    }
}
