// use mysql::*;
// use mysql::prelude::*;
use mysql::Pool;
use std::fs;
use std::path;
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
use crate::utils::encrypt_password;

use super::super::db::users::{
    log_user_in,
    get_user,
    create_login_token,
    get_remote_user,
    update_user_login_tokens,
};

// use sha2::{Sha256, Sha512, Sha224, Digest};
// use serde_json::Error;
use serde::{Serialize, Deserialize};
use savaged_libs::user::{ LoginTokenResult, User, LoginToken, UserUpdateResult };
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

    let login_results = log_user_in( pool.clone(), form.email.to_owned(), form.password.to_owned() );

    if login_results.user_id > 0 {
        let new_login_token = create_login_token(
            pool.clone(),
            login_results.user_id,
            "browser".to_owned(),
            "ip".to_owned(),
        ).unwrap();
        let user_result = get_user( pool.clone(), login_results.user_id);
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

    let mut login_token: Option<String> = None;
    let mut api_key: Option<String> = None;
    match &form.login_token {
        Some( val ) => {
            login_token = Some(val.to_owned());
        }
        None => {}
    }
    match &form.api_key {
        Some( val ) => {
            api_key = Some(val.to_owned());
        }
        None => {}
    }
    return Json( get_remote_user(
        pool.clone(),
        api_key,
        login_token,
        request,
    ) );

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

    let mut login_token: Option<String> = None;
    let mut api_key: Option<String> = None;
    let mut selected_token = "".to_owned();
    let mut new_value = "".to_owned();
    match &form.login_token {
        Some( val ) => {
            login_token = Some(val.to_owned());
        }
        None => {}
    }
    match &form.api_key {
        Some( val ) => {
            api_key = Some(val.to_owned());
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
    );

    match user_option {
        Some( user ) => {

            let mut return_tokens = user.login_tokens.clone();

            for token in &mut return_tokens {
                if token.token == selected_token {
                    token.friendly_name = new_value.to_owned();
                }
            }

            update_user_login_tokens( pool.clone(), user.id, return_tokens.clone() );

            return Json( return_tokens.clone() );
        }
        None => {

        }
    }

    return Json( Vec::new() );
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UpdateSettingData {
    api_key: Option<String>,
    login_token: Option<String>,
    password: Option<String>,
    repeat_password: Option<String>,
    remove_image: bool,
    current_user: String,
}

#[post("/_api/auth/update-settings")]
pub async fn auth_update_settings(
    pool: Data<Pool>,
    form: Json<UpdateSettingData>,
    request: HttpRequest,
) -> Json< UserUpdateResult > {

    let mut return_value: UserUpdateResult = UserUpdateResult {
        success: false,
        password_changed: false,
        message: "".to_string(),
    };

    let mut login_token: Option<String> = None;
    let mut api_key: Option<String> = None;

    // let mut new_value = "".to_owned();
    match &form.login_token {
        Some( val ) => {
            login_token = Some(val.to_owned());
        }
        None => {}
    }
    match &form.api_key {
        Some( val ) => {
            api_key = Some(val.to_owned());
        }
        None => {}
    }
    // match &form.selected_token {
    //     Some( val ) => {
    //         selected_token = val.to_owned();
    //     }
    //     None => {}
    // }

    let user_option = get_remote_user(
        pool.clone(),
        api_key,
        login_token,
        request,
    );

    match user_option {
        Some( user ) => {

            let user_data: Result<User, serde_json::Error> = serde_json::from_str( &form.current_user );
            match user_data {
                Ok(mut user_settings) => {
                    println!("auth_update_settings() user found!");
                    println!("auth_update_settings() user_data {:?}", user_settings);
                    println!("auth_update_settings() form.password {:?}", form.password);
                    println!("auth_update_settings() form.repeat_password {:?}", form.repeat_password);
                    println!("auth_update_settings() form.remove_image {:?}", form.remove_image);

                                // Override any potential hacker variables in POST
                    user_settings.is_premium = user.is_premium;
                    user_settings.is_ace = user.is_ace;
                    user_settings.is_admin = user.is_admin;
                    user_settings.is_developer = user.is_developer;
                    user_settings.id = user.id;
                    user_settings.lc_wildcard_reason = user.lc_wildcard_reason;
                    user_settings.premium_expires = user.premium_expires;
                    user_settings.last_seen_ip = user.last_seen_ip;
                    user_settings.last_seen_on = user.last_seen_on;


                    let data_dir_path = "./data/uploads/";
                    let png_filename = data_dir_path.to_owned() + &"users/".to_owned() + &user_settings.id.to_string()  + &".png".to_owned();
                    let jpg_filename = data_dir_path.to_owned() + &"users/".to_owned() + &user_settings.id.to_string()  + &".jpg".to_owned();
                    let webp_filename = data_dir_path.to_owned() + &"users/".to_owned() + &user_settings.id.to_string()  + &".webp".to_owned();

                    if form.remove_image {
                        if std::path::Path::new(&png_filename).exists() {
                            fs::remove_file(&png_filename);
                        }
                        if std::path::Path::new(&jpg_filename).exists() {
                            fs::remove_file(&jpg_filename);
                        }
                        if std::path::Path::new(&webp_filename).exists() {
                            fs::remove_file(&webp_filename);
                        }
                        user_settings.profile_image = "".to_string();
                    }
                    if std::path::Path::new(&png_filename).exists() {
                        user_settings.profile_image = "png".to_string();
                    }
                    if std::path::Path::new(&jpg_filename).exists() {
                        user_settings.profile_image = "jpg".to_string();
                    }
                    if std::path::Path::new(&webp_filename).exists() {
                        user_settings.profile_image = "webp".to_string();
                    }

                    let mut do_notify_admins = false;
                    if !user_settings.activated {
                        do_notify_admins = true;
                    }


                    if !user_settings.email.is_empty() {
                        let mut new_encrypted_pass = "".to_string();
                        if
                            form.password != None && !form.password.as_ref().unwrap().is_empty()
                            && form.repeat_password != None && !form.repeat_password.as_ref().unwrap().is_empty()
                            && form.repeat_password.as_ref() == form.password.as_ref()
                        {
                            return_value.password_changed = true;
                            new_encrypted_pass = encrypt_password( form.password.clone().unwrap().to_owned() );
                        }



                    } else {
                        return_value.message = "Email Address cannot be empty - this might be a data transfer error.".to_string();
                    }
                }
                Err( err ) => {

                }

            }
        }
        None => {

        }
    }

    return Json( return_value );
}

#[post("/_api/auth/token-remove")]
pub async fn auth_token_remove(
    pool: Data<Pool>,
    form: Json<UpdateTokenNameForm>,
    request: HttpRequest,
) -> Json< Vec<LoginToken> > {

    let mut login_token: Option<String> = None;
    let mut api_key: Option<String> = None;
    let mut selected_token = "".to_owned();
    // let mut new_value = "".to_owned();
    match &form.login_token {
        Some( val ) => {
            login_token = Some(val.to_owned());
        }
        None => {}
    }
    match &form.api_key {
        Some( val ) => {
            api_key = Some(val.to_owned());
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
    );

    match user_option {
        Some( user ) => {

            let mut return_tokens: Vec<LoginToken> = Vec::new();

            for token in user.login_tokens.iter() {
                if token.token != selected_token {
                    return_tokens.push( token.clone() );
                }
            }

            update_user_login_tokens( pool.clone(), user.id, return_tokens.clone() );

            return Json( return_tokens.clone() );
        }
        None => {

        }
    }

    return Json( Vec::new() );
}