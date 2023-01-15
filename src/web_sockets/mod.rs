mod handle_message;
pub mod lobby;
mod messages;
pub mod web_socket_router;

use std::time::{Duration, Instant};

use actix_web::web::Data;
use mysql::{Pool, MySqlError};
use serde;
use serde::{Serialize, Deserialize};
use serde_json;
use savaged_libs::user::User;
use savaged_libs::websocket_message::{
    WebSocketMessage,
    WebsocketMessageType,
};
use actix::{Actor, StreamHandler, AsyncContext, Addr, Running, fut, ContextFutureSpawner, Handler};
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use actix::WrapFuture;
use actix::ActorFutureExt;
use actix::ActorContext;
use handle_message::handle_message;
use uuid::Uuid;

use self::lobby::Lobby;
use self::messages::{Connect, Disconnect, WsMessage};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// Define HTTP actor
pub struct ServerWebsocket {
    id: Uuid,
    user: Option<User>,
    hb: Instant,
    remote_ip: String,
    remote_browser: String,
    pool: Data<Pool>,
    chat_server: Addr<Lobby>,
    req: HttpRequest,
    room_id: Option<Uuid>,
}

impl Actor for ServerWebsocket {
    type Context = ws::WebsocketContext<Self>;

    fn started(
        &mut self,
        ctx: &mut Self::Context
    ) {
        self.hb(ctx);

        let addr = ctx.address();
        // let recip = ;
        self.chat_server
            .send(Connect {
                addr: addr.recipient(),
                room_id: self.room_id,
                self_id: self.id,
            })
            .into_actor(self)
            .then(|res, _, ctx| {
                match res {
                    Ok(_res) => (),
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(
        &mut self,
        _: &mut Self::Context
    ) -> Running {
        self.chat_server.do_send(Disconnect { id: self.id, room_id: self.room_id });
        Running::Stop
    }
}

impl Handler<WsMessage> for ServerWebsocket {
    type Result = ();

    fn handle(
        &mut self,
        msg: WsMessage,
        ctx: &mut Self::Context
    ) {
        ctx.text(msg.0);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ServerWebsocket {
    fn handle(
        &mut self,
        msg: Result<ws::Message, ws::ProtocolError>,
        ctx: &mut Self::Context,
    )
    {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                ctx.pong(&msg);
                self.hb = Instant::now();
            },
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            },
            Ok(ws::Message::Text(sent_data)) => {
                // ctx.text(text);
                let msg_result: Result<WebSocketMessage, serde_json::Error> = serde_json::from_str(&sent_data);
                match msg_result {
                    Ok( msg ) => {
                        handle_message(
                            msg,
                            ctx,
                            self,
                        );

                    }
                    Err( err ) => {
                        println!("ERROR websockets::StreamHandler json from_str error {}, {}", err.to_string(), &sent_data );

                    }
                }
            },

            Ok(ws::Message::Binary(bin)) => {
                ctx.binary(bin);
            },

            Ok( ws::Message::Close( closed ) ) => {
                println!("Closed event {:?}", closed );
                ctx.close(closed);
                // ctx.stop();
            }

            Err( err ) => {
//                println!("StreamHandler handle error {:?}", err );
            },

            _ => {

            }
        }
    }
}

impl ServerWebsocket {
    pub fn new (
        // room_id: Uuid,
        user: Option<User>,
        pool: Data<Pool>,
        chat_server: Addr<Lobby>,
        req: HttpRequest,
    ) -> ServerWebsocket {
        ServerWebsocket {
            id: Uuid::new_v4(),
            user: None,
            hb: Instant::now(),
            pool: pool,
            chat_server: chat_server,
            req: req.clone(),
            room_id: None,
            remote_browser: "".to_owned(),
            remote_ip: "".to_owned(),
        }
    }

    fn hb(
        &self,
        ctx: &mut ws::WebsocketContext<Self>
    ) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                println!("Disconnecting failed heartbeat");
                // act.chat_server.do_send(Disconnect { id: act.id, room_id: act.room });
                // ctx.stop();
                return;
            }

            ctx.ping(b"PING");
        });
    }
}

// pub async fn start_websocket_connection(
//     pool: Data<Pool>,
//     chat_server: Addr<Lobby>,
//     req: HttpRequest,
//     stream: web::Payload,
// ) -> Result<HttpResponse, Error> {

//     let resp = ws::start(
//         ServerWebsocket::new(
//             // Uuid::new_v4(),
//             None,
//             pool.clone(),
//             chat_server.clone(),
//             req.clone(),
//         ),
//         &req,
//         stream
//     );
//     println!("{:?}", resp);
//     resp
// }
