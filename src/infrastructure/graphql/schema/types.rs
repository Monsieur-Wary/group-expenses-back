use super::*;

pub struct User(repositories::User);

impl User {
    fn groups(&self, context: &Context) -> Result<Vec<Group>, GraphQLError> {
        match repositories::GroupRepository::find_by_user(&self.0, &context.db_pool) {
            Err(e) => Err(GraphQLError::InternalServerError(e)),
            Ok(v) => Ok(v.into_iter().map(Into::into).collect()),
        }
    }
}

/// A user.
#[juniper::object(Context = Context)]
impl User {
    fn email(&self) -> &str {
        &self.0.email[..]
    }

    fn groups(&self, context: &Context) -> Result<Vec<Group>, GraphQLError> {
        self.groups(context)
    }
}

impl From<repositories::User> for User {
    fn from(row: repositories::User) -> Self {
        User(row)
    }
}

/// A user's group.
pub struct Group(repositories::Group);

impl Group {
    fn expenses(&self, context: &Context) -> Result<Vec<Expense>, GraphQLError> {
        match repositories::ExpenseRepository::find_by_group(&self.0, &context.db_pool)
            .map(|v| v.into_iter().map(Into::into).collect())
        {
            Err(e) => Err(GraphQLError::InternalServerError(e)),
            Ok(v) => Ok(v),
        }
    }

    fn persons(&self, context: &Context) -> Result<Vec<Person>, GraphQLError> {
        match repositories::PersonRepository::find_by_group(&self.0, &context.db_pool)
            .map(|v| v.into_iter().map(Into::into).collect())
        {
            Err(e) => Err(GraphQLError::InternalServerError(e)),
            Ok(v) => Ok(v),
        }
    }
}

#[juniper::object(Context = Context)]
impl Group {
    fn id(&self) -> String {
        self.0.id.to_string()
    }

    fn name(&self) -> &str {
        self.0.name.as_str()
    }

    fn expenses(&self, context: &Context) -> Result<Vec<Expense>, GraphQLError> {
        self.expenses(context)
    }

    fn persons(&self, context: &Context) -> Result<Vec<Person>, GraphQLError> {
        self.persons(context)
    }
}

impl From<repositories::Group> for Group {
    fn from(row: repositories::Group) -> Self {
        Group(row)
    }
}

pub struct Expense(repositories::Expense);

/// A unique group expense.
#[juniper::object(Context = Context)]
impl Expense {
    fn id(&self) -> String {
        self.0.id.to_string()
    }

    fn name(&self) -> &str {
        self.0.name.as_str()
    }

    fn amount(&self) -> &i32 {
        &self.0.amount
    }
}

impl From<repositories::Expense> for Expense {
    fn from(row: repositories::Expense) -> Self {
        Expense(row)
    }
}

pub struct Person(repositories::Person);

impl Person {
    fn expenses(&self, context: &Context) -> Result<Vec<Expense>, GraphQLError> {
        match repositories::ExpenseRepository::find_by_person(&self.0, &context.db_pool)
            .map(|v| v.into_iter().map(Into::into).collect())
        {
            Err(e) => Err(GraphQLError::InternalServerError(e)),
            Ok(v) => Ok(v),
        }
    }
}

/// A unique group person.
#[juniper::object(Context = Context)]
impl Person {
    fn id(&self) -> String {
        self.0.id.to_string()
    }

    fn name(&self) -> &str {
        self.0.name.as_str()
    }

    fn resources(&self) -> &i32 {
        &self.0.resources
    }

    fn expenses(&self, context: &Context) -> Result<Vec<Expense>, GraphQLError> {
        self.expenses(context)
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
pub struct AddGroupInput {
    pub name: String,
}

#[derive(juniper::GraphQLInputObject)]
pub struct AddPersonInput {
    pub group_id: String,
    pub name: String,
    pub resources: i32,
}

#[derive(juniper::GraphQLInputObject)]
pub struct AddExpenseInput {
    pub group_id: String,
    pub person_id: String,
    pub name: String,
    pub amount: i32,
}

#[derive(juniper::GraphQLInputObject)]
pub struct UpdatePersonInput {
    pub person_id: String,
    pub name: Option<String>,
    pub resources: Option<i32>,
}

#[derive(juniper::GraphQLInputObject)]
pub struct UpdateExpenseInput {
    pub expense_id: String,
    pub name: Option<String>,
    pub amount: Option<i32>,
}

#[derive(juniper::GraphQLInputObject)]
pub struct RemovePersonInput {
    pub person_id: String,
}

#[derive(juniper::GraphQLInputObject)]
pub struct RemoveExpenseInput {
    pub expense_id: String,
}
