use actix_web_actors::ws;
use savaged_libs::websocket_message::{
    WebSocketMessage,
    WebsocketMessageType,
};
use super::MyWs;

pub fn handle_message(
    msg: WebSocketMessage,
    ctx: &mut ws::WebsocketContext<MyWs>,
) {


    match msg.kind {
        WebsocketMessageType::Online => {
            println!("handle_message Online {:?}", msg);
            // update_global_vars.emit( global_vars );

            let pong: WebSocketMessage = WebSocketMessage {
                kind: WebsocketMessageType::Online,
                token: "".to_owned(),
                user: None,
            };
            send_message( pong, ctx );

            // ctx.text(msg);
        }

        WebsocketMessageType::Offline => {
            println!("handle_message Offline {:?}", msg);
            // update_global_vars.emit( global_vars );

            // ctx.text(msg);
            let pong: WebSocketMessage = WebSocketMessage {
                kind: WebsocketMessageType::Online,
                token: "".to_owned(),
                user: None,
            };
            send_message( pong, ctx );
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