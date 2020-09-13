use juniper::graphql_value;

pub enum GraphQLError {
    InvalidPassword,
    InvalidEmailAddress,
    InvalidCredentials,
    AlreadyUsedEmail,
    InternalServerError(String),
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
            GraphQLError::AlreadyUsedEmail => juniper::FieldError::new(
                "The email address is already used!",
                graphql_value!({
                    "code": "ALREADY_USED_EMAIL"
                }),
            ),
            GraphQLError::InternalServerError(r) => juniper::FieldError::new(
                format!("Something unexpected happend! Reason: {}", r),
                graphql_value!({
                    "code": "INTERNAL_SERVER_ERROR"
                }),
            ),
        }
    }
}