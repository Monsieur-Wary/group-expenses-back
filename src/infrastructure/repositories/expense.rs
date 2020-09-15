use super::{dashboard::Dashboard, person::Person, schema::expenses};

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Dashboard)]
#[belongs_to(Person)]
#[table_name = "expenses"]
pub struct Expense {
    pub id: uuid::Uuid,
    pub dashboard_id: uuid::Uuid,
    pub person_id: uuid::Uuid,
    pub name: String,
    pub amount: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
