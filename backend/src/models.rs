use serde::Serialize;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct Login {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct Logout {
    pub session_id: String,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct Credentials {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct GetUserData {
    pub id: Uuid,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
}
