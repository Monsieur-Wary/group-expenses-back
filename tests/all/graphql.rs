use crate::helpers;
use serde_json::json;

#[actix_rt::test]
async fn graphql_api_should_work() {
    let app = helpers::spawn_app();
    let client = reqwest::Client::new();

    /* --- Signup --- */
    // Arrange
    let email = format!("{}@htest.com", uuid::Uuid::new_v4());
    let pwd = String::from("hihihihi");
    let body = json!({
        "query": r#"
            mutation IT_SIGNUP($input: SignupInput!) {
                signup(input: $input)
            }
        "#,
        "variables": {
            "input": {
                "email": email,
                "password": pwd
            }
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
    assert!(res.errors.is_none());
    let data = res.data.unwrap();
    assert!(!data.signup.is_empty());

    /* --- Login --- */
    // Arrange
    let body = json!({
        "query": r#"
            query IT_LOGIN($email: String!, $password: String!) {
                login(email: $email, password: $password)
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
    assert!(res.errors.is_none());
    let data = res.data.unwrap();
    assert!(!data.login.is_empty());

    let bearer = format!("Bearer {}", data.login);

    /* --- viewer --- */
    // Arrange
    let viewer_body = json!({
        "query": r#"
            query IT_VIEWER {
                viewer {
                    dashboard {
                        persons {
                            id
                            name
                            resources
                        }
                        expenses {
                            id
                            name
                            amount
                        }
                    }
                }
            }
        "#
    });

    // Act
    let res = client
        .post(&format!("{}/graphql", app.address))
        .header(reqwest::header::AUTHORIZATION, &bearer)
        .json(&viewer_body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(200, res.status());

    let res = res
        .json::<GraphQLResponse<Viewer>>()
        .await
        .expect("Failed to convert response to json");

    // Assert
    assert!(res.errors.is_none());
    let data = res.data.unwrap();
    assert!(data.viewer.dashboard.persons.is_empty());
    assert!(data.viewer.dashboard.expenses.is_empty());

    /* --- addPerson --- */
    // Arrange
    let body = json!({
        "query": r#"
            mutation IT_ADD_PERSON($input: AddPersonInput!) {
                addPerson(input: $input)
            }
        "#,
        "variables": {
            "input": {
                "name": "Mary",
                "resources": 0
            }
        }
    });

    // Act
    let res = client
        .post(&format!("{}/graphql", app.address))
        .header(reqwest::header::AUTHORIZATION, &bearer)
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(200, res.status());

    let res = res
        .json::<GraphQLResponse<AddPerson>>()
        .await
        .expect("Failed to convert response to json");

    // Assert
    assert!(res.errors.is_none());

    /* --- Shouldn't be able to create duplicate person --- */
    // Act
    let res = client
        .post(&format!("{}/graphql", app.address))
        .header(reqwest::header::AUTHORIZATION, &bearer)
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(200, res.status());

    let res = res
        .json::<GraphQLResponse<AddPerson>>()
        .await
        .expect("Failed to convert response to json");

    // Assert
    assert!(res.errors.is_some());

    /* --- Check mutation results --- */
    // Act
    let res = client
        .post(&format!("{}/graphql", app.address))
        .header(reqwest::header::AUTHORIZATION, &bearer)
        .json(&viewer_body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(200, res.status());

    let res = res
        .json::<GraphQLResponse<Viewer>>()
        .await
        .expect("Failed to convert response to json");

    // Assert
    assert!(res.errors.is_none());
    let data = res.data.unwrap();
    assert!(!data.viewer.dashboard.persons.is_empty());

    let person_id = data.viewer.dashboard.persons[0].id;

    /* --- updatePerson --- */
    // Arrange
    let new_resources = 10;
    let body = json!({
        "query": r#"
            mutation IT_UPDATE_PERSON($input: UpdatePersonInput!) {
                updatePerson(input: $input)
            }
        "#,
        "variables": {
            "input": {
                "personId": person_id,
                "resources": new_resources
            }
        }
    });

    // Act
    let res = client
        .post(&format!("{}/graphql", app.address))
        .header(reqwest::header::AUTHORIZATION, &bearer)
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(200, res.status());

    let res = res
        .json::<GraphQLResponse<UpdatePerson>>()
        .await
        .expect("Failed to convert response to json");

    // Assert
    assert!(res.errors.is_none(), format!("{:?}", res.errors));

    /* --- addExpense --- */
    // Arrange
    let body = json!({
        "query": r#"
            mutation IT_ADD_EXPENSE($input: AddExpenseInput!) {
                addExpense(input: $input)
            }
        "#,
        "variables": {
            "input": {
                "personId": person_id,
                "name": "Burger King",
                "amount": 20
            }
        }
    });

    // Act
    let res = client
        .post(&format!("{}/graphql", app.address))
        .header(reqwest::header::AUTHORIZATION, &bearer)
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(200, res.status());

    let res = res
        .json::<GraphQLResponse<AddExpense>>()
        .await
        .expect("Failed to convert response to json");

    // Assert
    assert!(res.errors.is_none());

    /* --- Check mutation results --- */
    // Act
    let res = client
        .post(&format!("{}/graphql", app.address))
        .header(reqwest::header::AUTHORIZATION, &bearer)
        .json(&viewer_body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(200, res.status());

    let res = res
        .json::<GraphQLResponse<Viewer>>()
        .await
        .expect("Failed to convert response to json");

    // Assert
    assert!(res.errors.is_none());
    let data = res.data.unwrap();
    assert_eq!(data.viewer.dashboard.persons[0].resources, new_resources);
    assert!(!data.viewer.dashboard.expenses.is_empty());

    /* --- removePerson --- */
    // Arrange
    let body = json!({
        "query": r#"
            mutation IT_REMOVE_PERSON($input: RemovePersonInput!) {
                removePerson(input: $input)
            }
        "#,
        "variables": {
            "input": {
                "personId": person_id
            }
        }
    });

    // Act
    let res = client
        .post(&format!("{}/graphql", app.address))
        .header(reqwest::header::AUTHORIZATION, &bearer)
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(200, res.status());

    let res = res
        .json::<GraphQLResponse<RemovePerson>>()
        .await
        .expect("Failed to convert response to json");

    // Assert
    assert!(res.errors.is_none());

    /* --- Check mutation results --- */
    // Act
    let res = client
        .post(&format!("{}/graphql", app.address))
        .header(reqwest::header::AUTHORIZATION, &bearer)
        .json(&viewer_body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(200, res.status());

    let res = res
        .json::<GraphQLResponse<Viewer>>()
        .await
        .expect("Failed to convert response to json");

    // Assert
    assert!(res.errors.is_none());
    let data = res.data.unwrap();
    assert!(data.viewer.dashboard.persons.is_empty());
    assert!(data.viewer.dashboard.expenses.is_empty());
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
    signup: String,
}

#[derive(serde::Deserialize)]
struct Login {
    login: String,
}

#[derive(serde::Deserialize)]
struct Viewer {
    viewer: User,
}

#[derive(serde::Deserialize)]
struct User {
    dashboard: Dashboard,
}
#[derive(serde::Deserialize)]
struct Dashboard {
    persons: Vec<Person>,
    expenses: Vec<Expense>,
}

#[allow(dead_code)]
#[derive(serde::Deserialize)]
struct Person {
    id: uuid::Uuid,
    name: String,
    resources: i32,
}

#[allow(dead_code)]
#[derive(serde::Deserialize)]
struct Expense {
    id: uuid::Uuid,
    name: String,
    amount: i32,
}

#[allow(dead_code)]
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct AddPerson {
    add_person: bool,
}

#[allow(dead_code)]
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdatePerson {
    update_person: bool,
}

#[allow(dead_code)]
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct AddExpense {
    add_expense: bool,
}

#[allow(dead_code)]
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct RemovePerson {
    remove_person: bool,
}
