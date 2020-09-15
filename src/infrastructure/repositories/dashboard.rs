use super::{schema::dashboards, user::User, PostgresPool};
use diesel::prelude::*;

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(User, foreign_key = "id")]
#[table_name = "dashboards"]
pub struct Dashboard {
    pub id: uuid::Uuid,
    user_id: uuid::Uuid,
    created_at: chrono::DateTime<chrono::Utc>,
}

pub struct DashboardRepository;
impl DashboardRepository {
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
