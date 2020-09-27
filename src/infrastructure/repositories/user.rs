use super::{schema::users, PostgresPool};
use anyhow::Context;
use diesel::prelude::*;

#[derive(Identifiable, Queryable, PartialEq, Debug)]
pub struct User {
    pub id: uuid::Uuid,
    pub email: String,
    pub password: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// FIXME: Create a singleton with a pool field to avoid having to pass it to every methods.
pub struct UserRepository;
impl UserRepository {
    pub fn find_one(id: &uuid::Uuid, pool: &PostgresPool) -> anyhow::Result<Option<User>> {
        users::table
            .find(id)
            .first(&pool.get()?)
            .optional()
            .context("Couldn't find one user")
    }

    pub fn find_one_by_email(email: &str, pool: &PostgresPool) -> anyhow::Result<Option<User>> {
        users::table
            .filter(users::email.eq(email))
            .first(&pool.get()?)
            .optional()
            .context("Couldn't query to find one user by email")
    }

    pub fn save(new_user: &NewUser, pool: &PostgresPool) -> anyhow::Result<User> {
        diesel::insert_into(users::table)
            .values(new_user)
            .get_result::<User>(&pool.get()?)
            .context("Couldn't save this user to the database")
    }
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser {
    pub id: uuid::Uuid,
    pub email: String,
    pub password: String,
}
