use mysql::Pool;

use actix_web:: {
    HttpRequest,
    post,
    web::Json,
    web::{Data, self},

};
use savaged_libs::{
    admin_libs::{FetchAdminParameters, AdminPagingStatistics}, game_data::GameData,
};

use crate::db::{users::{
    admin_get_users, get_remote_user, admin_get_users_paging_data,
}, game_data::{db_admin_get_game_data, db_admin_get_game_data_paging_data}, books::get_books_list};

#[post("/_api/admin/game-data/{table}/get")]
pub async fn api_admin_game_data_get(
    path: web::Path<(String,)>,
    pool: Data<Pool>,
    form: Json<FetchAdminParameters>,
    request: HttpRequest,
) -> Json<Vec<GameData>> {

    let table = path.into_inner().0.to_string();
    // println!("api_admin_game_data_get table {}", table);

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
    let data_option = get_remote_user(
        pool.clone(),
        api_key,
        login_token,
        request,
    );

    match data_option {
        Some( game_data ) => {
            if game_data.has_developer_access() {
                let game_data = db_admin_get_game_data( pool, table, form );
                return Json( game_data );
            }
        }
        None => {}
    }

    return Json( Vec::new() );
}




#[post("/_api/admin/game-data/{table}/paging")]
pub async fn api_admin_game_data_paging(
    path: web::Path<(String,)>,
    pool: Data<Pool>,
    form: Json<FetchAdminParameters>,
    request: HttpRequest,
) -> Json<AdminPagingStatistics> {
    let table = path.into_inner().0.to_string();
    // println!("api_admin_game_data_get table {}", table);

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
    let data_option = get_remote_user(
        pool.clone(),
        api_key,
        login_token,
        request,
    );

    match data_option {
        Some( game_data ) => {
            if game_data.has_developer_access() {
                return Json(db_admin_get_game_data_paging_data( pool, table, form ));
            }
        }
        None => {}
    }

    return Json(
        AdminPagingStatistics {
            non_filtered_count: 0,
            filtered_count: 0,
            book_list: get_books_list(&pool)
        }
     );
}
