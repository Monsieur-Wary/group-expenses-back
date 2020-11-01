mod types;

use super::errors::*;
use crate::infrastructure::{config, repositories, security};
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

    // FIXME: Extract domain and repository logic to own module
    /// Get a group of the use.
    /// This is a user context dependant query.
    fn group(context: &Context, id: String) -> Result<Option<Group>, GraphQLError> {
        // Check id validity
        let id = match uuid::Uuid::parse_str(id.as_str()) {
            Err(e) => return Err(GraphQLError::InvalidId),
            Ok(u) => u,
        };
        // FIXME: Very inefficient quering. Should use joins instead ?
        let viewer = repositories::UserRepository::find_one(context.viewer.id(), &context.db_pool)
            .map_err(GraphQLError::InternalServerError)
            .and_then(|o| match o {
                None => Err(GraphQLError::UserNotFound),
                Some(u) => Ok(u),
            });
        viewer.and_then(|u| {
            repositories::GroupRepository::find_by_user(&u, &context.db_pool)
                .map_err(GraphQLError::InternalServerError)
                .map(|v| v.into_iter().find(|g| g.id == id).map(|g| g.into()))
        })
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

        let user = match repositories::UserRepository::save(&new_user, &context.db_pool) {
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
    /// Adds a group.
    /// This is a user context dependant mutation.
    fn addGroup(context: &Context, input: AddGroupInput) -> Result<bool, GraphQLError> {
        let AddGroupInput { name } = input;
        // Check name validity
        if !(1..=50).contains(&name.graphemes(true).count()) {
            return Err(GraphQLError::InvalidName);
        }
        // Check name uniqueness
        // FIXME: Very inefficient query. Should use joins instead ?
        let result = repositories::UserRepository::find_one(context.viewer.id(), &context.db_pool)
            .and_then(|o| {
                o.map(|u| {
                    repositories::GroupRepository::find_by_user(&u, &context.db_pool)
                        .map(|g| (u.id, g))
                })
                .transpose()
            });

        match result {
            Err(e) => Err(GraphQLError::InternalServerError(e)),
            Ok(None) => Err(GraphQLError::UserNotFound),
            Ok(Some((user_id, v))) => {
                if v.iter().any(|g| g.name == name) {
                    Err(GraphQLError::NonUniqueName(name))
                } else {
                    // Add this group to viewer's
                    let new_group = repositories::NewGroup {
                        id: uuid::Uuid::new_v4(),
                        user_id,
                        name,
                    };
                    repositories::GroupRepository::save(&new_group, &context.db_pool)
                        .map_err(GraphQLError::InternalServerError)
                        .map(|_| true)
                }
            }
        }
    }

    // FIXME: Extract domain and repository logic to own module
    /// Adds a person to the specified group.
    /// This is a user context dependant mutation.
    fn addPerson(context: &Context, input: AddPersonInput) -> Result<bool, GraphQLError> {
        let AddPersonInput {
            group_id,
            name,
            resources,
        } = input;
        // Check name validity
        if !(1..=50).contains(&name.graphemes(true).count()) {
            return Err(GraphQLError::InvalidName);
        }
        // Check resources validity
        if resources.is_negative() {
            return Err(GraphQLError::InvalidResources);
        }
        // Check id validity
        let group_id = match uuid::Uuid::parse_str(group_id.as_str()) {
            Err(e) => return Err(GraphQLError::InvalidId),
            Ok(u) => u,
        };
        // FIXME: Very inefficient quering. Should use joins instead ?
        let viewer = repositories::UserRepository::find_one(context.viewer.id(), &context.db_pool)
            .map_err(GraphQLError::InternalServerError)
            .and_then(|o| match o {
                None => Err(GraphQLError::UserNotFound),
                Some(u) => Ok(u),
            });
        let group = viewer.and_then(|u| {
            repositories::GroupRepository::find_by_user(&u, &context.db_pool)
                .map_err(GraphQLError::InternalServerError)
                .map(|v| v.into_iter().find(|g| g.id == group_id))
                .and_then(|o| match o {
                    None => Err(GraphQLError::GroupNotFound),
                    Some(g) => Ok(g),
                })
        });
        let person = group.and_then(|g| {
            repositories::PersonRepository::find_by_group(&g, &context.db_pool)
                .map_err(GraphQLError::InternalServerError)
                .map(|v| v.into_iter().find(|p| p.name == name))
                // Check name uniqueness
                .and_then(|o| match o {
                    Some(p) => Err(GraphQLError::NonUniqueName(name.clone())),
                    None => Ok(()),
                })
        });
        // Add this person to viewer's group
        person.and_then(|_| {
            let new_person = repositories::NewPerson {
                id: uuid::Uuid::new_v4(),
                group_id,
                name,
                resources,
            };
            repositories::PersonRepository::save(&new_person, &context.db_pool)
                .map_err(GraphQLError::InternalServerError)
                .map(|_| true)
        })
    }

    // FIXME: Extract domain and repository logic to own module
    /// Adds an expense to the specified group.
    /// This is a user context dependant mutation.
    fn addExpense(context: &Context, input: AddExpenseInput) -> Result<bool, GraphQLError> {
        let AddExpenseInput {
            group_id,
            person_id,
            name,
            amount,
        } = input;
        // Check input validity
        let group_id = match uuid::Uuid::parse_str(group_id.as_str()) {
            Err(e) => return Err(GraphQLError::InvalidId),
            Ok(u) => u,
        };
        let person_id = match uuid::Uuid::parse_str(person_id.as_str()) {
            Err(e) => return Err(GraphQLError::InvalidId),
            Ok(u) => u,
        };
        // Check amount validity
        if amount < 1 {
            return Err(GraphQLError::InvalidAmount);
        }
        // FIXME: Very inefficient quering. Should use joins instead ?
        let viewer = repositories::UserRepository::find_one(context.viewer.id(), &context.db_pool)
            .map_err(GraphQLError::InternalServerError)
            .and_then(|o| match o {
                None => Err(GraphQLError::UserNotFound),
                Some(u) => Ok(u),
            });
        let group = viewer.and_then(|u| {
            repositories::GroupRepository::find_by_user(&u, &context.db_pool)
                .map_err(GraphQLError::InternalServerError)
                .map(|v| v.into_iter().find(|g| g.id == group_id))
                .and_then(|o| match o {
                    None => Err(GraphQLError::GroupNotFound),
                    Some(g) => Ok(g),
                })
        });
        let person = group.and_then(|g| {
            repositories::PersonRepository::find_by_group(&g, &context.db_pool)
                .map_err(GraphQLError::InternalServerError)
                .map(|v| v.into_iter().find(|p| p.id == person_id))
                .and_then(|o| match o {
                    None => Err(GraphQLError::PersonNotFound),
                    Some(p) => Ok(p),
                })
        });
        // Add this expense to the viewer's group if the person exists
        person.and_then(|p| {
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
        })
    }

    // FIXME: Extract domain and repository logic to own module
    /// Update a person. Idempotent mutation.
    /// This is a user context dependant mutation.
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
    /// Update an expense. Idempotent mutation.
    /// This is a user context dependant mutation.
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
    /// Remove a person. Idempotent mutation.
    /// This is a user context dependant mutation.
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
    /// Remove an expense. Idempotent mutation.
    /// This is a user context dependant mutation.
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
