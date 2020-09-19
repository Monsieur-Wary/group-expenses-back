use super::{dashboard::Dashboard, schema::persons};

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Dashboard)]
#[table_name = "persons"]
pub struct Person {
    pub id: uuid::Uuid,
    pub dashboard_id: uuid::Uuid,
    pub name: String,
    pub resources: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
