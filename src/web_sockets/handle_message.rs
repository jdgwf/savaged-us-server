use crate::db::users::get_remote_user;
use super::{ServerWebsocket, messages::ClientActorMessage};
use crate::{
    db::{
        game_data::get_game_data_package,
        saves::get_user_saves,
        users::{get_user_from_login_token, update_user_login_tokens}, get_web_content,
    },
    utils::send_standard_email,
};
use actix_session::Session;
use actix_web_actors::ws;
use chrono::prelude::*;
use log::Log;
use savaged_libs::{
    user::LoginToken,
    websocket_message::{WebSocketMessage, WebsocketMessageType},
};
use tokio::task;

pub async fn handle_message(
    msg: WebSocketMessage,
    ctx: &mut ws::WebsocketContext<ServerWebsocket>,
    ws: &mut ServerWebsocket,
) {
    println!("handle_message msg {:?}", msg);
    match msg.kind {
        WebsocketMessageType::SavesUpdated => {
            println!("handle_message Saves {:?}", msg);

            let mut msg_send = WebSocketMessage::default();
            msg_send.kind = WebsocketMessageType::SavesUpdated;
            // if msg.token != None {
            //     let user_option =
            //         get_user_from_login_token(&ws.pool, msg.token, ws.req.clone());
            //     match user_option {
            //         Some(user) => {
            //             let saves =
            //                 get_user_saves(&&ws.pool, user.id, msg.updated_on, false);
            //             // for item in &saves {
            //             //     if (&item.name).to_owned() == "Chi Master".to_owned() {
            //             //         println!("saves item {:?}", item);
            //             //     }
            //             // }
            //             msg_send.saves = Some(saves);
            //         }
            //         None => {}
            //     }
            // }
            // println!("Saves {:?}", &ws.user);
            match ws.user.clone() {
                Some(user) => {
                    let saves =
                        get_user_saves(&&ws.pool, user.id, msg.updated_on, false).await;
                    // for item in &saves {
                    //     if (&item.name).to_owned() == "Chi Master".to_owned() {
                    //         println!("saves item {:?}", item);
                    //     }
                    // }
                    msg_send.saves = Some(saves);
                }
                None => {}
            }
            send_message(msg_send, ctx);
        }
        WebsocketMessageType::GameDataPackageUpdated => {
            println!("handle_message GameDataPackageUpdated {:?}", msg);

            let mut msg_send = WebSocketMessage::default();
            msg_send.kind = WebsocketMessageType::GameDataPackageUpdated;


            // println!("GameDataPackage {:?}", &ws.user);
            match ws.user.clone() {
                Some(user) => {
                    msg_send.game_data = Some(get_game_data_package(
                        &&ws.pool,
                        user.id,
                        msg.updated_on,
                        true,
                        user.is_premium,   // access_wildcard,
                        user.is_developer, // access_developer,
                        user.is_admin,     // access_admin,
                        false,             // all
                    ).await);
                }
                None => {
                    msg_send.game_data = Some(get_game_data_package(
                        &&ws.pool,
                        0,
                        msg.updated_on,
                        false, // access_registered
                        false, // access_wildcard,
                        false, // access_developer,
                        false, // access_admin,
                        false, // all
                    ).await);
                }
            }

            send_message(msg_send, ctx);
        }
        WebsocketMessageType::Online => {

            let mut msg_send = WebSocketMessage::default();

            msg_send.kind = WebsocketMessageType::Online;
            // send_message( msg_send, ctx );
            msg_send.web_content = Some(get_web_content(&ws.pool).await);
            // let mut user = get_remote_user(&&ws.pool, None, None, ws.req.clone(), ws.session.clone());


            println!("handle_message online user {:?}", ws.user);

            match &ws.user {
                Some(user) => {
                    // ws.user = Some(user.clone());

                    msg_send.user = Some(user.clone());

                    // let pool = &ws.pool;
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
                    ws.user = None;
                }
            }

            send_message(msg_send, ctx);

            // ctx.text(msg);
        }

        WebsocketMessageType::RequestUsers => {

            match ws.user.clone() {
                Some( user ) => {
                    if user.has_admin_access() {
                        let mut msg_send = WebSocketMessage::default();

                        msg_send.kind = WebsocketMessageType::Online;

                        // ws.chat_server
                        // let chat_server = ws.chat_server.tx;

                        send_message(
                            msg_send, ctx
                        );
                    }
                }
                None => {

                }
            }
            // ws.location = msg.payload.clone();
            // ws.chat_server.do_send(ClientActorMessage {
            //     msg_type: WebsocketMessageType::SetLocation,
            //     id: ws.id,
            //     msg: msg.payload.unwrap(),
            //     room_id: ws.room_id,
            // });
        }

        WebsocketMessageType::SetLocation => {

            ws.location = msg.payload.clone();
            ws.chat_server.do_send(ClientActorMessage {
                msg_type: WebsocketMessageType::SetLocation,
                id: ws.id,
                msg: msg.payload.unwrap(),
                room_id: ws.room_id,
            });
        }

        WebsocketMessageType::SetRoom => {

            ws.location = msg.payload.clone();
            ws.chat_server.do_send(ClientActorMessage {
                msg_type: WebsocketMessageType::SetRoom,
                id: ws.id,
                msg: msg.payload.unwrap(),
                room_id: ws.room_id,
            });
        }

        WebsocketMessageType::Logout => {
            // println!("handle_message Offline {:?}", msg);
            // println!("Offline {:?}", ws.user);
            // update_site_vars.emit( global_vars );

            // ctx.text(msg);
            // let mut msg_send = WebSocketMessage::default();
            // msg_send.kind = WebsocketMessageType::Offline;
            // send_message( msg_send, ctx );
            // update_user_login_tokens(pool, user_id, login_tokens)
            let session_result =  ws.session.insert("user_id", 0);
            match session_result {
                Ok(_) => {
                    println!("handle_message Logout Session ID set {}", 0);
                }
                Err(err) => {
                    println!("handle_message Logout error setting session user {:?}", err);
                }
            }
            match msg.token {
                Some(msg_token) => {
                    let user_option = get_user_from_login_token(
                        &ws.pool,
                        Some(msg_token.clone()),
                        ws.req.clone(),
                    ).await;
                    match user_option {
                        Some(user) => {
                            let mut login_tokens: Vec<LoginToken> = Vec::new();

                            for mut token_entry in user.login_tokens.into_iter() {
                                if token_entry.token == msg_token {
                                    token_entry.logged_out = true;
                                    token_entry.token = "".to_owned();
                                    token_entry.last_seen = chrono::offset::Utc::now();
                                }
                                login_tokens.push(token_entry);
                            }

                            update_user_login_tokens(&ws.pool, user.id, login_tokens);
                            // ws.user = Some(user.get_public_info());

                            // msg_send.user = Some(user.clone());

                            // // let pool = &ws.pool;
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
                        None => {}
                    }
                }
                None => {}
            }
            // match ws.user.clone() {
            //     Some( user ) => {
            //         if user.has_admin_access() {
            //             let mut msg_send = WebSocketMessage::default();

            //             msg_send.kind = WebsocketMessageType::Online;

            //             // ws.chat_server
            //             // let chat_server = ws.chat_server.tx;

            //             send_message(
            //                 msg_send, ctx
            //             );
            //         }
            //     }
            //     None => {

            //     }
            // }
        }

        _ => {
            println!(
                "ERROR websockets::handle_message::send_message Unhandled Message Type! {:?}",
                msg
            );
        }

        WebsocketMessageType::Offline => {
            // println!("handle_message Offline {:?}", msg);
            // println!("Offline {:?}", ws.user);
            // update_site_vars.emit( global_vars );

            // ctx.text(msg);
            let mut msg_send = WebSocketMessage::default();
            msg_send.kind = WebsocketMessageType::Offline;
            send_message(msg_send, ctx);
        }

        _ => {
            println!(
                "ERROR websockets::handle_message::send_message Unhandled Message Type! {:?}",
                msg
            );
        }
    }
}

fn send_message(send_message: WebSocketMessage, ctx: &mut ws::WebsocketContext<ServerWebsocket>) {
    let send_data_result = serde_json::to_string(&send_message);

    match send_data_result {
        Ok(send_data) => {
            ctx.text(send_data);
        }
        Err(err) => {
            println!(
                "ERROR websockets::handle_message::send_message json to_str error {} {:?}",
                err.to_string(),
                &send_message
            );
        }
    }
}
