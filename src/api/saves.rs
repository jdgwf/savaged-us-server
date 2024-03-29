use actix_session::Session;
// use mysql_async::*;
// use mysql_async::prelude::*;
use mysql_async::Pool;
use savaged_libs::save_db_row::SaveDBRow;
// use std::fs;
// use std::path;
// use chrono::DateTime;
// use chrono::Utc;
// use std::env;
// use chrono::prelude::*;
// use chrono_tz::US::Central;
// use actix_multipart::Multipart;
// use chrono_tz::Tz;
use crate::api::auth::ApiKeyOrToken;
use crate::db::saves::get_user_saves;
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
// use crate::utils::encrypt_password;

use super::super::db::users::get_remote_user;

// use sha2::{Sha256, Sha512, Sha224, Digest};
// use serde_json::Error;
// use serde::{Serialize, Deserialize};
// use savaged_libs::user::{ LoginTokenResult, User, LoginToken, UserUpdateResult };
// use base64;
// use derive_more::{Display};

#[post("/_api/saves/get")]
pub async fn api_saves_get(
    pool: Data<Pool>,
    form: Json<ApiKeyOrToken>,
    request: HttpRequest,
    session: Session,
) -> Json<Vec<SaveDBRow>> {
    let mut login_token: Option<String> = None;
    let mut api_key: Option<String> = None;
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



    println!("api_saves_get session.entries {:?}", &session.entries());
    // println!("api_key {:?}", api_key);
    // println!("login_token {:?}", login_token);

    let user_option = get_remote_user(&pool, api_key, login_token, request, session).await;

    match user_option {
        Some(user) => {
            let saves = get_user_saves(&pool, user.id, form.last_updated, false).await;
            return Json(saves);
        }

        None => {
            return Json(Vec::new());
        }
    }
}
