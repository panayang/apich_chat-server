
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct ChatRoom {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct Message {
    pub id: Uuid,
    pub room_id: Uuid,
    pub user_id: Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
}
