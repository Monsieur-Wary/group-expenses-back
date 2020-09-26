mod types;

use super::errors::*;
use crate::infrastructure::{config, repositories, security};
use anyhow::Context as ErrorContext;
use types::*;
use unicode_segmentation::UnicodeSegmentation;

pub struct Query;
#[juniper::object(Context = Context)]
impl Query {
    // FIXME: Extract domain and repository logic to own module
    /// Login a user.
    fn login(context: &Context, email: String, password: String) -> Result<String, GraphQLError> {
        // Check email validity and password validity
        if !regex::Regex::new(r"^\S+@\S+\.\S+$")
            .unwrap()
            .is_match(&email)
            // https://stackoverflow.com/a/46290728
            || !(8..=64).contains(&password.graphemes(true).count())
        {
            return Err(GraphQLError::InvalidEmailAddress);
        }

        match repositories::UserRepository::find_one_by_email(&email[..], &context.db_pool) {
            Err(e) => Err(GraphQLError::InternalServerError(e)),
            Ok(None) => Err(GraphQLError::InvalidCredentials),
            Ok(Some(user)) => {
                match security::verify_password(password.as_bytes(), &user.password[..]) {
                    Err(e) => Err(GraphQLError::InternalServerError(e)),
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
                                Err(e) => return Err(GraphQLError::InternalServerError(e)),
                                Ok(token) => token,
                            };

                            Ok(token)
                        }
                    }
                }
            }
        }
    }

    // FIXME: Extract domain and repository logic to own module
    /// The authenticated user.
    /// This is a user context dependant query.
    fn viewer(context: &Context) -> Result<User, GraphQLError> {
        match repositories::UserRepository::find_one(context.viewer.id(), &context.db_pool) {
            Err(e) => Err(GraphQLError::InternalServerError(e)),
            Ok(None) => Err(GraphQLError::UserNotFound),
            Ok(Some(user)) => Ok(user.into()),
        }
    }
}

pub struct Mutation;
#[juniper::object(Context = Context)]
impl Mutation {
    // FIXME: Extract domain and repository logic to own module
    /// Signup a new user. Check if the email isn't already taken or valid and that the password is valid and proceed to create his account.
    fn signup(context: &Context, input: SignupInput) -> Result<String, GraphQLError> {
        let SignupInput { email, password } = input;

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
            Err(e) => return Err(GraphQLError::InternalServerError(e)),
            Ok(Some(_)) => return Err(GraphQLError::AlreadyUsedEmail),
            _ => (),
        }

        // Prepare the new user's data
        let user_id = uuid::Uuid::new_v4();
        let hash = match security::hash_password(
            password.as_bytes(),
            context.config.security().hash_salt(),
        ) {
            Err(e) => return Err(GraphQLError::InternalServerError(e)),
            Ok(hash) => hash,
        };
        let new_user = repositories::NewUser {
            id: user_id,
            email,
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
            .context("Couldn't get a connection for this transaction")
            .and_then(|conn| {
                conn.build_transaction().run(|| {
                    repositories::UserRepository::save(&new_user, &context.db_pool).and_then(|u| {
                        repositories::DashboardRepository::save(&new_dashboard, &context.db_pool)
                            .map(|_| u)
                    })
                })
            });

        let user = match transaction {
            Err(e) => return Err(GraphQLError::InternalServerError(e)),
            Ok(u) => u,
        };

        // Sign token
        let token = match security::sign_token(
            user_id,
            context.config.security().token_expiration_time(),
            context.config.security().secret_key(),
        ) {
            Err(e) => return Err(GraphQLError::InternalServerError(e)),
            Ok(token) => token,
        };

        Ok(token)
    }

    // FIXME: Extract domain and repository logic to own module
    /// Adds a person to the current dashboard.
    fn addPerson(context: &Context, input: AddPersonInput) -> Result<bool, GraphQLError> {
        let AddPersonInput { name, resources } = input;
        // Check name validity
        if !(1..=50).contains(&name.graphemes(true).count()) {
            return Err(GraphQLError::InvalidName);
        }
        // Check resources validity
        if resources.is_negative() {
            return Err(GraphQLError::InvalidResources);
        }
        // Check name uniqueness
        // FIXME: Very inefficient query. Should use joins instead ?
        let result = repositories::UserRepository::find_one(context.viewer.id(), &context.db_pool)
            .and_then(|o| {
                o.map(|u| repositories::DashboardRepository::find_one_by_user(&u, &context.db_pool))
                    .transpose()
            })
            .and_then(|o| {
                o.map(|d| {
                    repositories::PersonRepository::find_by_dashboard(&d, &context.db_pool)
                        .map(|v| (d.id, v))
                })
                .transpose()
            });

        match result {
            Err(e) => Err(GraphQLError::InternalServerError(e)),
            Ok(None) => Err(GraphQLError::InternalServerError(anyhow::anyhow!(
                "Couldn't find this viewer's data"
            ))),
            Ok(Some((dashboard_id, v))) => {
                if v.iter().any(|p| p.name == name) {
                    Err(GraphQLError::NonUniqueName(name))
                } else {
                    // Add this person to viewer's dashboard
                    let new_person = repositories::NewPerson {
                        id: uuid::Uuid::new_v4(),
                        dashboard_id,
                        name,
                        resources,
                    };
                    repositories::PersonRepository::save(&new_person, &context.db_pool)
                        .map_err(GraphQLError::InternalServerError)
                        .map(|_| true)
                }
            }
        }
    }
}

pub struct Context {
    pub db_pool: repositories::PostgresPool,
    pub config: config::Settings,
    pub viewer: security::Viewer,
}

impl juniper::Context for Context {}

pub type Schema = juniper::RootNode<'static, Query, Mutation>;

pub fn create_schema() -> Schema {
    Schema::new(Query, Mutation)
}
