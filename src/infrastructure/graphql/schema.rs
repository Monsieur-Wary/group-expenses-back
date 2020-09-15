use super::errors::*;
use crate::infrastructure::{config, repositories, security};
use unicode_segmentation::UnicodeSegmentation;

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
    fn signup(
        context: &Context,
        email: String,
        password: String,
    ) -> Result<AuthPayload, GraphQLError> {
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

        // Prepare the new user's data
        let user_id = uuid::Uuid::new_v4();
        let hash = match security::hash_password(
            password.as_bytes(),
            context.config.security().hash_salt(),
        ) {
            Err(e) => return Err(GraphQLError::InternalServerError(e.to_string())),
            Ok(hash) => hash,
        };
        let new_user = repositories::NewUser {
            id: user_id,
            email: email.clone(),
            password: hash,
        };

        // Prepare the new dashboard's data
        let dashboard_id = uuid::Uuid::new_v4();
        let new_dashboard = repositories::NewDashboard {
            id: dashboard_id,
            user_id,
        };

        let transaction = context
            .db_pool
            .get()
            .map_err(anyhow::Error::new)
            .and_then(|conn| {
                conn.build_transaction().run(|| {
                    repositories::UserRepository::save(&new_user, &context.db_pool)?;
                    repositories::DashboardRepository::save(&new_dashboard, &context.db_pool)?;
                    Ok(())
                })
            });

        if let Err(e) = transaction {
            return Err(GraphQLError::InternalServerError(e.to_string()));
        }

        // Sign token
        let token = match security::sign_token(
            user_id,
            context.config.security().token_expiration_time(),
            context.config.security().secret_key(),
        ) {
            Err(e) => return Err(GraphQLError::InternalServerError(e.to_string())),
            Ok(token) => token,
        };
        let user = User { email };
        let dashboard = Dashboard {
            persons: vec![],
            expenses: vec![],
        };

        Ok(AuthPayload {
            token,
            user,
            dashboard,
        })
    }

    // FIXME: Extract domain and repository logic to own module
    #[graphql(description = "Log in a user.")]
    fn login(
        context: &Context,
        email: String,
        password: String,
    ) -> Result<AuthPayload, GraphQLError> {
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
                match security::verify_password(password.as_bytes(), &user.password[..]) {
                    Err(e) => Err(GraphQLError::InternalServerError(e.to_string())),
                    Ok(verified) => {
                        if !verified {
                            Err(GraphQLError::InvalidCredentials)
                        } else {
                            // Sign token
                            let token = match security::sign_token(
                                user.id,
                                context.config.security().token_expiration_time(),
                                context.config.security().secret_key(),
                            ) {
                                Err(e) => {
                                    return Err(GraphQLError::InternalServerError(e.to_string()))
                                }
                                Ok(token) => token,
                            };
                            let user = User { email };
                            // FIXME: Implement persons and expenses
                            let dashboard = Dashboard {
                                persons: vec![],
                                expenses: vec![],
                            };

                            Ok(AuthPayload {
                                token,
                                user,
                                dashboard,
                            })
                        }
                    }
                }
            }
        }
    }
}

pub struct Context {
    db_pool: repositories::PostgresPool,
    config: config::Settings,
}
impl Context {
    pub fn new(db_pool: repositories::PostgresPool, config: config::Settings) -> Self {
        Context { db_pool, config }
    }
}
impl juniper::Context for Context {}

#[derive(juniper::GraphQLObject)]
#[graphql(description = "The payload received after a signup or a login.")]
struct AuthPayload {
    pub token: String,
    pub user: User,
    pub dashboard: Dashboard,
}

#[derive(juniper::GraphQLObject)]
#[graphql(description = "The created user after sign up.")]
struct User {
    pub email: String,
}

#[derive(juniper::GraphQLObject)]
#[graphql(description = "The created dashboard after sign up.")]
struct Dashboard {
    pub persons: Vec<String>,
    pub expenses: Vec<String>,
}

pub type Schema = juniper::RootNode<'static, Query, Mutation>;

pub fn create_schema() -> Schema {
    Schema::new(Query, Mutation)
}
