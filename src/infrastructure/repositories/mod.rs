mod dashboard;
mod expense;
mod person;
mod schema;
mod user;

pub(super) use self::{dashboard::*, expense::*, person::*, user::*};
use crate::infrastructure::config;
use anyhow::Context;
use diesel::{pg::PgConnection, r2d2::ConnectionManager};
use r2d2::Pool;

/// The Postgres-specific connection pool managing all database connections.
pub type PostgresPool = Pool<ConnectionManager<PgConnection>>;

/// Create the database connection pool.
pub fn get_pool(config: &config::Settings) -> anyhow::Result<PostgresPool> {
    let mgr = ConnectionManager::<PgConnection>::new(config.database().connection_string());
    r2d2::Pool::builder()
        .build(mgr)
        .context("Couldn't build the postgres connection pool")
}
