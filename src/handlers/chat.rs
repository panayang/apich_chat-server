
use actix::prelude::*;
use actix_web::{web, HttpRequest, HttpResponse, Error};
use actix_web_actors::ws;
use crate::ws::{ChatServer, Connect, Disconnect, ClientMessage, WsMessage};
use uuid::Uuid;
use crate::db::DbPool;
use crate::models::{ChatRoom, Message, User};
use serde::Deserialize;

struct WsSession {
    user_id: Uuid,
    room_id: Uuid,
    server_addr: Addr<ChatServer>,
}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address();
        self.server_addr.do_send(Connect {
            addr: addr.recipient(),
            room_id: self.room_id,
            user_id: self.user_id,
        });
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        self.server_addr.do_send(Disconnect {
            room_id: self.room_id,
            user_id: self.user_id,
        });
        Running::Stop
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                self.server_addr.do_send(ClientMessage {
                    room_id: self.room_id,
                    user_id: self.user_id,
                    content: text.to_string(),
                });
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => (),
        }
    }
}

impl Handler<WsMessage> for WsSession {
    type Result = ();

    fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

pub async fn start_ws_connection(
    req: HttpRequest,
    stream: web::Payload,
    chat_server: web::Data<Addr<ChatServer>>,
    path: web::Path<(Uuid, Uuid)>,
) -> Result<HttpResponse, Error> {
    let (user_id, room_id) = path.into_inner();
    let session = WsSession {
        user_id,
        room_id,
        server_addr: chat_server.get_ref().clone(),
    };
    ws::start(session, &req, stream)
}

#[derive(Deserialize)]
pub struct CreateRoomPayload {
    name: String,
}

pub async fn create_room(pool: web::Data<DbPool>, payload: web::Json<CreateRoomPayload>) -> Result<HttpResponse, Error> {
    let new_room = sqlx::query_as::<_, ChatRoom>(
        "INSERT INTO chat_rooms (name) VALUES ($1) RETURNING *"
    )
    .bind(&payload.name)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|_| actix_web::error::ErrorInternalServerError("Could not create room"))?;

    Ok(HttpResponse::Ok().json(new_room))
}

pub async fn get_rooms(pool: web::Data<DbPool>) -> Result<HttpResponse, Error> {
    let rooms = sqlx::query_as::<_, ChatRoom>("SELECT * FROM chat_rooms")
        .fetch_all(pool.get_ref())
        .await
        .map_err(|_| actix_web::error::ErrorInternalServerError("Could not get rooms"))?;

    Ok(HttpResponse::Ok().json(rooms))
}

pub async fn get_messages(pool: web::Data<DbPool>, path: web::Path<Uuid>) -> Result<HttpResponse, Error> {
    let room_id = path.into_inner();
    let messages = sqlx::query_as::<_, Message>("SELECT * FROM messages WHERE room_id = $1 ORDER BY created_at ASC")
        .bind(room_id)
        .fetch_all(pool.get_ref())
        .await
        .map_err(|_| actix_web::error::ErrorInternalServerError("Could not get messages"))?;

    Ok(HttpResponse::Ok().json(messages))
}

pub async fn search_users(pool: web::Data<DbPool>, path: web::Path<String>) -> Result<HttpResponse, Error> {
    let username = format!("%{}%", path.into_inner());
    let users = sqlx::query_as::<_, User>("SELECT id, username, created_at FROM users WHERE username LIKE $1")
        .bind(username)
        .fetch_all(pool.get_ref())
        .await
        .map_err(|_| actix_web::error::ErrorInternalServerError("Could not search users"))?;

    Ok(HttpResponse::Ok().json(users))
}
