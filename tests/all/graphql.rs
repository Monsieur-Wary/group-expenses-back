use crate::helpers;
use serde_json::json;

#[actix_rt::test]
async fn graphql_api_should_work() {
    let app = helpers::spawn_app();
    let client = reqwest::Client::new();

    /* --- SignUp --- */
    // Arrange
    let email = format!("{}@htest.com", uuid::Uuid::new_v4());
    let pwd = String::from("hihihihi");
    let body = json!({
        "query": r#"
            mutation IT_SIGNUP($email: String!, $password: String!) {
                signup(email: $email, password: $password) {
                    token
                    user {
                        email
                        dashboard {
                            expenses {
                                id
                            }
                            persons {
                                id
                            }
                        }
                    }
                }
            }
        "#,
        "variables": {
            "email": email,
            "password": pwd
        }
    });

    // Act
    let res = client
        .post(&format!("{}/graphql", app.address))
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(200, res.status());

    let res = res
        .json::<GraphQLResponse<Signup>>()
        .await
        .expect("Failed to convert response to json");

    // Assert
    assert_eq!(None, res.errors);
    let data = res.data.unwrap();
    assert!(!data.signup.token.is_empty());
    assert_eq!(email, data.signup.user.email);

    /* --- Login --- */
    // Arrange
    let body = json!({
        "query": r#"
            mutation IT_LOGIN($email: String!, $password: String!) {
                login(email: $email, password: $password) {
                    token
                    user {
                        email
                        dashboard {
                            expenses {
                                id
                            }
                            persons {
                                id
                            }
                        }
                    }
                }
            }
        "#,
        "variables": {
            "email": email,
            "password": pwd
        }
    });

    // Act
    let res = client
        .post(&format!("{}/graphql", app.address))
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(200, res.status());

    let res = res
        .json::<GraphQLResponse<Login>>()
        .await
        .expect("Failed to convert response to json");

    // Assert
    assert_eq!(None, res.errors);
    let data = res.data.unwrap();
    assert!(!data.login.token.is_empty());
}

#[actix_rt::test]
async fn non_auth_operations_should_be_protected() {
    let app = helpers::spawn_app();
    let client = reqwest::Client::new();

    // Arrange
    let body = json!({
        "query": r#"
            query NON_AUTH {
                nonAuth {
                    myfield
                }
            }
        "#
    });

    // Act
    let res = client
        .post(&format!("{}/graphql", app.address))
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(401, res.status());
}

#[derive(serde::Deserialize)]
struct GraphQLResponse<T> {
    data: Option<T>,
    errors: Option<Vec<serde_json::Value>>,
}

#[derive(serde::Deserialize)]
struct Signup {
    signup: AuthPayload,
}

#[derive(serde::Deserialize)]
struct Login {
    login: AuthPayload,
}
#[derive(serde::Deserialize)]
struct AuthPayload {
    token: String,
    user: User,
}
#[derive(serde::Deserialize)]
struct User {
    email: String,
}
