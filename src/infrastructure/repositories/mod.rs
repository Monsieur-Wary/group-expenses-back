pub mod models;
pub mod schema;

use crate::configuration;
use diesel::{pg::PgConnection, prelude::*, r2d2::ConnectionManager};
use r2d2::Pool;
use schema::users;

// FIXME: Create a singleton with a pool field to avoid having to pass it to every methods.
pub struct UserRepository;
impl UserRepository {
    pub fn find_one_by_email(
        email: &str,
        pool: &PostgresPool,
    ) -> anyhow::Result<Option<models::User>> {
        users::table
            .filter(users::email.eq(email))
            .first(&pool.get()?)
            .optional()
            .map_err(anyhow::Error::new)
    }

    pub fn save(new_user: &models::NewUser, pool: &PostgresPool) -> anyhow::Result<()> {
        diesel::insert_into(users::table)
            .values(new_user)
            .execute(&pool.get()?)?;
        Ok(())
    }
}

// The Postgres-specific connection pool managing all database connections.
pub type PostgresPool = Pool<ConnectionManager<PgConnection>>;

/// Create the database connection pool.
pub fn get_pool(config: &configuration::Settings) -> anyhow::Result<PostgresPool> {
    let mgr = ConnectionManager::<PgConnection>::new(config.database().connection_string());
    r2d2::Pool::builder().build(mgr).map_err(anyhow::Error::new)
}
