use super::{group::Group, schema::persons, PostgresPool};
use anyhow::Context;
use diesel::prelude::*;

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Group)]
pub struct Person {
    pub id: uuid::Uuid,
    pub group_id: uuid::Uuid,
    pub name: String,
    pub resources: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub struct PersonRepository;
impl PersonRepository {
    pub fn find_by_group(group: &Group, pool: &PostgresPool) -> anyhow::Result<Vec<Person>> {
        Person::belonging_to(group)
            .load(&pool.get()?)
            .context(format!("Couldn't find this group's ({}) persons", group.id))
    }

    pub fn save(new_person: &NewPerson, pool: &PostgresPool) -> anyhow::Result<Person> {
        diesel::insert_into(persons::table)
            .values(new_person)
            .get_result::<Person>(&pool.get()?)
            .context("Couldn't save this person to the database")
    }

    pub fn update_one(person: &UpdatePerson, pool: &PostgresPool) -> anyhow::Result<()> {
        if person.name.is_none() && person.resources.is_none() {
            return Ok(());
        }

        diesel::update(persons::table.filter(persons::id.eq(person.id)))
            .set(person)
            .execute(&pool.get()?)
            .context("Couldn't update this person to the database")
            .map(|_| ())
    }

    pub fn delete_one(id: &uuid::Uuid, pool: &PostgresPool) -> anyhow::Result<()> {
        diesel::delete(persons::table)
            .filter(persons::id.eq(id))
            .execute(&pool.get()?)
            .context(format!("Couldn't delete this person ({})", id))
            .map(|_| ())
    }
}

#[derive(Insertable)]
#[table_name = "persons"]
pub struct NewPerson {
    pub id: uuid::Uuid,
    pub group_id: uuid::Uuid,
    pub name: String,
    pub resources: i32,
}

#[derive(AsChangeset)]
#[table_name = "persons"]
pub struct UpdatePerson {
    pub id: uuid::Uuid,
    pub name: Option<String>,
    pub resources: Option<i32>,
}
