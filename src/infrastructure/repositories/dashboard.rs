use super::{schema::dashboards, user::User, PostgresPool};
use anyhow::Context;
use diesel::prelude::*;

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(User)]
#[table_name = "dashboards"]
pub struct Dashboard {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub struct DashboardRepository;
impl DashboardRepository {
    pub fn find_one_by_user(user: &User, pool: &PostgresPool) -> anyhow::Result<Dashboard> {
        Dashboard::belonging_to(user)
            .first(&pool.get()?)
            .context(format!(
                "Couldn't find this user's ({}) dashboard.",
                user.id
            ))
    }

    pub fn save(new_dashboard: &NewDashboard, pool: &PostgresPool) -> anyhow::Result<()> {
        diesel::insert_into(dashboards::table)
            .values(new_dashboard)
            .execute(&pool.get()?)?;
        Ok(())
    }
}

#[derive(Insertable)]
#[table_name = "dashboards"]
pub struct NewDashboard {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
}
