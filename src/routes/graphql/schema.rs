use crate::{infrastructure::repositories, routes::graphql::errors::*};
use unicode_segmentation::UnicodeSegmentation;

pub struct Context {
    db_pool: repositories::PostgresPool,
}
impl Context {
    pub fn new(db_pool: repositories::PostgresPool) -> Self {
        Context { db_pool }
    }
}
impl juniper::Context for Context {}

pub struct Query;
#[juniper::object(Context = Context)]
impl Query {}

pub struct Mutation;
#[juniper::object(Context = Context)]
impl Mutation {
    // FIXME: Extract domain and repository logic to own module
    #[graphql(
        description = "Signup a new user. Check if the email isn't already taken or valid and that the password is valid and proceed to create his account."
    )]
    fn signup(context: &Context, email: String, password: String) -> Result<User, GraphQLError> {
        // Check email validity
        if !regex::Regex::new(r"^\S+@\S+\.\S+$")
            .unwrap()
            .is_match(&email)
        {
            return Err(GraphQLError::InvalidEmailAddress);
        }

        // Check password validity
        if !(8..=64).contains(&password.graphemes(true).count()) {
            return Err(GraphQLError::InvalidPassword);
        }

        // Check email availability
        match repositories::UserRepository::find_one_by_email(&email[..], &context.db_pool) {
            Err(e) => return Err(GraphQLError::InternalServerError(e.to_string())),
            Ok(Some(_)) => return Err(GraphQLError::AlreadyUsedEmail),
            _ => (),
        }

        // Save user account
        let new_user =
            repositories::models::NewUser::new(uuid::Uuid::new_v4(), email.clone(), password);

        if let Err(e) = repositories::UserRepository::save(&new_user, &context.db_pool) {
            return Err(GraphQLError::InternalServerError(e.to_string()));
        }

        let created_user = User::new(email);

        Ok(created_user)
    }

    // FIXME: Extract domain and repository logic to own module
    #[graphql(description = "Log in a user.")]
    fn login(context: &Context, email: String, password: String) -> Result<User, GraphQLError> {
        // Check email validity and password validity
        // https://stackoverflow.com/a/46290728
        if !regex::Regex::new(r"^\S+@\S+\.\S+$")
            .unwrap()
            .is_match(&email)
            || !(8..=64).contains(&password.graphemes(true).count())
        {
            return Err(GraphQLError::InvalidEmailAddress);
        }

        match repositories::UserRepository::find_one_by_email(&email[..], &context.db_pool) {
            Err(e) => Err(GraphQLError::InternalServerError(e.to_string())),
            Ok(None) => Err(GraphQLError::InvalidCredentials),
            Ok(Some(user)) => {
                if user.password != password {
                    Err(GraphQLError::InvalidCredentials)
                } else {
                    Ok(User::new(email))
                }
            }
        }
    }
}

#[derive(juniper::GraphQLObject)]
#[graphql(description = "The created user after sign up.")]
struct User {
    email: String,
}
impl User {
    fn new(email: String) -> Self {
        User { email }
    }
}

pub type Schema = juniper::RootNode<'static, Query, Mutation>;
