mod api;
mod db;
extern crate dotenv;
use actix::Actor;
use actix_web::HttpRequest;
use mysql::*;

mod utils;
mod web_sockets;
// use web_sockets::start_websocket_connection;

// use yew::Renderer;
use yew::prelude::*;
use yew::ServerRenderer;
use savaged_front_end::{
    ServerApp,
    ServerAppProps
};
use tokio::task::LocalSet;
use tokio::task::{
    spawn_blocking,
    // spawn_local,
};
// use serde::{Deserialize, Serialize};

// use std::path::PathBuf;
use api::auth::{
    // get_user_groups,
    auth_api_login_for_token,
    auth_get_user_data,
    auth_token_remove,
    auth_token_update_name,
    auth_update_settings,

};
use api::saves::{
    auth_get_user_saves,
};
use api::notifications::{
    notifications_get,
    notifications_set_deleted,
    notifications_set_read,
    notifications_delete_basic_admin,
    notifications_set_all_read,
};

use api::data::chargen_data::{
    hindrances_get,
    chargen_data_get,
};
use api::data::books::books_get;

use api::banners::{
    banners_get,
};
use actix_files as fs;
use dotenv::dotenv;

use actix_web::{
    HttpServer,
    // get,
    HttpResponse,
    App,
    web::Data,
    middleware::Logger,
};
use actix_cors::Cors;

