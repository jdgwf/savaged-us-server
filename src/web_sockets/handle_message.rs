use actix_web_actors::ws;
use log::Log;
use savaged_libs::{websocket_message::{
    WebSocketMessage,
    WebsocketMessageType,
}, user::LoginToken};
use super::ServerWebsocket;
use crate::{db::{users::{get_user_from_login_token, update_user_login_tokens}, chargen_data::get_chargen_data, saves::get_user_saves}, utils::send_standard_email};
use tokio::task;
use chrono::prelude::*;

pub fn handle_message(
    msg: WebSocketMessage,
    ctx: &mut ws::WebsocketContext<ServerWebsocket>,
    ws: &mut ServerWebsocket,
) {


    match msg.kind {

        WebsocketMessageType::Saves => {
            // println!("handle_message Saves {:?}", msg);

            let mut message_to_be_send = WebSocketMessage::default();
            message_to_be_send.kind = WebsocketMessageType::Saves;
            if msg.token != None {
                let user_option = get_user_from_login_token(
                    ws.pool.clone(),
                    msg.token,
                    ws.req.clone()
                );
                match user_option {
                    Some( user ) => {
                        let saves = get_user_saves(
                            &ws.pool.clone(),
                            user.id,
                            msg.updated_on,
                            false,
                        );
                        // for item in &saves {
                        //     if (&item.name).to_owned() == "Chi Master".to_owned() {
                        //         println!("saves item {:?}", item);
                        //     }
                        // }
                        message_to_be_send.saves = Some(
                            saves
                        );

                    }
                    None => {

                    }
                }
            }

            send_message( message_to_be_send, ctx );

        }
        WebsocketMessageType::ChargenData => {

            // println!("handle_message ChargenData {:?}", msg);

            let mut message_to_be_send = WebSocketMessage::default();
            message_to_be_send.kind = WebsocketMessageType::ChargenData;
            if msg.token != None {
                let user_option = get_user_from_login_token(
                    ws.pool.clone(),
                    msg.token,
                    ws.req.clone()
                );
                match user_option {
                    Some( user ) => {
                        // ws.user = Some(user.get_public_info());

                        // message_to_be_send.user = Some(user.clone());
                        // println!("** Online {:?}", ws.user);

                        message_to_be_send.chargen_data = Some(get_chargen_data(
                            &ws.pool.clone(),
                            user.id,
                            msg.updated_on,
                            true,
                            user.is_premium, // access_wildcard,
                            user.is_developer, // access_developer,
                            user.is_admin, // access_admin,
                            false, // all
                        ));

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

                        message_to_be_send.chargen_data = Some(get_chargen_data(
                            &ws.pool.clone(),
                            0,
                            msg.updated_on,
                            false,  // access_registered
                            false, // access_wildcard,
                            false,  // access_developer,
                            false,  // access_admin,
                            false, // all
                        ));
                    }
                }
            } else {
                message_to_be_send.chargen_data = Some(get_chargen_data(
                    &ws.pool.clone(),
                    0,
                    msg.updated_on,
                    false,  // access_registered
                    false, // access_wildcard,
                    false,  // access_developer,
                    false,  // access_admin,
                    false, // all
                ));
            }

            send_message( message_to_be_send, ctx );


        }
        WebsocketMessageType::Online => {
            // println!("handle_message Online {:?}", msg);
            // update_global_vars.emit( global_vars );

            let mut message_to_be_send = WebSocketMessage::default();

            message_to_be_send.kind = WebsocketMessageType::Online;
            // send_message( message_to_be_send, ctx );

            if msg.token != None {
                let user_option = get_user_from_login_token(
                    ws.pool.clone(),
                    msg.token,
                    ws.req.clone()
                );
                match user_option {
                    Some( user ) => {
                        ws.user = Some(user.clone());

                        message_to_be_send.user = Some(user.clone());

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


        WebsocketMessageType::Logout => {
            // println!("handle_message Offline {:?}", msg);
            // println!("Offline {:?}", ws.user);
            // update_global_vars.emit( global_vars );

            // ctx.text(msg);
            // let mut message_to_be_send = WebSocketMessage::default();
            // message_to_be_send.kind = WebsocketMessageType::Offline;
            // send_message( message_to_be_send, ctx );
            // update_user_login_tokens(pool, user_id, login_tokens)
            match msg.token {
                    Some( msg_token ) => {
                    let user_option = get_user_from_login_token(
                        ws.pool.clone(),
                        Some(msg_token.clone()),
                        ws.req.clone()
                    );
                    match user_option {
                        Some( user ) => {
                            let mut login_tokens: Vec<LoginToken> = Vec::new();

                            for mut token_entry in user.login_tokens.into_iter() {
                                if token_entry.token == msg_token {
                                    token_entry.logged_out = true;
                                    token_entry.token = "".to_owned();
                                    token_entry.last_seen = chrono::offset::Utc::now();
                                }
                                login_tokens.push( token_entry );
                            }

                            update_user_login_tokens(
                                ws.pool.clone(),
                                user.id,
                                login_tokens,
                            );
                            // ws.user = Some(user.get_public_info());

                            // message_to_be_send.user = Some(user.clone());

                            // // let pool = ws.pool.clone();
                            // // let user_id = user.id;
                            // // task::spawn_local(async move {
                            // //     println!("** Moo?");
                            // //     send_standard_email(
                            // //         pool,
                            // //         user_id,
                            // //         "Helloooooo".to_string(),
                            // //         r#"Don't be such a fart face <strong>Strong text</strong>"#.to_string()
                            // //     ).await;
                            // //     }
                            // // );
                        }
                        None => {

                        }
                    }

                }
                None => {}
            }
        }

        _ => {
            println!("ERROR websockets::handle_message::send_message Unhandled Message Type! {:?}", msg );
        }

        WebsocketMessageType::Offline => {
            // println!("handle_message Offline {:?}", msg);
            // println!("Offline {:?}", ws.user);
            // update_global_vars.emit( global_vars );

            // ctx.text(msg);
            let mut message_to_be_send = WebSocketMessage::default();
            message_to_be_send.kind = WebsocketMessageType::Offline;
            send_message( message_to_be_send, ctx );
        }

        _ => {
            println!("ERROR websockets::handle_message::send_message Unhandled Message Type! {:?}", msg );
        }
    }
}

fn send_message(
    send_message: WebSocketMessage,
    ctx: &mut ws::WebsocketContext<ServerWebsocket>,
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