use mysql::*;
// use mysql::prelude::*;

// use chrono::DateTime;
// use chrono::Utc;
// use std::env;
// use chrono::prelude::*;
// use chrono_tz::US::Central;
// use actix_multipart::Multipart;
// use chrono_tz::Tz;
use actix_web:: {
    // get,
    post,
    // put,
    // error::ResponseError,
    // web::Path,
    // web,
    web::Json,
    web::Data,
    // web::Form,
    // HttpResponse,
    // http::{header::ContentType, StatusCode }
};
use actix_web::HttpRequest;
use super::super::db::users::{
    log_user_in,
    get_user,
    create_login_token,
    get_remote_user,
    update_user_login_tokens,
};

// use sha2::{Sha256, Sha512, Sha224, Digest};
// use serde_json;
use serde::{Serialize, Deserialize};
use savaged_libs::user::{ LoginTokenResult, User, LoginToken };
// use base64;
// use derive_more::{Display};

// #[get("/_api/auth/user-groups")]
// pub async fn auth_get_user_groups(
//     pool: Data<Pool>,
//     // task_gid: Path<String>,
//     // body: Json<Struct>,
// ) -> Json<Vec<UserGroup>> {

//     match pool.get_conn() {
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LoginForm {
    email: String,
    password: String,
}

#[post("/_api/auth/login-for-token")]
pub async fn auth_api_login_for_token(
    pool: Data<Pool>,
    form: Json<LoginForm>,
) -> Json<LoginTokenResult> {

    let mut rv = LoginTokenResult {
        success: false,
        active_notifications: 0,
        user : User::default(),
        user_groups: Vec::new(),
        login_token: "".to_owned(),
        last_seen: None,
        registered: None,
    };

    let login_results = log_user_in( pool.clone(), form.email.to_owned(), form.password.to_owned() ).await;

    if login_results.user_id > 0 {
        let new_login_token = create_login_token(
            pool.clone(),
            login_results.user_id,
            "browser".to_owned(),
            "ip".to_owned(),
        ).await.unwrap();
        let user_result = get_user( pool.clone(), login_results.user_id).await;
        match user_result {
            Some( user ) => {
                rv.success = true;
                rv.login_token = new_login_token;
                rv.user = user.clone();
                rv.user.get_image("");
                rv.registered = user.created_on.clone();
            }
            None => {

            }
        }

    }

    return Json( rv );
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ApiKeyOrToken {
    pub api_key: Option<String>,
    pub login_token: Option<String>,
}

#[post("/_api/auth/get-user-data")]
pub async fn auth_get_user_data(
    pool: Data<Pool>,
    form: Json<ApiKeyOrToken>,
    request: HttpRequest,
) -> Json< Option<User> > {

    let mut login_token = "".to_owned();
    let mut api_key = "".to_owned();
    match &form.login_token {
        Some( val ) => {
            login_token = val.to_owned();
        }
        None => {}
    }
    match &form.api_key {
        Some( val ) => {
            api_key = val.to_owned();
        }
        None => {}
    }
    return Json( get_remote_user(
        pool.clone(),
        api_key,
        login_token,
        request,
    ).await );

}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UpdateTokenNameForm {
    api_key: Option<String>,
    login_token: Option<String>,
    selected_token: Option<String>,
    new_value: Option<String>,
}

#[post("/_api/auth/token-update-name")]
pub async fn auth_token_update_name(
    pool: Data<Pool>,
    form: Json<UpdateTokenNameForm>,
    request: HttpRequest,
) -> Json< Vec<LoginToken> > {

    let mut login_token = "".to_owned();
    let mut api_key = "".to_owned();
    let mut selected_token = "".to_owned();
    let mut new_value = "".to_owned();
    match &form.login_token {
        Some( val ) => {
            login_token = val.to_owned();
        }
        None => {}
    }
    match &form.api_key {
        Some( val ) => {
            api_key = val.to_owned();
        }
        None => {}
    }
    match &form.selected_token {
        Some( val ) => {
            selected_token = val.to_owned();
        }
        None => {}
    }
    match &form.new_value {
        Some( val ) => {
            new_value = val.to_owned();
        }
        None => {}
    }

    let user_option = get_remote_user(
        pool.clone(),
        api_key,
        login_token,
        request,
    ).await;

    match user_option {
        Some( user ) => {

            let mut return_tokens = user.login_tokens.clone();

            for token in &mut return_tokens {
                if token.token == selected_token {
                    token.friendly_name = new_value.to_owned();
                }
            }

            update_user_login_tokens( pool.clone(), user.id, return_tokens.clone() ).await;

            return Json( return_tokens.clone() );
        }
        None => {

        }
    }

    return Json( Vec::new() );
}

#[post("/_api/auth/token-remove")]
pub async fn auth_token_remove(
    pool: Data<Pool>,
    form: Json<UpdateTokenNameForm>,
    request: HttpRequest,
) -> Json< Vec<LoginToken> > {

    let mut login_token = "".to_owned();
    let mut api_key = "".to_owned();
    let mut selected_token = "".to_owned();
    // let mut new_value = "".to_owned();
    match &form.login_token {
        Some( val ) => {
            login_token = val.to_owned();
        }
        None => {}
    }
    match &form.api_key {
        Some( val ) => {
            api_key = val.to_owned();
        }
        None => {}
    }
    match &form.selected_token {
        Some( val ) => {
            selected_token = val.to_owned();
        }
        None => {}
    }
    // match &form.new_value {
    //     Some( val ) => {
    //         new_value = val.to_owned();
    //     }
    //     None => {}
    // }

    let user_option = get_remote_user(
        pool.clone(),
        api_key,
        login_token,
        request,
    ).await;

    match user_option {
        Some( user ) => {

            let mut return_tokens: Vec<LoginToken> = Vec::new();

            for token in user.login_tokens.iter() {
                if token.token != selected_token {
                    return_tokens.push( token.clone() );
                }
            }

            update_user_login_tokens( pool.clone(), user.id, return_tokens.clone() ).await;

            return Json( return_tokens.clone() );
        }
        None => {

        }
    }

    return Json( Vec::new() );
}