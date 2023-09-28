use actix_session::Session;
use mysql_async::Pool;

use actix_web::{post, web::Data, web::Json, HttpRequest};
use savaged_libs::{
    admin_libs::{AdminPagingStatistics, FetchAdminParameters},
    user::User,
};

use crate::db::users::{admin_get_users, admin_get_users_paging_data, get_remote_user};

#[post("/_api/admin/users/get")]
pub async fn api_admin_users_get(
    pool: Data<Pool>,
    form: Json<FetchAdminParameters>,
    request: HttpRequest,
    session: Session,
) -> Json<Vec<User>> {
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
    let user_option = get_remote_user(&pool, api_key, login_token, request, session).await;

    match user_option {
        Some(user) => {
            if user.has_admin_access() {
                return Json(admin_get_users(&pool, form).await);
            }
        }
        None => {}
    }

    return Json(Vec::new());
}

#[post("/_api/admin/users/paging")]
pub async fn api_admin_users_paging(
    pool: Data<Pool>,
    form: Json<FetchAdminParameters>,
    request: HttpRequest,
    session: Session,
) -> Json<AdminPagingStatistics> {
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
    let user_option = get_remote_user(&pool, api_key, login_token, request, session).await;

    match user_option {
        Some(user) => {
            if user.has_admin_access() {
                return Json(admin_get_users_paging_data(&pool, form).await);
            }
        }
        None => {}
    }

    return Json(AdminPagingStatistics {
        non_filtered_count: 0,
        filtered_count: 0,
        book_list: None,
    });
}
