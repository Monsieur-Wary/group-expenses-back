use super::schema::users;

#[derive(Queryable)]
pub struct User {
    pub id: uuid::Uuid,
    pub email: String,
    pub password: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser {
    id: uuid::Uuid,
    email: String,
    password: String,
}

impl NewUser {
    pub fn new(id: uuid::Uuid, email: String, password: String) -> Self {
        NewUser {
            id,
            email,
            password,
        }
    }
}
