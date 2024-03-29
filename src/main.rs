mod api;
mod db;
extern crate dotenv;
use actix::Actor;
use actix_session::config::CookieContentSecurity;
use actix_session::config::PersistentSession;
use actix_session::storage::SessionKey;
use actix_web::FromRequest;
use actix_web::body::MessageBody;
use actix_web::cookie::time;
use actix_web::http::header;
use actix_web::HttpRequest;
use actix_web::web;
use mysql_async::*;
use actix_web::cookie::Key;
mod utils;
mod web_sockets;
// use web_sockets::start_websocket_connection;

// use yew::Renderer;
use savaged_front_end::{ServerApp, ServerAppProps};
use savaged_libs::web_content::WebContent;
use tokio::task::spawn_blocking;
use tokio::task::LocalSet;
use yew::prelude::*;
use yew::ServerRenderer;
// use serde::{Deserialize, Serialize};

use db::banners::get_active_banners;
// use std::path::PathBuf;
use api::auth::{
    api_auth_get_user_data,
    api_auth_login,
    api_logout,
    api_auth_login_for_token,
};

use actix_session::{Session, SessionMiddleware, storage::CookieSessionStore};

use api::user::{
    api_user_save_username, api_user_set_user_image_data, api_user_token_remove,
    api_user_token_update_name, api_user_update_settings, api_user_username_available,
};

use api::notifications::{
    api_notifications_delete_basic_admin, api_notifications_get, api_notifications_set_all_read,
    api_notifications_set_deleted, api_notifications_set_read,
};
use api::saves::api_saves_get;

use api::admin::users::{api_admin_users_get, api_admin_users_paging};

use api::admin::game_data::{
    api_admin_game_data_delete, api_admin_game_data_get, api_admin_game_data_paging,
    api_admin_game_data_save,
};

use api::data::game_data::{
    api_game_data_get,
};
// use api::data::books::books_get;

use api::banners::api_banners_get;

use actix_files as fs;
use dotenv::dotenv;

use actix_cors::Cors;
use actix_web::{
    middleware::Logger,
    web::Data,
    App,
    // get,
    HttpResponse,
    HttpServer,
};

use crate::db::get_web_content;
use crate::db::users::get_user;
use crate::web_sockets::lobby::Lobby;
use crate::web_sockets::web_socket_router::web_socket_router;

const VERSION: &str = env!("CARGO_PKG_VERSION");


pub const CONFIG_ALLOWED_IMAGE_TYPES: &'static [&'static str] =
    &["image/jpeg", "image/jpg", "image/png", "image/webp"];

