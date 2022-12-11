use actix_web_actors::ws;
use savaged_libs::websocket_message::{
    WebSocketMessage,
    WebsocketMessageType,
};
use super::MyWs;
use crate::{db::users::get_user_from_login_token, utils::send_standard_email};
use tokio::task;

pub fn handle_message(
    msg: WebSocketMessage,
    ctx: &mut ws::WebsocketContext<MyWs>,
    ws: &mut MyWs,
) {


    match msg.kind {

        WebsocketMessageType::Saves => {
            println!("handle_message Online {:?}", msg);
        }
        WebsocketMessageType::ChargenData => {
            println!("handle_message Online {:?}", msg);
        }
        WebsocketMessageType::Online => {
            println!("handle_message Online {:?}", msg);
            // update_global_vars.emit( global_vars );

            let mut message_to_be_send: WebSocketMessage = WebSocketMessage {
                kind: WebsocketMessageType::Online,
                token: None,
                user: None,
                payload: None,
                chargen_data: None,
                saves: None,
            };

            // send_message( message_to_be_send, ctx );

            if msg.token != None {
                let user_option = get_user_from_login_token(
                    ws.pool.clone(),
                    msg.token,
                    ws.req.clone()
                );
                match user_option {
                    Some( user ) => {
                        ws.user = Some(user.get_public_info());

                        message_to_be_send.user = Some(user.clone());
                        println!("** Online {:?}", ws.user);

                        // let pool = ws.pool.clone();
                        // let user_id = user.id;
                        // task::spawn_local(async move {
                        //     println!("** Moo?");
                        //     send_standard_email(
                        //         pool,
                        //         user_id,
                        //         "Helloooooo".to_string(),
                        //         r#"Don't be such a fart face <strong>Strong text</strong>"#.to_string()
                        //     ).await;
                        //     }
                        // );
                    }
                    None => {

                    }
                }
            }

            send_message( message_to_be_send, ctx );

            // ctx.text(msg);
        }

        WebsocketMessageType::Offline => {
            println!("handle_message Offline {:?}", msg);
            println!("Offline {:?}", ws.user);
            // update_global_vars.emit( global_vars );

            // ctx.text(msg);
            let message_to_be_send: WebSocketMessage = WebSocketMessage {
                kind: WebsocketMessageType::Online,
                token: None,
                user: None,
                payload: None,
                chargen_data: None,
                saves: None,
            };
            send_message( message_to_be_send, ctx );
        }

        _ => {
            println!("ERROR websockets::handle_message::send_message Unhandled Message Type! {:?}", msg );
        }
    }
}

fn send_message(
    send_message: WebSocketMessage,
    ctx: &mut ws::WebsocketContext<MyWs>,
) {

    let send_data_result = serde_json::to_string( &send_message );

    match send_data_result {
        Ok( send_data ) => {
            ctx.text(send_data);
        }
        Err( err ) => {
            println!("ERROR websockets::handle_message::send_message json to_str error {} {:?}", err.to_string(), &send_message);
        }
    }


}