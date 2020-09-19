use super::*;

pub struct AuthPayload {
    pub token: String,
    pub user: User,
}

/// The payload received after a signup or a login.
#[juniper::object(Context = Context)]
impl AuthPayload {
    pub fn token(&self) -> &str {
        self.token.as_str()
    }

    pub fn user(&self) -> &User {
        &self.user
    }

    pub fn dashboard(&self, context: &Context) -> Result<Dashboard, GraphQLError> {
        self.user.dashboard(context)
    }
}

pub struct User(repositories::User);

impl User {
    fn dashboard(&self, context: &Context) -> Result<Dashboard, GraphQLError> {
        match repositories::DashboardRepository::find_one_by_user(&self.0, &context.db_pool) {
            Err(e) => Err(GraphQLError::InternalServerError(e)),
            Ok(d) => Ok(d.into()),
        }
    }
}

/// The created user after sign up.
#[juniper::object(Context = Context)]
impl User {
    fn email(&self) -> &str {
        &self.0.email[..]
    }

    fn dashboard(&self, context: &Context) -> Result<Dashboard, GraphQLError> {
        self.dashboard(context)
    }
}

impl From<repositories::User> for User {
    fn from(row: repositories::User) -> Self {
        User(row)
    }
}

/// The created dashboard after sign up.
pub struct Dashboard(repositories::Dashboard);

#[juniper::object(Context = Context)]
impl Dashboard {
    pub fn id(&self) -> String {
        self.0.id.to_string()
    }
}

impl From<repositories::Dashboard> for Dashboard {
    fn from(row: repositories::Dashboard) -> Self {
        Dashboard(row)
    }
}