fn get_secret_key() -> Key {
    // let key: String = std::iter::repeat('a').take(4065).collect();
    // let session_key: Result<SessionKey, _> = key.try_into();
    let mut secret_key = "".to_string();
    match std::env::var("SHA_SECRET_KEY") {
        Ok(val) => {
            secret_key = val.clone() + &"-" + &val;
        }
        Err(_) => {}
    };

    return Key::from( secret_key.as_str().as_bytes() );
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let mut serve_ip = "127.0.0.1".to_string();
    match std::env::var("SERVE_IP") {
        Ok(val) => {
            serve_ip = val;
        }
        Err(_) => {}
    };

    let mut serve_port = 3000;
    match std::env::var("PORT") {
        Ok(val) => {
            serve_port = val.parse().unwrap();
        }
        Err(_) => {}
    }

    let mut db_user = "".to_string();
    match std::env::var("DB_USER") {
        Ok(val) => {
            db_user = val;
        }
        Err(_) => {}
    }

    let mut db_password = "".to_string();
    match std::env::var("DB_PASSWORD") {
        Ok(val) => {
            db_password = val;
        }
        Err(_) => {}
    };
    let mut db_host = "".to_string();
    match std::env::var("DB_HOST") {
        Ok(val) => {
            db_host = val;
        }
        Err(_) => {}
    };
    let mut db_port = "".to_string();
    match std::env::var("DB_PORT") {
        Ok(val) => {
            db_port = val;
        }
        Err(_) => {}
    };
    let mut db_database = "".to_string();
    match std::env::var("DB_DATABASE") {
        Ok(val) => {
            db_database = val;
        }
        Err(_) => {}
    };

    let mut db_socketpath = "".to_string();
    match std::env::var("DB_SOCKETPATH") {
        Ok(val) => {
            db_socketpath = val;
        }
        Err(_) => {}
    };

    let mut db_conn_url = format!(
        "mysql://{}:{}@{}:{}/{}",
        db_user, db_password, db_host, db_port, db_database,
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

    println!(
        "db_conn_url {}",
        format!(
            "mysql://{}:{}@{}:{}/{}",
            db_user, "REDACTED", db_host, db_port, db_database,
        )
    );

    let cookie_secret_key = get_secret_key();

    match Opts::try_from(db_conn_url.as_ref()) {
        Ok(opts) => {
            let pool = Pool::new(opts);
                // Ok(pool) => {
                    let mysql_connection_pool = pool.clone();
                    let chat_server = Lobby::default().start(); //create and spin up a lobby



                    env_logger::init();

                    println!("Version {} running on http://{}:{}", VERSION, serve_ip, serve_port);
                    HttpServer::new(move || {

                        // let session_middleware = SessionMiddleware::new(
                        //     CookieSessionStore::default(),
                        //     cookie_secret_key.clone()
                        // );

                        let logger = Logger::default();
                        // let cors = Cors::permissive().allowed_header(header::CONTENT_TYPE);
                        let cors = Cors::default()
                            .allowed_origin("https://v4-rust.savaged.us")
                            .allowed_origin("https://savaged.us")
                            .allowed_origin("http://localhost")
                            .allowed_origin("http://127.0.0.1")
                            .allowed_origin("http://localhost:8080")
                            .allowed_origin("http://127.0.0.1:8080")
                            .allowed_origin("http://localhost:5001")
                            .allowed_origin("http://127.0.0.1:5001")
                            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                            .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                            .allowed_header(header::CONTENT_TYPE)
                            .max_age(3600);

                        App::new()
                            // .wrap( session_middleware )
                            .wrap(
                                SessionMiddleware::builder(
                                    CookieSessionStore::default(),
                                    cookie_secret_key.clone()
                                )// .cookie_secure(true)
                                 .cookie_path("/".to_owned())
                                 .session_lifecycle(
                                    PersistentSession::default()
                                        .session_ttl(time::Duration::days(365))
                                )
                                 .cookie_name("savaged_session".to_owned())
                                //  .cookie_content_security(CookieContentSecurity::Private)
                                //  .cookie_same_site(actix_web::cookie::SameSite::None)
                                 .build(),
                            )
                            .wrap( logger )
                            .wrap( cors )

                            .app_data(Data::new(mysql_connection_pool.clone()))
                            .app_data(Data::new(chat_server.clone()))

                            .service(web_socket_router)

                            // Authentication Handlers
                            .service(api_auth_login_for_token)
                            .service(api_auth_login)
                            .service(api_auth_get_user_data)
                            .service( api_logout )

                            // Saves Handlers
                            .service(api_saves_get)

                            // User Settings
                            .service(api_user_token_remove)
                            .service(api_user_token_update_name)
                            .service(api_user_update_settings)
                            .service(api_user_save_username)
                            .service(api_user_username_available)
                            .service(api_user_set_user_image_data)

                            // User Notification Page Handlers
                            .service(api_notifications_set_deleted)
                            .service(api_notifications_set_read)
                            .service(api_notifications_get)
                            .service(api_notifications_delete_basic_admin)
                            .service(api_notifications_set_all_read)

                            // Data Endpoints
                            // .service( hindrances_get )
                            // .service( books_get )
                            .service(api_game_data_get)
                            // get banners API
                            .service(api_banners_get)
                            // admin API
                            .service(api_admin_users_get)
                            .service(api_admin_users_paging)
                            .service(api_admin_game_data_get)
                            .service(api_admin_game_data_paging)
                            .service(api_admin_game_data_save)
                            .service(api_admin_game_data_delete)

                            // render yew app SSR.
                            .service(
                                actix_web::web::resource(
                                    ["/"], // see above for routes which will render via SSR
                                )
                                .route(actix_web::web::get().to(yew_render)),
                            )
                            // serve user images...
                            .service(
                                fs::Files::new("/data-images/", "./data/uploads/")
                                    .use_last_modified(true),
                            )
                            // serve other file system files...
                            .service(fs::Files::new("/", "./public").use_last_modified(true))
                            .default_service(actix_web::web::get().to(yew_render))
                    })
                    .bind((serve_ip, serve_port))?
                    .run()
                    .await
                // }
                // Err(err) => {
                //     println!("MysqL Pool Error 2 {}", err);
                //     std::process::exit(0x0100);
                // }
            // }
        }
        Err(err) => {
            println!("MysqL Pool Error 1 {}", err);
            std::process::exit(0x0100);
        }
    }
}

async fn yew_render(
    pool: Data<Pool>,
    request: HttpRequest,
    session: Session,
    // payload: &mut actix_web::dev::Payload,
) -> HttpResponse {
    let url = request.uri().to_string();

    // let mut payload = actix_web::dev::Payload::None;
    // let s = actix_session::Session::from_request(&request, &mut payload);
    // let session = s.into_inner().unwrap();
    let mut session_user_id: u32 = 0;

    let session_result= session.get::<u32>("user_id");

    match session_result {
        Ok( user_id_option ) => {
            match user_id_option {
                Some( user_id ) => {
                    println!("yew_render SESSION value: {}", user_id);
                    session_user_id = user_id;
                }
                None => {
                    let _ = session.insert("user_id", 0);
                }
            }

        }
        Err( err ) => {
            println!("yew_render Session Error {}", err);
            // let _ = session.insert("user_id", 0);
        }
    }

    let user = get_user( &pool, session_user_id );
    let mut web_content: WebContent = get_web_content(&pool).await;

    web_content.user = get_user(
        &pool,
        session_user_id
    ).await;

    let content = spawn_blocking(move || {
        use tokio::runtime::Builder;
        let set = LocalSet::new();

        let rt = Builder::new_current_thread().enable_all().build().unwrap();

        set.block_on(&rt, async {
            // let server_renderer = ServerRenderer::<ServerApp>::new();
            // let url = url.to_owned();



            let server_renderer = ServerRenderer::<ServerApp>::with_props(move || {



                // println!("yew_render session_user_id {:?}", session_user_id);
                // println!("yew_render user {:?}", &web_content.user);
                // web_content.user = user;

                ServerAppProps {
                    url: AttrValue::from(url.clone()),
                    web_content: web_content,
                }
            });

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

    // println!("yew_render pre-rendering");
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(format!(
            "{}{}{}",
            index_html_before, content, index_html_after,
        ))
}
