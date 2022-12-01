use mysql::*;
use mysql::prelude::*;

use crate::db::utils::mysql_datetime_to_chrono_utc;
use actix_web:: {
    post,
    web::Json,
    web::Data,
};

use super::super::db::users::{
    get_remote_user,
};

use serde::{Serialize, Deserialize};
use savaged_libs::notification::{ Notification };
use actix_web::HttpRequest;
use super::auth::ApiKeyOrToken;

#[post("/_api/auth/notifications-get")]
pub async fn notifications_get(
    pool: Data<Pool>,
    form: Json<ApiKeyOrToken>,
    request: HttpRequest,
) -> Json<Vec<Notification>> {
    // println!("notifications_get");
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

    let current_user = get_remote_user(
        pool.clone(),
        api_key,
        login_token,
        request,
    ).await;

    match current_user {
        Some( user ) => {

            return Json( _get_notifications_for_user( pool.clone(), user.id ).await );
        }
        None => {
            return Json( Vec::new() );
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct NotificationForm {
    pub api_key: Option<String>,
    pub login_token: Option<String>,
    pub notification_id: Option<String>,
    pub read: Option<String>,
}

#[post("/_api/auth/notifications-set-deleted")]
pub async fn notifications_set_deleted(
    pool: Data<Pool>,
    form: Json<NotificationForm>,
    request: HttpRequest,
) -> Json<Vec<Notification>> {
    // println!("notifications_set_deleted");
    let mut login_token = "".to_owned();
    let mut api_key = "".to_owned();
    let mut notification_id: u32 = 0;
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

    match &form.notification_id {
        Some( val ) => {
            // notification_id = val.parse();
            notification_id = val.parse::<u32>().unwrap();
        }
        None => {}
    }

    let current_user = get_remote_user(
        pool.clone(),
        api_key,
        login_token,
        request,
    ).await;

    match current_user {
        Some( user ) => {

            // println!("notifications_set_deleted notification_id: {}", notification_id);
            match pool.get_conn() {
                Ok( mut conn) => {
                    let notifications_result: Option<Row>  = conn.exec_first(
                        "update `user_notifications`
                            set `deleted` = 1
                            where `user_id` = :user_id
                            and `id` = :notification_id

                            limit 1
                        ",
                        params!{
                            "user_id" => user.id,
                            "notification_id" => notification_id,
                        }
                    ).unwrap();
                    match notifications_result {
                        Some(_ ) => {
                            return Json( _get_notifications_for_user( pool.clone(), user.id ).await );
                        }

                        None => {
                            // println!("notifications_get Error 4 {}", err );
                            return Json( _get_notifications_for_user( pool.clone(), user.id ).await );
                        }
                    }
                }
                Err( err ) => {
                    println!("notifications_set_deleted Error 3 {}", err );
                    return Json(Vec::new());
                }
            }
        }
        None => {
            return Json( Vec::new() );
        }
    }
}

#[post("/_api/auth/notifications-set-read")]
pub async fn notifications_set_read(
    pool: Data<Pool>,
    form: Json<NotificationForm>,
    request: HttpRequest,
) -> Json<Vec<Notification>> {
    // println!("notifications_set_read");
    let mut login_token = "".to_owned();
    let mut api_key = "".to_owned();
    let mut notification_id: u32 = 0;
    let mut read: bool = true;
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

    match &form.notification_id {
        Some( val ) => {
            // notification_id = val;
            notification_id = val.parse::<u32>().unwrap();
            // println!( "notification_id {}", val);
        }
        None => {}
    }

    match &form.read {
        Some( val ) => {
            // println!( "val {}", val);
            let my_int = val.parse::<u32>().unwrap();
            // println!( "my_int {}", val);
            if my_int > 0 {
                read = true;
            } else {
                read = false;
            }
            // println!( "read {}", read);
        }
        None => {}
    }

    let current_user = get_remote_user(
        pool.clone(),
        api_key,
        login_token,
        request,
    ).await;

    match current_user {
        Some( user ) => {

            // println!("notifications_set_read notification_id: {}", notification_id);
            // println!("notifications_set_read read: {}", read);
            match pool.get_conn() {
                Ok( mut conn) => {
                    let notifications_result: Option<Row>  = conn.exec_first(
                        "update `user_notifications`
                            set `read` = :read
                            where `user_id` = :user_id
                            and `id` = :notification_id

                            limit 1
                        ",
                        params!{
                            "user_id" => user.id,
                            "notification_id" => notification_id,
                            "read" => read,
                        }
                    ).unwrap();
                    match notifications_result {
                        Some( _ ) => {
                            return Json( _get_notifications_for_user( pool.clone(), user.id ).await );
                        }

                        None => {
                            // println!("notifications_get Error 4 {}", err );
                            return Json( _get_notifications_for_user( pool.clone(), user.id ).await );
                        }
                    }
                }
                Err( err ) => {
                    println!("notifications_set_read Error 3 {}", err );
                    return Json(Vec::new());
                }
            }

        }
        None => {
            return Json( Vec::new() );
        }
    }
}

#[post("/_api/auth/notifications-set-all-read")]
pub async fn notifications_set_all_read(
    pool: Data<Pool>,
    form: Json<NotificationForm>,
    request: HttpRequest,
) -> Json<Vec<Notification>> {
    // println!("notifications_set_read");
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

    let current_user = get_remote_user(
        pool.clone(),
        api_key,
        login_token,
        request,
    ).await;

    match current_user {
        Some( user ) => {

            match pool.get_conn() {
                Ok( mut conn) => {
                    let notifications_result: Option<Row>  = conn.exec_first(
                        "update `user_notifications`
                            set `read` = 1
                            where `user_id` = :user_id
                            and `version_of` = 0
                        ",
                        params!{
                            "user_id" => user.id,
                        }
                    ).unwrap();
                    match notifications_result {
                        Some( _ ) => {
                            return Json( _get_notifications_for_user( pool.clone(), user.id ).await );
                        }

                        None => {
                            // println!("notifications_get Error 4 {}", err );
                            return Json( _get_notifications_for_user( pool.clone(), user.id ).await );
                        }
                    }
                }
                Err( err ) => {
                    println!("notifications_set_all_read Error 3 {}", err );
                    return Json(Vec::new());
                }
            }

        }
        None => {
            return Json( Vec::new() );
        }
    }
}

async fn _get_notifications_for_user(
    pool: Data<Pool>,
    current_user_id: u32,
) -> Vec<Notification> {
    match pool.get_conn() {
        Ok( mut conn) => {
            let notifications_result = conn
            .query_map(
                format!("SELECT
                    `id`,
                    `user_id`,
                    `read`,
                    `subject`,
                    `message`,
                    `created_by`,
                    `created_on`
                 from `user_notifications` where `user_id` = {} and `deleted` < 1
                 order by created_on desc

                 ", current_user_id),
                |(
                    id,
                    user_id,
                    read,
                    subject,
                    message,
                    created_by,
                    created_on,
                )| {

                    let created_on_string: String = created_on;
                    // let mut dt = DateTime::<Utc>::default();
                    // let dt_utc = DateTime::parse_from_rfc3339( date_string.as_ref() );
                    // let utc_dt: DateTime<Utc> = Utc::from( dt );
                    Notification {
                        id: id,
                        user_id: user_id,
                        read: read,
                        subject: subject,
                        message: message,
                        created_by: created_by,
                        created_on: mysql_datetime_to_chrono_utc(created_on_string),
                    }
                },
            );
            match notifications_result {
                Ok( notifications ) => {
                    return notifications;
                }

                Err( err ) => {
                    println!("_get_notifications_for_user Error 4 {}", err );
                    return Vec::new();
                }
            }
        }
        Err( err ) => {
            println!("_get_notifications_for_user Error 3 {}", err );
            return Vec::new();
        }
    }
}

#[post("/_api/auth/notifications-delete-basic-admin")]
pub async fn notifications_delete_basic_admin(
    pool: Data<Pool>,
    form: Json<NotificationForm>,
    request: HttpRequest,
) -> Json<Vec<Notification>> {
    // println!("notifications_set_read");
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

    let current_user = get_remote_user(
        pool.clone(),
        api_key,
        login_token,
        request,
    ).await;

    match current_user {
        Some( user ) => {

            match pool.get_conn() {
                Ok( mut conn) => {
                    let notifications_result: Option<Row>  = conn.exec_first(
                        "update `user_notifications`
                            set `deleted` = 1,
                            `updated_on` = now(),
                            `updated_by` = :user_id
                            where `user_id` = :user_id
                            and `deleted` = 0
                            and `version_of` = 0
                            and (
                                `subject` like '%New User Activation%'
                                or
                                `subject` like '%Accounting Process%'
                                or
                                `subject` like '%API Key Set/Change%'
                                or
                                `subject` like '%Password Reset%'
                            )
                        ",
                        params!{
                            "user_id" => user.id,
                        }
                    ).unwrap();
                    match notifications_result {
                        Some( _ ) => {
                            return Json( _get_notifications_for_user( pool.clone(), user.id ).await );
                        }

                        None => {
                            // println!("notifications_get Error 4 {}", err );
                            return Json( _get_notifications_for_user( pool.clone(), user.id ).await );
                        }
                    }
                }
                Err( err ) => {
                    println!("notifications_delete_basic_admin Error 3 {}", err );
                    return Json(Vec::new());
                }
            }

        }
        None => {
            return Json( Vec::new() );
        }
    }
}
