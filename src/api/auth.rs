use actix_session::Session;
use actix_web::FromRequest;
use chrono::DateTime;
use chrono::Utc;
// use mysql_async::*;
// use mysql_async::prelude::*;
use mysql_async::Pool;
use savaged_libs::save_db_row::SaveDBRow;
use std::fs;
use std::path;
// use chrono::DateTime;
// use chrono::Utc;
// use std::env;
// use chrono::prelude::*;
// use chrono_tz::US::Central;
// use actix_multipart::Multipart;
// use chrono_tz::Tz;
use crate::db::saves::get_user_saves;
use crate::utils::encrypt_password;
use actix_web::HttpRequest;
use actix_web::{
    // get,
    post,
    web::Data,
    // web::Form,
    // HttpResponse,
    // http::{header::ContentType, StatusCode }
    // put,
    // error::ResponseError,
    // web::Path,
    // web,
    web::Json,
};

use super::super::db::users::{
    create_login_token, get_remote_user, get_user, log_user_in, update_user_login_tokens,
};

// use sha2::{Sha256, Sha512, Sha224, Digest};
// use serde_json::Error;
use savaged_libs::user::{LoginToken, LoginTokenResult, User, UserUpdateResult};
use serde::{Deserialize, Serialize};
// use base64;
// use derive_more::{Display};

// #[get("/_api/auth/user-groups")]
// pub async fn auth_get_user_groups(
//     pool: Data<Pool>,
//     // task_gid: Path<String>,
//     // body: Json<Struct>,
// ) -> Json<Vec<UserGroup>> {

//     match pool.get_conn().await {
//         Ok( mut conn) => {
//             let selected_payments_result = conn
//             .query_map(
//                 "SELECT id, name from user_groups",
//                 |(id, name,)| {
//                     UserGroup { id, name, }
//                 },
//             );
//             match selected_payments_result {
//                 Ok( selected_payments ) => {
//                     return actix_web::web::Json( selected_payments );
//                 }

//                 Err( err ) => {
//                     println!("get_user_groups Error 4 {}", err );
//                 }
//             }
//         }
//         Err( err ) => {
//             println!("get_user_groups Error 3 {}", err );
//         }
//     }

//     return Json( Vec::new() );
// }

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LoginForm {
    email: String,
    password: String,
}

#[post("/_api/auth/login-for-token")]
pub async fn api_auth_login_for_token(
    pool: Data<Pool>,
    form: Json<LoginForm>,
    request: HttpRequest,
) -> Json<LoginTokenResult> {
    let conn_info = request.connection_info();

    let mut real_remote_addy = "".to_string();
    let mut user_agent = "".to_string();
    let mut x_forwarded_for = "".to_string();

    let real_remote_addy_option = conn_info.realip_remote_addr();
    match real_remote_addy_option {
        Some(val) => {
            real_remote_addy = val.to_string();
        }
        None => {}
    }

    let user_agent_option = request.headers().get("user-agent");
    match user_agent_option {
        Some(val) => {
            user_agent = format!("{:?}", val).to_string().replace("\"", "");
        }
        None => {}
    }

    let x_forwarded_for_option = request.headers().get("x-forwarded-for");
    match x_forwarded_for_option {
        Some(val) => {
            x_forwarded_for = format!("{:?}", val).to_string().replace("\"", "");
        }
        None => {}
    }

    // println!("real_remote_addy {}", real_remote_addy);
    // println!("user_agent {}", user_agent);
    // println!("x_forwarded_for {}", x_forwarded_for);

    if !x_forwarded_for.is_empty() {
        real_remote_addy = x_forwarded_for;
    }

    let mut rv = LoginTokenResult {
        success: false,
        active_notifications: 0,
        user: User::default(),
        user_groups: Vec::new(),
        login_token: "".to_owned(),
        last_seen: None,
        registered: None,
    };

    let login_results = log_user_in(
        &pool,
        form.email.to_owned(),
        form.password.to_owned(),
    ).await;

    if login_results.user_id > 0 {
        let new_login_token = create_login_token(
            &pool,
            login_results.user_id,
            user_agent.to_owned(),
            real_remote_addy.to_owned(),
        ).await
        .unwrap();
        let user_result = get_user(&pool, login_results.user_id).await;
        match user_result {
            Some(user) => {
                rv.success = true;
                rv.login_token = new_login_token;
                rv.user = user.clone();
                rv.user.get_image("");
                rv.registered = user.created_on.clone();
            }
            None => {}
        }
    }

    return Json(rv);

}

