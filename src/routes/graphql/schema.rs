use juniper::FieldResult;

#[derive(juniper::GraphQLObject)]
#[graphql(description = "The created user after sign up.")]
struct CreatedUser {
    email: String,
}
impl CreatedUser {
    fn new(email: String) -> Self {
        CreatedUser { email }
    }
}

pub struct Context;
impl juniper::Context for Context {}

pub struct Query;
#[juniper::object(Context = Context)]
impl Query {}

pub struct Mutation;
#[juniper::object(Context = Context)]
impl Mutation {
    #[graphql(
        description = "Signup a new user. Check if the email isn't already taken or valid and that the password is valid and proceed to create his account."
    )]
    fn signup(email: String, password: String) -> FieldResult<CreatedUser> {
        if password.is_empty() || email == "a@a.fr" {
            return Err("The credentials are invalid!".into());
        }

        let created_user = CreatedUser::new(email);

        Ok(created_user)
    }
}

pub type Schema = juniper::RootNode<'static, Query, Mutation>;
