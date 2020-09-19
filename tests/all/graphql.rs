use crate::helpers;
use serde_json::json;

#[actix_rt::test]
async fn graphql_api_should_work() {
    /* --- SignUp --- */
    // Arrange
    let app = helpers::spawn_app();
    let client = reqwest::Client::new();

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
                            id
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
    let signup_token = data.signup.token;
    assert!(!signup_token.is_empty());
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
                            id
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
    let login_token = data.login.token;

    assert_eq!(signup_token, login_token);
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
