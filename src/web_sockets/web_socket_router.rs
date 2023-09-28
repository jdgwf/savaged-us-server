use crate::db::users::get_remote_user;
use crate::web_sockets::Lobby;
use crate::web_sockets::ServerWebsocket;
use actix::Addr;
use actix_session::Session;
use actix_web::{get, web::Data, web::Payload, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use mysql_async::Pool;

#[get("/_ws")]
pub async fn web_socket_router(
    req: HttpRequest,
    pool: Data<Pool>,
    stream: Payload,
    chat_server: Data<Addr<Lobby>>,
    session: Session,
) -> Result<HttpResponse, Error> {
    let user = get_remote_user(&pool, None, None, req.clone(), session.clone()).await;


    let ws = ServerWebsocket::new(
        user,
        pool.clone(),
        chat_server.get_ref().clone(),
        req.clone(),
        session,
    );
    // println!("_ws hit");
    let resp = ws::start(ws, &req, stream)?;
    Ok(resp)
}
