use crate::db::users::get_user;
use crate::db::users::get_user_from_login_token;
use crate::web_sockets::Lobby;
use crate::web_sockets::ServerWebsocket;
use actix::Addr;
use actix_session::Session;
use actix_web::{get, web::Data, web::Payload, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use mysql::Pool;

#[get("/_ws")]
pub async fn web_socket_router(
    req: HttpRequest,
    pool: Data<Pool>,
    stream: Payload,
    chat_server: Data<Addr<Lobby>>,
    session: Session,
) -> Result<HttpResponse, Error> {
    let mut user = get_user_from_login_token(&pool, None, req.clone());


    let session_result= session.get::<u32>("user_id");

    println!("web_socket_router");

    match session_result {
        Ok( user_id_option ) => {
            match user_id_option {
                Some( user_id ) => {
                    println!("web_socket_router SESSION value: {}", user_id);
                    // session_user_id = user_id;
                    // session.insert("web_socket_router user_id", login_results.user_id);
                    user = get_user(&pool, user_id);
                }
                None => {
                    // session.insert("user_id", login_results.user_id);
                    println!("web_socket_router SESSION value: None");
                }
            }

        }
        Err( err ) => {
            println!("Session Error {}", err);
        }
    }

    let ws = ServerWebsocket::new(
        user,
        pool.clone(),
        chat_server.get_ref().clone(),
        req.clone(),
        session,
    );

    let resp = ws::start(ws, &req, stream)?;
    Ok(resp)
}
