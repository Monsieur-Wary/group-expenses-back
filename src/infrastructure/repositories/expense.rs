use super::{dashboard::Dashboard, person::Person, schema::expenses, PostgresPool};
use anyhow::Context;
use diesel::prelude::*;

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Dashboard)]
#[belongs_to(Person)]
#[table_name = "expenses"]
pub struct Expense {
    pub id: uuid::Uuid,
    pub dashboard_id: uuid::Uuid,
    pub person_id: uuid::Uuid,
    pub name: String,
    pub amount: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub struct ExpenseRepository;
impl ExpenseRepository {
    pub fn find_by_dashboard(
        dashboard: &Dashboard,
        pool: &PostgresPool,
    ) -> anyhow::Result<Vec<Expense>> {
        Expense::belonging_to(dashboard)
            .load(&pool.get()?)
            .context(format!(
                "Couldn't find this dashboard's ({}) expenses",
                dashboard.id
            ))
    }

    pub fn save(new_expense: &NewExpense, pool: &PostgresPool) -> anyhow::Result<Expense> {
        diesel::insert_into(expenses::table)
            .values(new_expense)
            .get_result::<Expense>(&pool.get()?)
            .context("Couldn't save this expense to the database")
    }
}

#[derive(Insertable)]
#[table_name = "expenses"]
pub struct NewExpense {
    pub id: uuid::Uuid,
    pub dashboard_id: uuid::Uuid,
    pub person_id: uuid::Uuid,
    pub name: String,
    pub amount: i32,
}
