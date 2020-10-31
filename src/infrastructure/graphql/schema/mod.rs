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

        // Prepare the new group's data
        let group_id = uuid::Uuid::new_v4();
        let new_group = repositories::NewGroup {
            id: group_id,
            user_id,
        };

        let transaction = context
            .db_pool
            .get()
            .context("Couldn't get a connection for this transaction")
            .and_then(|conn| {
                conn.build_transaction().run(|| {
                    repositories::UserRepository::save(&new_user, &context.db_pool).and_then(|u| {
                        repositories::GroupRepository::save(&new_group, &context.db_pool).map(|_| u)
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
    /// Adds a person to the current group.
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
                o.map(|u| repositories::GroupRepository::find_one_by_user(&u, &context.db_pool))
                    .transpose()
            })
            .and_then(|o| {
                o.map(|g| {
                    repositories::PersonRepository::find_by_group(&g, &context.db_pool)
                        .map(|v| (g.id, v))
                })
                .transpose()
            });

        match result {
            Err(e) => Err(GraphQLError::InternalServerError(e)),
            Ok(None) => Err(GraphQLError::InternalServerError(anyhow::anyhow!(
                "Couldn't find this viewer's data"
            ))),
            Ok(Some((group_id, v))) => {
                if v.iter().any(|p| p.name == name) {
                    Err(GraphQLError::NonUniqueName(name))
                } else {
                    // Add this person to viewer's group
                    let new_person = repositories::NewPerson {
                        id: uuid::Uuid::new_v4(),
                        group_id,
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

    // FIXME: Extract domain and repository logic to own module
    /// Adds an expense to the current group.
    fn addExpense(context: &Context, input: AddExpenseInput) -> Result<bool, GraphQLError> {
        let AddExpenseInput {
            person_id,
            name,
            amount,
        } = input;
        // Check input validity
        let person_id = match uuid::Uuid::parse_str(person_id.as_str()) {
            Err(e) => return Err(GraphQLError::InvalidId),
            Ok(u) => u,
        };

        if amount < 1 {
            return Err(GraphQLError::InvalidAmount);
        }
        // Add this expense to the viewer's group if the person exsits
        // FIXME: Very inefficient query. Should use joins instead ?
        let result = repositories::UserRepository::find_one(context.viewer.id(), &context.db_pool)
            .and_then(|o| {
                o.map(|u| repositories::GroupRepository::find_one_by_user(&u, &context.db_pool))
                    .transpose()
            })
            .and_then(|o| {
                o.map(|g| {
                    repositories::PersonRepository::find_by_group(&g, &context.db_pool)
                        .map(|v| (g.id, v))
                })
                .transpose()
            });

        match result {
            Err(e) => Err(GraphQLError::InternalServerError(e)),
            Ok(None) => Err(GraphQLError::InternalServerError(anyhow::anyhow!(
                "Couldn't find this viewer's data"
            ))),
            Ok(Some((group_id, v))) => {
                if !v.iter().any(|p| p.id == person_id) {
                    Err(GraphQLError::PersonNotFound)
                } else {
                    // Add this person to viewer's group
                    let new_expense = repositories::NewExpense {
                        id: uuid::Uuid::new_v4(),
                        group_id,
                        person_id,
                        name,
                        amount,
                    };
                    repositories::ExpenseRepository::save(&new_expense, &context.db_pool)
                        .map_err(GraphQLError::InternalServerError)
                        .map(|_| true)
                }
            }
        }
    }

    // FIXME: Extract domain and repository logic to own module
    /// Update a person on the current group. Idempotent mutation.
    fn updatePerson(context: &Context, input: UpdatePersonInput) -> Result<bool, GraphQLError> {
        let UpdatePersonInput {
            person_id,
            name,
            resources,
        } = input;
        // Check input validity
        let person_id = match uuid::Uuid::parse_str(person_id.as_str()) {
            Err(e) => return Err(GraphQLError::InvalidId),
            Ok(u) => u,
        };

        let person = repositories::UpdatePerson {
            id: person_id,
            name,
            resources,
        };
        repositories::PersonRepository::update_one(&person, &context.db_pool)
            .map_err(GraphQLError::InternalServerError)
            .map(|_| true)
    }

    // FIXME: Extract domain and repository logic to own module
    /// Update an expense on the current group. Idempotent mutation.
    fn updateExpense(context: &Context, input: UpdateExpenseInput) -> Result<bool, GraphQLError> {
        let UpdateExpenseInput {
            expense_id,
            name,
            amount,
        } = input;
        // Check input validity
        let expense_id = match uuid::Uuid::parse_str(expense_id.as_str()) {
            Err(e) => return Err(GraphQLError::InvalidId),
            Ok(u) => u,
        };

        let expense = repositories::UpdateExpense {
            id: expense_id,
            name,
            amount,
        };
        repositories::ExpenseRepository::update_one(&expense, &context.db_pool)
            .map_err(GraphQLError::InternalServerError)
            .map(|_| true)
    }

    // FIXME: Extract domain and repository logic to own module
    /// Remove a person on the current group. Idempotent mutation.
    fn removePerson(context: &Context, input: RemovePersonInput) -> Result<bool, GraphQLError> {
        let RemovePersonInput { person_id } = input;
        // Check input validity
        let person_id = match uuid::Uuid::parse_str(person_id.as_str()) {
            Err(e) => return Err(GraphQLError::InvalidId),
            Ok(u) => u,
        };
        // Delete the person
        repositories::PersonRepository::delete_one(&person_id, &context.db_pool)
            .map_err(GraphQLError::InternalServerError)
            .map(|_| true)
    }

    // FIXME: Extract domain and repository logic to own module
    /// Remove an expense on the current group. Idempotent mutation.
    fn removeExpense(context: &Context, input: RemoveExpenseInput) -> Result<bool, GraphQLError> {
        let RemoveExpenseInput { expense_id } = input;
        // Check input validity
        let expense_id = match uuid::Uuid::parse_str(expense_id.as_str()) {
            Err(e) => return Err(GraphQLError::InvalidId),
            Ok(u) => u,
        };
        // Delete the expense
        repositories::ExpenseRepository::delete_one(&expense_id, &context.db_pool)
            .map_err(GraphQLError::InternalServerError)
            .map(|_| true)
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
