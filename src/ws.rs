
use actix::prelude::*;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Message)]
#[rtype(result = "()")]
pub struct WsMessage(pub String);

#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub addr: Recipient<WsMessage>,
    pub room_id: Uuid,
    pub user_id: Uuid,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub room_id: Uuid,
    pub user_id: Uuid,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientMessage {
    pub room_id: Uuid,
    pub user_id: Uuid,
    pub content: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct BroadcastMessage(pub crate::models::Message);


pub struct ChatServer {
    sessions: HashMap<Uuid, Recipient<WsMessage>>,
    rooms: HashMap<Uuid, HashSet<Uuid>>,
    db_pool: crate::db::DbPool,
}

impl ChatServer {
    pub fn new(db_pool: crate::db::DbPool) -> ChatServer {
        ChatServer {
            sessions: HashMap::new(),
            rooms: HashMap::new(),
            db_pool,
        }
    }
}

impl Actor for ChatServer {
    type Context = Context<Self>;
}

impl Handler<Connect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) {
        self.rooms
            .entry(msg.room_id)
            .or_insert_with(HashSet::new)
            .insert(msg.user_id);

        self.sessions.insert(msg.user_id, msg.addr);
    }
}

impl Handler<Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        if let Some(room) = self.rooms.get_mut(&msg.room_id) {
            room.remove(&msg.user_id);
        }
        self.sessions.remove(&msg.user_id);
    }
}

impl Handler<ClientMessage> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, ctx: &mut Context<Self>) {
        let pool = self.db_pool.clone();
        let room_id = msg.room_id;
        let user_id = msg.user_id;
        let content = msg.content.clone();
        let self_addr = ctx.address();

        actix::spawn(async move {
            let new_message = sqlx::query_as::<_, crate::models::Message>(
                "INSERT INTO messages (room_id, user_id, content) VALUES ($1, $2, $3) RETURNING *"
            )
            .bind(room_id)
            .bind(user_id)
            .bind(content)
            .fetch_one(&pool)
            .await;

            if let Ok(message) = new_message {
                self_addr.do_send(BroadcastMessage(message));
            }
        });
    }
}

impl Handler<BroadcastMessage> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: BroadcastMessage, _: &mut Context<Self>) {
        if let Some(room) = self.rooms.get(&msg.0.room_id) {
            for user_id in room {
                if let Some(addr) = self.sessions.get(user_id) {
                    let _ = addr.do_send(WsMessage(serde_json::to_string(&msg.0).unwrap()));
                }
            }
        }
    }
}
