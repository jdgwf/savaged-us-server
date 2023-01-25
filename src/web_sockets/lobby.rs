use crate::web_sockets::{messages::{ClientActorMessage, Connect, Disconnect, WsMessage}, handle_message::handle_message};
use actix::prelude::{Actor, Context, Handler, Recipient};
use savaged_libs::websocket_message::WebSocketMessage;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

// Socket = Recipient<WsMessage>;

#[derive(Clone)]
pub struct Lobby {
    pub sessions: HashMap<Uuid, Recipient<WsMessage>>, //self id to self
    pub rooms: HashMap<Uuid, HashSet<Uuid>>,           //room id  to list of users id
}

impl Default for Lobby {
    fn default() -> Lobby {
        Lobby {
            sessions: HashMap::new(),
            rooms: HashMap::new(),
        }
    }
}

impl Lobby {
    fn send_message(&self, message: &str, id_to: &Uuid) {
        if let Some(socket_recipient) = self.sessions.get(id_to) {
            let _ = socket_recipient.do_send(WsMessage(message.to_owned()));
        } else {
            println!("attempting to send message but couldn't find user id.");
        }
    }
}

impl Actor for Lobby {
    type Context = Context<Self>;
}

/// Handler for Disconnect message.
impl Handler<Disconnect> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        if self.sessions.remove(&msg.id).is_some() {
            println!("Handler for Disconnect Lobby");

            match msg.room_id {
                Some(room_id) => {
                    self.rooms
                        .get(&room_id)
                        .unwrap()
                        .iter()
                        .filter(|conn_id| *conn_id.to_owned() != msg.id)
                        .for_each(|user_id| {
                            self.send_message(&format!("{} disconnected.", &msg.id), user_id)
                        });
                    if let Some(lobby) = self.rooms.get_mut(&room_id) {
                        if lobby.len() > 1 {
                            lobby.remove(&msg.id);
                        } else {
                            //only one in the lobby, remove it entirely
                            self.rooms.remove(&room_id);
                        }
                    }
                }

                None => {}
            }
        }
    }
}

impl Handler<Connect> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        println!("Handler for Connect Lobby");
        // create a room if necessary, and then add the id to it
        match msg.room_id {
            Some(room_id) => {
                self.rooms
                    .entry(room_id)
                    .or_insert_with(HashSet::new)
                    .insert(msg.self_id);

                // send to everyone in the room that new uuid just joined
                self.rooms
                    .get(&room_id)
                    .unwrap()
                    .iter()
                    .filter(|conn_id| *conn_id.to_owned() != msg.self_id)
                    .for_each(|conn_id| {
                        self.send_message(&format!("{} just joined!", msg.self_id), conn_id)
                    });
            }
            None => {}
        }

        // store the address
        self.sessions.insert(msg.self_id, msg.addr);

        // send self your new uuid
        // self.send_message(&format!("your id is {}", msg.self_id), &msg.self_id);
    }
}

impl Handler<ClientActorMessage> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: ClientActorMessage, _: &mut Context<Self>) -> Self::Result {
        println!(
            "Handler<ClientActorMessage> {:?} {:?} {:?}",
            msg.id, msg.room_id, msg.msg
        );


        if msg.msg.starts_with("\\w") {
            if let Some(id_to) = msg.msg.split(' ').collect::<Vec<&str>>().get(1) {
                self.send_message(&msg.msg, &Uuid::parse_str(id_to).unwrap());
            }
        } else {
            match msg.room_id {
                Some(room_id) => {
                    self.rooms
                        .get(&room_id)
                        .unwrap()
                        .iter()
                        .for_each(|client| self.send_message(&msg.msg, client));
                }
                None => {}
            }
        }
    }
}

impl Lobby {
    pub fn get_sessions() -> Vec<Recipient<WsMessage>> {
        return Vec::new();
    }
}