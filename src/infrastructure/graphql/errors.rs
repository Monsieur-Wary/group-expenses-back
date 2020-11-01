use juniper::graphql_value;

pub enum GraphQLError {
    InvalidPassword,
    InvalidEmailAddress,
    InvalidCredentials,
    InvalidName,
    InvalidResources,
    InvalidId,
    InvalidAmount,
    AlreadyUsedEmail,
    UserNotFound,
    GroupNotFound,
    PersonNotFound,
    NonUniqueName(String),
    InternalServerError(anyhow::Error),
}

impl juniper::IntoFieldError for GraphQLError {
    fn into_field_error(self) -> juniper::FieldError {
        match self {
            GraphQLError::InvalidPassword => juniper::FieldError::new(
                "The password is invalid!",
                graphql_value!({
                    "code": "INVALID_PASSWORD"
                }),
            ),
            GraphQLError::InvalidEmailAddress => juniper::FieldError::new(
                "The email address is invalid!",
                graphql_value!({
                    "code": "INVALID_EMAIL_ADDRESS"
                }),
            ),
            GraphQLError::InvalidCredentials => juniper::FieldError::new(
                "The credentials are invalid!",
                graphql_value!({
                    "code": "INVALID_CREDENTIALS"
                }),
            ),
            GraphQLError::InvalidName => juniper::FieldError::new(
                "The name is invalid!",
                graphql_value!({
                    "code": "INVALID_NAME"
                }),
            ),
            GraphQLError::InvalidResources => juniper::FieldError::new(
                "The resources are invalid!",
                graphql_value!({
                    "code": "INVALID_RESOURCES"
                }),
            ),
            GraphQLError::InvalidId => juniper::FieldError::new(
                "The id is invalid!",
                graphql_value!({
                    "code": "INVALID_ID"
                }),
            ),
            GraphQLError::InvalidAmount => juniper::FieldError::new(
                "The amount is invalid!",
                graphql_value!({
                    "code": "INVALID_AMOUNT"
                }),
            ),
            GraphQLError::AlreadyUsedEmail => juniper::FieldError::new(
                "The email address is already used!",
                graphql_value!({
                    "code": "ALREADY_USED_EMAIL"
                }),
            ),
            GraphQLError::UserNotFound => juniper::FieldError::new(
                "The viewer was not found!",
                graphql_value!({
                    "code": "USER_NOT_FOUND"
                }),
            ),
            GraphQLError::GroupNotFound => juniper::FieldError::new(
                "The group was not found!",
                graphql_value!({
                    "code": "GROUP_NOT_FOUND"
                }),
            ),
            GraphQLError::PersonNotFound => juniper::FieldError::new(
                "The person was not found!",
                graphql_value!({
                    "code": "PERSON_NOT_FOUND"
                }),
            ),
            GraphQLError::NonUniqueName(n) => juniper::FieldError::new(
                format!("The person's name ({}) is not unique!", n),
                graphql_value!({
                    "code": "NAME_NOT_UNIQUE"
                }),
            ),
            // https://docs.rs/anyhow/1.0.26/anyhow/struct.Error.html#display-representations
            GraphQLError::InternalServerError(e) => juniper::FieldError::new(
                format!("Something unexpected happend! Reason: {:#}", e),
                graphql_value!({
                    "code": "INTERNAL_SERVER_ERROR"
                }),
            ),
        }
    }
}
