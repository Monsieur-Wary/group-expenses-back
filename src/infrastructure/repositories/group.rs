use super::{schema::groups, user::User, PostgresPool};
use anyhow::Context;
use diesel::prelude::*;

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(User)]
pub struct Group {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub struct GroupRepository;
impl GroupRepository {
    pub fn find_by_user(user: &User, pool: &PostgresPool) -> anyhow::Result<Vec<Group>> {
        Group::belonging_to(user)
            .load(&pool.get()?)
            .context(format!("Couldn't find this user's ({}) groups", user.id))
    }

    pub fn save(new_group: &NewGroup, pool: &PostgresPool) -> anyhow::Result<Group> {
        diesel::insert_into(groups::table)
            .values(new_group)
            .get_result::<Group>(&pool.get()?)
            .context("Couldn't save this group to the database")
    }
}

#[derive(Insertable)]
#[table_name = "groups"]
pub struct NewGroup {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub name: String,
}
