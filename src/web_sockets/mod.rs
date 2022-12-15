mod handle_message;

use actix_web::web::Data;
use mysql::Pool;
use serde;
use serde::{Serialize, Deserialize};
use serde_json;
use savaged_libs::public_user_info::PublicUserInfo;
use savaged_libs::websocket_message::{
    WebSocketMessage,
    WebsocketMessageType,
};
use actix::{Actor, StreamHandler, AsyncContext};
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;

use handle_message::handle_message;
/// Define HTTP actor
pub struct MyWs {
    user: Option<PublicUserInfo>,
    pool: Data<Pool>,
    req: HttpRequest,
    room_id: String,
}

impl Actor for MyWs {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
    fn handle(
        &mut self,
        msg: Result<ws::Message, ws::ProtocolError>,
        ctx: &mut Self::Context,
    )
    {
        // println!("ctx address {:?}", ctx.address());
        // println!("user {:?}", self.user );
        // println!("self address {:?}", self. );

        match msg {
            Ok(ws::Message::Ping(msg)) => {
                ctx.pong(&msg);
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
            }

            Err( err ) => {
//                println!("StreamHandler handle error {:?}", err );
            },

            _ => {

            }
        }
    }
}

pub async fn websocket_handler(
    pool: Data<Pool>,
    req: HttpRequest,
    stream: web::Payload,

) -> Result<HttpResponse, Error> {

    let resp = ws::start(
        MyWs {
            user: None,
            pool: pool,
            req: req.clone(),
            room_id: "".to_owned(),
        },
        &req,
        stream
    );
    println!("{:?}", resp);
    resp
}
