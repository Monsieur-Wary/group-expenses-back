use crate::helpers;
use serde_json::json;

struct GraphQLClient {
    url: String,
    client: reqwest::Client,
}

enum GraphQLRequestInput<'a> {
    WithToken {
        body: &'a serde_json::Value,
        token: &'a str,
    },
    WithoutToken {
        body: &'a serde_json::Value,
    },
}

impl GraphQLClient {
    fn new(url: String) -> Self {
        let client = reqwest::Client::new();
        GraphQLClient { client, url }
    }

    async fn send<T>(&self, input: &GraphQLRequestInput<'_>) -> reqwest::Result<GraphQLResponse<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        let req = self.client.post(&self.url);
        let req = match input {
            GraphQLRequestInput::WithoutToken { body } => req.json(body),
            GraphQLRequestInput::WithToken { body, token } => req
                .header(reqwest::header::AUTHORIZATION, format!("Bearer {}", token))
                .json(body),
        };
        let res = req.send().await?;

        assert_eq!(200, res.status());

        res.json::<GraphQLResponse<T>>().await
    }
}

#[actix_rt::test]
async fn graphql_api_should_work() {
    let app = helpers::spawn_app();
    let client = GraphQLClient::new(format!("{}/graphql", app.address));

    /* --- Signup --- */
    // Arrange
    let email = format!("{}@htest.com", helpers::rand_string());
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
    let input = GraphQLRequestInput::WithoutToken { body: &body };
    let res = client
        .send::<Signup>(&input)
        .await
        .expect("Failed to convert response to json");

    // Assert
    assert!(res.errors.is_none(), format!("{:?}", res.errors));
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
    let input = GraphQLRequestInput::WithoutToken { body: &body };
    let res = client
        .send::<Login>(&input)
        .await
        .expect("Failed to convert response to json");

    // Assert
    assert!(res.errors.is_none(), format!("{:?}", res.errors));
    let data = res.data.unwrap();
    assert!(!data.login.is_empty());

    let token = data.login;

    /* --- viewer --- */
    // Arrange
    let body = json!({
        "query": r#"
            query IT_VIEWER {
                viewer {
                    groups {
                        id
                        name
                        persons {
                            id
                            name
                        }
                        expenses {
                            id
                            name
                        }
                    }
                }
            }
        "#
    });

    // Act
    let viewer_input = GraphQLRequestInput::WithToken {
        body: &body,
        token: &token,
    };
    let res = client
        .send::<Viewer>(&viewer_input)
        .await
        .expect("Failed to convert response to json");

    // Assert
    assert!(res.errors.is_none(), format!("{:?}", res.errors));
    let data = res.data.unwrap();
    assert!(data.viewer.groups.is_empty());

    /* --- addGroup --- */
    // Arrange
    let group_name = "Mary";
    let body = json!({
        "query": r#"
            mutation IT_ADD_GROUP($input: AddGroupInput!) {
                addGroup(input: $input)
            }
        "#,
        "variables": {
            "input": {
                "name": group_name
            }
        }
    });

    // Act
    let input = GraphQLRequestInput::WithToken {
        body: &body,
        token: &token,
    };
    let res = client
        .send::<AddGroup>(&input)
        .await
        .expect("Failed to convert response to json");

    // Assert
    assert!(res.errors.is_none(), format!("{:?}", res.errors));

    /* --- Check Mutation result --- */
    let res = client
        .send::<Viewer>(&viewer_input)
        .await
        .expect("Failed to convert response to json");

    // Assert
    assert!(res.errors.is_none(), format!("{:?}", res.errors));
    let data = res.data.unwrap();
    assert!(data.viewer.groups.len() == 1);
    assert_eq!(group_name, data.viewer.groups[0].name);
    let group_id = data.viewer.groups[0].id;

    /* --- updateGroup --- */
    // Arrange
    let body = json!({
        "query": r#"
            mutation IT_UPDATE_GROUP($input: UpdateGroupInput!) {
                updateGroup(input: $input)
            }
        "#,
        "variables": {
            "input": {
                "personId": group_id,
                "name": "Maria"
            }
        }
    });

    // Act
    let input = GraphQLRequestInput::WithToken {
        body: &body,
        token: &token,
    };
    let res = client
        .send::<UpdateGroup>(&input)
        .await
        .expect("Failed to convert response to json");

    // Assert
    assert!(res.errors.is_none(), format!("{:?}", res.errors));

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
                "groupId": group_id,
                "name": "Mary",
                "resources": 0,
            }
        }
    });

    // Act
    let input = GraphQLRequestInput::WithToken {
        body: &body,
        token: &token,
    };
    let res = client
        .send::<AddPerson>(&input)
        .await
        .expect("Failed to convert response to json");

    // Assert
    assert!(res.errors.is_none(), format!("{:?}", res.errors));

    /* --- Shouldn't be able to create duplicate person --- */
    // Act
    let res = client
        .send::<AddPerson>(&input)
        .await
        .expect("Failed to convert response to json");

    // Assert
    assert!(res.errors.is_some());

    /* --- Check mutation results --- */
    // Arrange
    let body = json!({
        "query": r#"
            query IT_GROUP($id: String!) {
                group(id: $id) {
                    id
                    name
                    persons {
                        id
                        name
                        resources
                        expenses {
                            id
                            name
                            amount
                        }
                    }
                    expenses {
                        id
                        name
                        amount
                    }
                }
            }
        "#,
        "variables": {
            "id": group_id
        }
    });

    // Act
    let group_input = GraphQLRequestInput::WithToken {
        body: &body,
        token: &token,
    };
    let res = client
        .send::<GroupQuery>(&group_input)
        .await
        .expect("Failed to convert response to json");

    // Assert
    assert!(res.errors.is_none(), format!("{:?}", res.errors));
    let data = res.data.unwrap();
    assert!(!data.group.persons.is_empty());
    assert!(data.group.persons.len() == 1);

    let person_id = data.group.persons[0].id;

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
    let input = GraphQLRequestInput::WithToken {
        body: &body,
        token: &token,
    };
    let res = client
        .send::<UpdatePerson>(&input)
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
                "groupId": group_id,
                "personId": person_id,
                "name": "Burger King",
                "amount": 20
            }
        }
    });

    // Act
    let input = GraphQLRequestInput::WithToken {
        body: &body,
        token: &token,
    };
    let res = client
        .send::<AddExpense>(&input)
        .await
        .expect("Failed to convert response to json");

    // Assert
    assert!(res.errors.is_none(), format!("{:?}", res.errors));

    /* --- Check mutation results --- */
    let res = client
        .send::<GroupQuery>(&group_input)
        .await
        .expect("Failed to convert response to json");

    // Assert
    assert!(res.errors.is_none(), format!("{:?}", res.errors));
    let data = res.data.unwrap();
    assert_eq!(data.group.persons[0].resources, new_resources);
    assert!(!data.group.persons[0].expenses.is_empty());

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
    let input = GraphQLRequestInput::WithToken {
        body: &body,
        token: &token,
    };
    let res = client
        .send::<RemovePerson>(&input)
        .await
        .expect("Failed to convert response to json");

    // Assert
    assert!(res.errors.is_none(), format!("{:?}", res.errors));

    /* --- Check mutation results --- */
    // Act
    let res = client
        .send::<GroupQuery>(&group_input)
        .await
        .expect("Failed to convert response to json");

    // Assert
    assert!(res.errors.is_none(), format!("{:?}", res.errors));
    let data = res.data.unwrap();
    assert!(data.group.persons.is_empty());
    assert!(data.group.expenses.is_empty());

    /* --- removeGroup --- */
    // Arrange
    let body = json!({
        "query": r#"
            mutation IT_REMOVE_GROUP($input: RemoveGroupInput!) {
                removeGroup(input: $input)
            }
        "#,
        "variables": {
            "input": {
                "groupId": group_id
            }
        }
    });

    // Act
    let input = GraphQLRequestInput::WithToken {
        body: &body,
        token: &token,
    };
    let res = client
        .send::<RemoveGroup>(&input)
        .await
        .expect("Failed to convert response to json");

    // Assert
    assert!(res.errors.is_none(), format!("{:?}", res.errors));

    /* --- Check Mutation result --- */
    let res = client
        .send::<Viewer>(&viewer_input)
        .await
        .expect("Failed to convert response to json");

    // Assert
    assert!(res.errors.is_none(), format!("{:?}", res.errors));
    let data = res.data.unwrap();
    assert!(data.viewer.groups.is_empty());
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
struct GroupQuery {
    group: Group,
}

#[derive(serde::Deserialize)]
struct User {
    groups: Vec<Group>,
}

#[allow(dead_code)]
#[derive(serde::Deserialize)]
struct Group {
    id: uuid::Uuid,
    name: String,
    persons: Vec<Person>,
    expenses: Vec<Expense>,
}

#[allow(dead_code)]
#[derive(serde::Deserialize)]
struct Person {
    id: uuid::Uuid,
    name: String,
    resources: i32,
    expenses: Vec<Expense>,
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
struct AddGroup {
    add_group: bool,
}

#[allow(dead_code)]
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateGroup {
    update_group: bool,
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

#[allow(dead_code)]
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct RemoveGroup {
    remove_group: bool,
}
