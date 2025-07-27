use actix::Actor;
use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use std::env;

mod db;
mod models;
mod auth;
mod handlers;
mod ws;
mod middleware;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let secret_key = env::var("SECRET_KEY").expect("SECRET_KEY must be set");

    let pool = db::connect(&database_url).await.expect("Failed to create pool.");
    let chat_server = ws::ChatServer::new(pool.clone()).start();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(secret_key.clone()))
            .app_data(web::Data::new(chat_server.clone()))
            .service(
                web::scope("/api")
                    .route("/register", web::post().to(handlers::auth::register))
                    .route("/login", web::post().to(handlers::auth::login))
            )
            .service(
                web::scope("/api")
                    .wrap(middleware::Authentication)
                    .route("/ws/{user_id}/{room_id}", web::get().to(handlers::chat::start_ws_connection))
                    .route("/rooms", web::post().to(handlers::chat::create_room))
                    .route("/rooms", web::get().to(handlers::chat::get_rooms))
                    .route("/rooms/{room_id}/messages", web::get().to(handlers::chat::get_messages))
                    .route("/users/search/{username}", web::get().to(handlers::chat::search_users))
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}