use super::{group::Group, person::Person, schema::expenses, PostgresPool};
use anyhow::Context;
use diesel::prelude::*;

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Group)]
#[belongs_to(Person)]
pub struct Expense {
    pub id: uuid::Uuid,
    pub group_id: uuid::Uuid,
    pub person_id: uuid::Uuid,
    pub name: String,
    pub amount: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub struct ExpenseRepository;
impl ExpenseRepository {
    pub fn find_by_person(person: &Person, pool: &PostgresPool) -> anyhow::Result<Vec<Expense>> {
        Expense::belonging_to(person)
            .load(&pool.get()?)
            .context(format!(
                "Couldn't find this person's ({}) expenses",
                person.id
            ))
    }

    pub fn find_by_group(group: &Group, pool: &PostgresPool) -> anyhow::Result<Vec<Expense>> {
        Expense::belonging_to(group)
            .load(&pool.get()?)
            .context(format!(
                "Couldn't find this group's ({}) expenses",
                group.id
            ))
    }

    pub fn save(new_expense: &NewExpense, pool: &PostgresPool) -> anyhow::Result<Expense> {
        diesel::insert_into(expenses::table)
            .values(new_expense)
            .get_result::<Expense>(&pool.get()?)
            .context("Couldn't save this expense to the database")
    }

    pub fn update_one(expense: &UpdateExpense, pool: &PostgresPool) -> anyhow::Result<()> {
        if expense.name.is_none() && expense.amount.is_none() {
            return Ok(());
        }

        diesel::update(expenses::table.filter(expenses::id.eq(expense.id)))
            .set(expense)
            .execute(&pool.get()?)
            .context("Couldn't update this expense to the database")
            .map(|_| ())
    }

    pub fn delete_one(id: &uuid::Uuid, pool: &PostgresPool) -> anyhow::Result<()> {
        diesel::delete(expenses::table)
            .filter(expenses::id.eq(id))
            .execute(&pool.get()?)
            .context(format!("Couldn't delete this expense ({})", id))
            .map(|_| ())
    }
}

#[derive(Insertable)]
#[table_name = "expenses"]
pub struct NewExpense {
    pub id: uuid::Uuid,
    pub group_id: uuid::Uuid,
    pub person_id: uuid::Uuid,
    pub name: String,
    pub amount: i32,
}

#[derive(AsChangeset)]
#[table_name = "expenses"]
pub struct UpdateExpense {
    pub id: uuid::Uuid,
    pub name: Option<String>,
    pub amount: Option<i32>,
}
