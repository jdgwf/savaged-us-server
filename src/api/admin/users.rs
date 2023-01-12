use mysql::Pool;

use actix_web:: {
    HttpRequest,
    post,
    web::Json,
    web::Data,

};
use savaged_libs::{
    admin_libs::{FetchAdminParameters, AdminPagingStatistics},
    user::User
};

use crate::db::users::{
    admin_get_users, get_remote_user, admin_get_users_paging_data,
};

#[post("/_api/admin/users/get")]
pub async fn api_admin_users_get(
    pool: Data<Pool>,
    form: Json<FetchAdminParameters>,
    request: HttpRequest,
) -> Json<Vec<User>> {

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
    let user_option = get_remote_user(
        pool.clone(),
        api_key,
        login_token,
        request,
    );

    match user_option {
        Some( user ) => {
            if user.has_admin_access() {
                return Json(admin_get_users( pool, form ));
            }
        }
        None => {}
    }

    return Json( Vec::new() );
}




#[post("/_api/admin/users/paging")]
pub async fn api_admin_users_paging(
    pool: Data<Pool>,
    form: Json<FetchAdminParameters>,
    request: HttpRequest,
) -> Json<AdminPagingStatistics> {

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
    let user_option = get_remote_user(
        pool.clone(),
        api_key,
        login_token,
        request,
    );

    match user_option {
        Some( user ) => {
            if user.has_admin_access() {
                return Json(admin_get_users_paging_data( pool, form ));
            }
        }
        None => {}
    }

    return Json(
        AdminPagingStatistics {
            non_filtered_count: 0,
            filtered_count: 0,
        }
     );
}