use crate::web_sockets::lobby::Lobby;
use crate::web_sockets::web_socket_router::web_socket_router;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let ssr_routes = vec![
        "/",

        "/info/",
        "/info",
        "/info/about",
        "/info/tech",
        "/info/to-dos",

        "/register",
        // "/playground",
        "/login",
        "/forgot-password",

        "/me/",
        "/me",
        "/me/settings-private",
        "/me/settings-public",
        "/me/notifications",
        "/me/subscription",
        "/me/devices",
        "/me/api-key",
        "/me/saves",
        "/me/campaigns",
    ];

    dotenv().ok();

    let mut serve_ip = "127.0.0.1".to_string();
    match std::env::var("SERVE_IP") {
        Ok( val ) => {
            serve_ip = val;
        }
        Err( _ ) => {

        }
    };

    let mut serve_port = 3000;
    match std::env::var("PORT") {
        Ok( val ) => {
            serve_port = val.parse().unwrap();
        }
        Err( _ ) => {

        }
    }

    let mut db_user = "".to_string();
    match std::env::var("DB_USER") {
        Ok( val ) => {
            db_user = val;
        }
        Err( _ ) => {

        }
    }

    let mut db_password = "".to_string();
    match std::env::var("DB_PASSWORD") {
        Ok( val ) => {
            db_password = val;
        }
        Err( _ ) => {

        }
    };
    let mut db_host = "".to_string();
    match std::env::var("DB_HOST") {
        Ok( val ) => {
            db_host = val;
        }
        Err( _ ) => {

        }
    };
    let mut db_port = "".to_string();
    match std::env::var("DB_PORT") {
        Ok( val ) => {
            db_port = val;
        }
        Err( _ ) => {

        }
    };
    let mut db_database = "".to_string();
    match std::env::var("DB_DATABASE") {
        Ok( val ) => {
            db_database = val;
        }
        Err( _ ) => {

        }
    };

    let mut db_socketpath = "".to_string();
    match std::env::var("DB_SOCKETPATH") {
        Ok( val ) => {
            db_socketpath = val;
        }
        Err( _ ) => {

        }
    };

    let mut db_conn_url = format!(
        "mysql://{}:{}@{}:{}/{}",
        db_user,
        db_password,
        db_host,
        db_port,
        db_database,
    );

    if !db_socketpath.is_empty() {
        db_conn_url = format!(
            "mysql://{}:{}@unix:{}/{}",
            db_user,
            db_password,
            db_socketpath,
            // db_port,
            db_database,
        );
    }

    // let mysql_connection_pool;

    println!("db_conn_url {}", format!(
        "mysql://{}:{}@{}:{}/{}",
        db_user,
        "REDACTED",
        db_host,
        db_port,
        db_database,
    ));

    match Opts::try_from( db_conn_url.as_ref() ) {
        Ok( opts ) => {

            match Pool::new(opts) {
                Ok( pool ) => {

                    let mysql_connection_pool = pool.clone();
                    let chat_server = Lobby::default().start(); //create and spin up a lobby

                    env_logger::init();

                    println!("Running on http://{}:{}", serve_ip, serve_port);
                    HttpServer::new( move || {
                        let logger = Logger::default();
                        let cors = Cors::permissive();

                        App:: new()
                            .wrap( logger )
                            .wrap( cors )
                            // .app_data(ApiError::json_error(JsonConfig::default()))

                            .app_data( Data::new(mysql_connection_pool.clone()))
                            .app_data( Data::new(chat_server.clone()))
                            // .route(
                            //     "/_ws",
                            //     actix_web::web::get().to(start_websocket_connection)
                            // )
                            .service( web_socket_router )
                            // Authentication Handlers
                            .service( auth_api_login_for_token )
                            .service( auth_get_user_data )

                            // Saves Handlers
                            .service( auth_get_user_saves )

                            // User Token Administration
                            .service( auth_token_remove )
                            .service( auth_update_settings )
                            .service( auth_token_update_name )

                            // User Notification Page Handlers
                            .service( notifications_set_deleted )
                            .service( notifications_set_read )
                            .service( notifications_get )
                            .service( notifications_delete_basic_admin )
                            .service( notifications_set_all_read )


                            // Data Endpoints
                            .service( hindrances_get )
                            .service( books_get )
                            .service( chargen_data_get )

                            // get banners API
                            .service( banners_get )

                            // render yew app SSR.
                            .service(
                                actix_web::web::resource(
                                    ssr_routes.clone() // see above for routes which will render via SSR
                                ).route(actix_web::web::get().to(yew_render))
                            )

                            // serve user images...
                            .service(
                                fs::Files::new(
                                    "/data-images/",
                                    "./data/uploads/")
                                    .use_last_modified(true)

                            )

                            // serve other file system files...
                            .service(
                                fs::Files::new(
                                    "/",
                                    "./public")
                                    .use_last_modified(true)

                            )

                    }).bind( (serve_ip, serve_port) )?
                    .run()
                    .await

                }
                Err( err ) => {
                    println!("MysqL Pool Error 2 {}", err );
                    std::process::exit(0x0100);
                }
            }
        }
        Err( err ) => {
            println!("MysqL Pool Error 1 {}", err );
            std::process::exit(0x0100);
        }
    }

}

async fn yew_render(
    request: HttpRequest,
) -> HttpResponse {
    let url = request.uri().to_string();
    let content = spawn_blocking(move || {
        use tokio::runtime::Builder;
        let set = LocalSet::new();

        let rt = Builder::new_current_thread().enable_all().build().unwrap();

        set.block_on(&rt, async {
            // let server_renderer = ServerRenderer::<ServerApp>::new();
            // let url = url.to_owned();
            let server_renderer = ServerRenderer::<ServerApp>::with_props(
                move || {

                    ServerAppProps {
                        url: AttrValue::from(url.clone()),
                    }
                }
            );

            server_renderer.render().await

        })
    })
    .await
    .expect("the thread has failed.");

    let index_html_s = tokio::fs::read_to_string("./public/index.html")
    .await
    .expect("failed to read ./public/index.html");

    let (index_html_before, index_html_after) = index_html_s.split_once("<body>").unwrap();
    let mut index_html_before = index_html_before.to_owned();
    index_html_before.push_str("<body>");

    let index_html_after = index_html_after.to_owned();

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(format!(
            "{}{}{}",
            index_html_before,
            content,
            index_html_after,
        ))
}