#[post("/_api/auth/login")]
pub async fn api_auth_login(
    pool: Data<Pool>,
    form: Json<LoginForm>,
    request: HttpRequest,
    session: Session,
    // payload: mut actix_web::dev::Payload,
) -> Json<LoginTokenResult> {
    let conn_info = request.connection_info();

    // let mut payload = actix_web::dev::Payload::None;
    // let s = actix_session::Session::from_request(&request, &mut payload);

    // let session = s.into_inner().unwrap();
    // println!("api_auth_login session.entries {:?}", session.entries());
    let mut real_remote_addy = "".to_string();
    let mut user_agent = "".to_string();
    let mut x_forwarded_for = "".to_string();

    let real_remote_addy_option = conn_info.realip_remote_addr();
    match real_remote_addy_option {
        Some(val) => {
            real_remote_addy = val.to_string();
        }
        None => {}
    }

    let user_agent_option = request.headers().get("user-agent");
    match user_agent_option {
        Some(val) => {
            user_agent = format!("{:?}", val).to_string().replace("\"", "");
        }
        None => {}
    }

    let x_forwarded_for_option = request.headers().get("x-forwarded-for");
    match x_forwarded_for_option {
        Some(val) => {
            x_forwarded_for = format!("{:?}", val).to_string().replace("\"", "");
        }
        None => {}
    }

    // println!("real_remote_addy {}", real_remote_addy);
    // println!("user_agent {}", user_agent);
    // println!("x_forwarded_for {}", x_forwarded_for);

    if !x_forwarded_for.is_empty() {
        real_remote_addy = x_forwarded_for;
    }

    let mut rv = LoginTokenResult {
        success: false,
        active_notifications: 0,
        user: User::default(),
        user_groups: Vec::new(),
        login_token: "".to_owned(),
        last_seen: None,
        registered: None,
    };

    let login_results = log_user_in(
        &pool,
        form.email.to_owned(),
        form.password.to_owned(),
    ).await;

    if login_results.user_id > 0 {
        let session_result = session.insert("user_id", login_results.user_id);
        match session_result {
            Ok(_) => {
                // println!("api_auth_login Session ID set {}", login_results.user_id);
            }
            Err(err) => {
                // println!("api_auth_login error setting session user {:?}", err);
                let _ = session.insert("user_id", 0);
            }
        }

        let user_result = get_user(&pool, login_results.user_id).await;
        match user_result {
            Some(user) => {
                // println!("api_auth_login Session ID set 2 {}", user.id);
                rv.success = true;
                rv.login_token = "".to_owned();
                rv.user = user.clone();
                rv.user.get_image("");
                rv.registered = user.created_on.clone();
            }
            None => {
                let _ = session.insert("user_id", 0);
            }
        }

    }
    // println!("api_auth_login session.entries #2 {:?}", session.entries());
    return Json(rv);

}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ApiKeyOrToken {
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub login_token: Option<String>,

    #[serde(default)]
    pub last_updated: Option<DateTime<Utc>>,
}

#[post("/_api/auth/logout")]
pub async fn api_logout(
    _pool: Data<Pool>,
    _form: Json<ApiKeyOrToken>,
    _request: HttpRequest,
    session: Session,
) -> Json<bool> {
    let _ = session.insert("user_id", 0);
    return Json(true);
}

#[post("/_api/auth/get-user-data")]
pub async fn api_auth_get_user_data(
    pool: Data<Pool>,
    form: Json<ApiKeyOrToken>,
    request: HttpRequest,
    session: Session,
) -> Json<Option<User>> {
    let mut login_token: Option<String> = None;
    let mut api_key: Option<String> = None;

    let session_result= session.get::<u32>("user_id");

    // println!("XXXX api_auth_get_user_data {:?}", session_result );

    match session_result {
        Ok( user_id_option ) => {
            match user_id_option {
                Some( user_id ) => {
                    // println!("api_auth_get_user_data SESSION user_id: {}", user_id);
                    // session_user_id = user_id;
                    // session.insert("user_id", login_results.user_id);
                    return Json(get_user(&pool, user_id).await);
                }
                None => {
                    // session.insert("user_id", login_results.user_id);
                }
            }

        }
        Err( err ) => {
            println!("Session Error {}", err);
        }
    }

    match &form.login_token {
        Some(val) => {
            login_token = Some(val.to_owned());
        }
        None => {}
    }
    match &form.api_key {
        Some(val) => {
            api_key = Some(val.to_owned());
        }
        None => {}
    }

    return Json(get_remote_user(&pool, api_key, login_token, request, session).await);
}
