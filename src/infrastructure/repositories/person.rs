use super::{dashboard::Dashboard, schema::persons, PostgresPool};
use anyhow::Context;
use diesel::prelude::*;

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Dashboard)]
#[table_name = "persons"]
pub struct Person {
    pub id: uuid::Uuid,
    pub dashboard_id: uuid::Uuid,
    pub name: String,
    pub resources: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub struct PersonRepository;
impl PersonRepository {
    pub fn find_by_dashboard(
        dashboard: &Dashboard,
        pool: &PostgresPool,
    ) -> anyhow::Result<Vec<Person>> {
        Person::belonging_to(dashboard)
            .load(&pool.get()?)
            .context(format!(
                "Couldn't find this dashboard's ({}) persons",
                dashboard.id
            ))
    }
}
