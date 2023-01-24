use crate::db::users::get_user_from_login_token;
use crate::web_sockets::Lobby;
use crate::web_sockets::ServerWebsocket;
use actix::Addr;
use actix_web::{get, web::Data, web::Payload, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use mysql::Pool;

#[get("/_ws")]
pub async fn web_socket_router(
    req: HttpRequest,
    pool: Data<Pool>,
    stream: Payload,
    // Path(group_id): Path<Uuid>,
    chat_server: Data<Addr<Lobby>>,
) -> Result<HttpResponse, Error> {
    let user = get_user_from_login_token(pool.clone(), None, req.clone());

    let ws = ServerWebsocket::new(
        user,
        pool.clone(),
        chat_server.get_ref().clone(),
        req.clone(),
        // stream,
    );

    let resp = ws::start(ws, &req, stream)?;
    Ok(resp)
}
