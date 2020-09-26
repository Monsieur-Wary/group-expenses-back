use super::*;

pub struct User(repositories::User);

impl User {
    fn dashboard(&self, context: &Context) -> Result<Dashboard, GraphQLError> {
        match repositories::DashboardRepository::find_one_by_user(&self.0, &context.db_pool) {
            Err(e) => Err(GraphQLError::InternalServerError(e)),
            Ok(d) => Ok(d.into()),
        }
    }
}

/// A user.
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

/// A user's dashboard.
pub struct Dashboard(repositories::Dashboard);

impl Dashboard {
    fn expenses(&self, context: &Context) -> Result<Vec<Expense>, GraphQLError> {
        match repositories::ExpenseRepository::find_by_dashboard(&self.0, &context.db_pool)
            .map(|v| v.into_iter().map(Into::into).collect::<Vec<_>>())
        {
            Err(e) => Err(GraphQLError::InternalServerError(e)),
            Ok(v) => Ok(v),
        }
    }

    fn persons(&self, context: &Context) -> Result<Vec<Person>, GraphQLError> {
        match repositories::PersonRepository::find_by_dashboard(&self.0, &context.db_pool)
            .map(|v| v.into_iter().map(Into::into).collect::<Vec<_>>())
        {
            Err(e) => Err(GraphQLError::InternalServerError(e)),
            Ok(v) => Ok(v),
        }
    }
}

#[juniper::object(Context = Context)]
impl Dashboard {
    fn expenses(&self, context: &Context) -> Result<Vec<Expense>, GraphQLError> {
        self.expenses(context)
    }

    fn persons(&self, context: &Context) -> Result<Vec<Person>, GraphQLError> {
        self.persons(context)
    }
}

impl From<repositories::Dashboard> for Dashboard {
    fn from(row: repositories::Dashboard) -> Self {
        Dashboard(row)
    }
}

pub struct Expense(repositories::Expense);

/// A unique dashboard expense.
#[juniper::object(Context = Context)]
impl Expense {
    fn id(&self) -> String {
        self.0.id.to_string()
    }
}

impl From<repositories::Expense> for Expense {
    fn from(row: repositories::Expense) -> Self {
        Expense(row)
    }
}

pub struct Person(repositories::Person);

/// A unique dashboard person.
#[juniper::object(Context = Context)]
impl Person {
    fn id(&self) -> String {
        self.0.id.to_string()
    }
}

impl From<repositories::Person> for Person {
    fn from(row: repositories::Person) -> Self {
        Person(row)
    }
}

#[derive(juniper::GraphQLInputObject)]
pub struct SignupInput {
    pub email: String,
    pub password: String,
}

#[derive(juniper::GraphQLInputObject)]
pub struct AddPersonInput {
    pub name: String,
    pub resources: i32,
}

#[derive(juniper::GraphQLInputObject)]
pub struct AddExpenseInput {
    pub person_id: String,
    pub name: String,
    pub amount: i32,
}

#[derive(juniper::GraphQLInputObject)]
pub struct RemovePersonInput {
    pub person_id: String,
}

#[derive(juniper::GraphQLInputObject)]
pub struct RemoveExpenseInput {
    pub expense_id: String,
}
