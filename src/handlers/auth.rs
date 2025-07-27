
use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use crate::{db::DbPool, models::User, auth::{hash_password, create_jwt, verify_password}};

#[derive(Deserialize)]
pub struct AuthPayload {
    username: String,
    password: String,
}

pub async fn register(pool: web::Data<DbPool>, payload: web::Json<AuthPayload>) -> impl Responder {
    let hashed_password = match hash_password(&payload.password) {
        Ok(h) => h,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let new_user = sqlx::query_as::<_, User>(
        "INSERT INTO users (username, password_hash) VALUES ($1, $2) RETURNING *"
    )
    .bind(&payload.username)
    .bind(&hashed_password)
    .fetch_one(pool.get_ref())
    .await;

    match new_user {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(_) => HttpResponse::InternalServerError().body("Could not create user"),
    }
}

pub async fn login(pool: web::Data<DbPool>, payload: web::Json<AuthPayload>, secret: web::Data<String>) -> impl Responder {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
        .bind(&payload.username)
        .fetch_optional(pool.get_ref())
        .await;

    if let Ok(Some(user)) = user {
        if verify_password(&payload.password, &user.password_hash).unwrap_or(false) {
            let token = create_jwt(&user.id.to_string(), secret.get_ref()).unwrap();
            return HttpResponse::Ok().json(serde_json::json!({ "token": token }));
        }
    }

    HttpResponse::Unauthorized().body("Invalid credentials")
}
