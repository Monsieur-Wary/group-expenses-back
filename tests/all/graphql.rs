use crate::helpers;
use serde_json::json;

#[actix_rt::test]
async fn graphql_api_should_work() {
    // Arrange
    let app = helpers::spawn_app().await;
    let client = reqwest::Client::new();

    let email = "hi@hi.hi";
    let query = r#"
    mutation IT_SIGNUP($email: String!) {
        signup(email: $email, password: "hiiii!") {
            email
        }
    }
    "#;
    let body = json!({
        "query": query,
        "variables": {
            "email": email
        }
    });

    // Act
    let res = client
        .post(&format!("{}/graphql", &app.address))
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request.")
        .json::<GraphQLResponse>()
        .await
        .expect("Failed to convert response to json.");

    // Assert
    assert_eq!(None, res.errors);
    assert_eq!(email, res.data.signup.email);
}

#[derive(serde::Deserialize)]
struct GraphQLResponse {
    data: Signup,
    errors: Option<serde_json::Value>,
}

#[derive(serde::Deserialize)]
struct Signup {
    signup: CreatedUser,
}
#[derive(serde::Deserialize)]
struct CreatedUser {
    email: String,
}
