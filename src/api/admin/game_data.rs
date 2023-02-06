use actix_web::web::Json;
use actix_web::{
    post,
    web::{self, Data},
    HttpRequest,
};
use mysql::Pool;
use savaged_libs::alert_level::AlertLevel;
use savaged_libs::utils::string_manipulation::uppercase_first;
use savaged_libs::{
    admin_libs::{
        AdminDeletePackage, AdminPagingStatistics, AdminSavePackage, AdminSaveReturn,
        FetchAdminParameters,
    },
    game_data_row::GameDataRow,
};

use crate::db::{
    admin_data::{
        db_admin_admin_get_item, db_admin_delete_game_data, db_admin_get_game_data,
        db_admin_get_game_data_paging_data, db_admin_insert_game_data, db_admin_update_game_data,
    },
    books::get_books,
    users::get_remote_user,
};

#[post("/_api/admin/game-data/{table}/get")]
pub async fn api_admin_game_data_get(
    path: web::Path<(String,)>,
    pool: Data<Pool>,
    form: Json<FetchAdminParameters>,
    request: HttpRequest,
) -> Json<Vec<GameDataRow>> {
    let table = path.into_inner().0.to_string();
    // println!("api_admin_game_data_get table {}", table);
    // println!("api_admin_game_data_get form {:?}", form);

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
    let user_option = get_remote_user(&pool, api_key, login_token, request);

    match user_option {
        Some(user) => {
            if user.has_developer_access() {
                let game_data = db_admin_get_game_data(&pool, table, form);
                return Json(game_data);
            }
        }
        None => {}
    }

    return Json(Vec::new());
}

#[post("/_api/admin/game-data/{table}/paging")]
pub async fn api_admin_game_data_paging(
    path: web::Path<(String,)>,
    pool: Data<Pool>,
    form: Json<FetchAdminParameters>,
    request: HttpRequest,
) -> Json<AdminPagingStatistics> {
    let table = path.into_inner().0.to_string();
    // println!("api_admin_game_data_paging table {}", table);

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
    let user_option = get_remote_user(&pool, api_key, login_token, request);

    let needs_book_list = form.needs_book_list;

    match user_option {
        Some(user) => {
            if user.has_developer_access() {
                let mut val = db_admin_get_game_data_paging_data(&pool, table, form);
                if needs_book_list {
                    val.book_list = Some(get_books(&pool, 0, None, true, true, true, true, true));
                }
                return Json(val);
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

#[post("/_api/admin/game-data/{table}/save")]
pub async fn api_admin_game_data_save(
    path: web::Path<(String,)>,
    pool: Data<Pool>,
    form: Json<AdminSavePackage>,
    request: HttpRequest,
) -> Json<AdminSaveReturn> {
    let table = path.into_inner().0.to_string();
    // println!("api_admin_game_data_save table {}", table);

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
    let user_option = get_remote_user(&pool, api_key, login_token, request);

    // let needs_book_list = form.needs_book_list;
    // println!("needs_book_list {}", needs_book_list);
    match user_option {
        Some(user) => {
            if user.has_developer_access() {
                let book_list = Some(get_books(&pool, 0, None, true, true, true, true, true));

                let item_opt = db_admin_admin_get_item(&pool, table.to_owned(), form.id);

                match item_opt {
                    Some(item) => {
                        if user.admin_can_write_item(&book_list, item.created_by, item.book_id)
                            || user.admin_can_write_book(&book_list, item.book_id)
                        {
                            println!("saving {}", form.id);
                            let _affected_rows = db_admin_update_game_data(
                                &pool,
                                table.to_owned(),
                                user.id,
                                form.id,
                                form.data.clone(),
                            );

                            return Json(AdminSaveReturn {
                                rows_affected: 0,
                                level: AlertLevel::Success,
                                message: uppercase_first(table.as_str())
                                    + &" '".to_owned()
                                    + &form.name
                                    + &"' has been saved".to_owned(),
                                game_data: Some(db_admin_get_game_data(
                                    &pool,
                                    table,
                                    Json(form.fetch_parameters.clone()),
                                )),
                            });
                        } else {
                            return Json(AdminSaveReturn {
                                rows_affected: 0,
                                level: AlertLevel::Danger,
                                message: "You do not have access to save this item!".to_owned(),
                                game_data: Some(db_admin_get_game_data(
                                    &pool,
                                    table,
                                    Json(form.fetch_parameters.clone()),
                                )),
                            });
                        }
                    }
                    None => {
                        // no existing save found, likely inserting a new row
                        if user.admin_can_write_book(&book_list, form.book_id) {
                            println!("form.id  {}", form.id);
                            if form.id == 0 {
                                println!("inserting {}", form.id);
                                let _affected_rows = db_admin_insert_game_data(
                                    &pool,
                                    table.to_owned(),
                                    user.id,
                                    form.data.clone(),
                                );
                                return Json(AdminSaveReturn {
                                    rows_affected: 0,
                                    level: AlertLevel::Success,
                                    message: uppercase_first(table.as_str())
                                        + &" '".to_owned()
                                        + &form.name
                                        + &"' has been added".to_owned(),
                                    game_data: Some(db_admin_get_game_data(
                                        &pool,
                                        table,
                                        Json(form.fetch_parameters.clone()),
                                    )),
                                });
                            }
                            return Json(AdminSaveReturn {
                                rows_affected: 0,
                                level: AlertLevel::Danger,
                                message: "Cannot find item!".to_owned(),
                                game_data: Some(db_admin_get_game_data(
                                    &pool,
                                    table,
                                    Json(form.fetch_parameters.clone()),
                                )),
                            });
                        } else {
                            return Json(AdminSaveReturn {
                                rows_affected: 0,
                                level: AlertLevel::Danger,
                                message: "No access to save to book!".to_owned(),
                                game_data: Some(db_admin_get_game_data(
                                    &pool,
                                    table,
                                    Json(form.fetch_parameters.clone()),
                                )),
                            });
                        }
                    }
                }

                // rv.game_data = Some(db_admin_get_game_data( pool, table, Json(form.fetch_parameters.clone()) ));

                // return Json(rv);
            } else {
                return Json(AdminSaveReturn {
                    rows_affected: 0,
                    level: AlertLevel::Danger,
                    message: "You do not have developer access!".to_owned(),
                    game_data: None,
                });
            }
        }
        None => {
            return Json(AdminSaveReturn {
                rows_affected: 0,
                level: AlertLevel::Danger,
                message: "You could not be authenticated!".to_owned(),
                game_data: None,
            });
        }
    }
}

#[post("/_api/admin/game-data/{table}/delete")]
pub async fn api_admin_game_data_delete(
    path: web::Path<(String,)>,
    pool: Data<Pool>,
    form: Json<AdminDeletePackage>,
    request: HttpRequest,
) -> Json<AdminSaveReturn> {
    let table = path.into_inner().0.to_string();
    println!("api_admin_game_data_delete table {}", table);

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

    let user_option = get_remote_user(&pool, api_key, login_token, request);

    match user_option {
        Some(user) => {
            if user.has_developer_access() {
                let mut rv = AdminSaveReturn {
                    rows_affected: 0,
                    level: AlertLevel::Success,
                    message: "Data Deleted".to_owned(),
                    game_data: None,
                };

                let book_list = Some(get_books(&pool, 0, None, true, true, true, true, true));

                let item_opt = db_admin_admin_get_item(&pool, table.to_owned(), form.id);

                match item_opt {
                    Some(item) => {
                        if user.admin_can_delete_item(&book_list, item.created_by, item.book_id) {
                            let _affected_rows = db_admin_delete_game_data(
                                &pool,
                                table.to_owned(),
                                user.id,
                                form.id,
                            );

                            rv.message = uppercase_first(table.as_str())
                                + &" '".to_owned()
                                + &form.name
                                + &"' has been deleted.".to_owned();

                            rv.game_data = Some(db_admin_get_game_data(
                                &pool,
                                table,
                                Json(form.fetch_parameters.clone()),
                            ));

                            return Json(rv);
                        } else {
                            return Json(AdminSaveReturn {
                                rows_affected: 0,
                                level: AlertLevel::Danger,
                                message: "You do not have access to delete this item!".to_owned(),
                                game_data: Some(db_admin_get_game_data(
                                    &pool,
                                    table,
                                    Json(form.fetch_parameters.clone()),
                                )),
                            });
                        }
                    }
                    None => {
                        return Json(AdminSaveReturn {
                            rows_affected: 0,
                            level: AlertLevel::Danger,
                            message: "Cannot find item!".to_owned(),
                            game_data: Some(db_admin_get_game_data(
                                &pool,
                                table,
                                Json(form.fetch_parameters.clone()),
                            )),
                        });
                    }
                }
            } else {
                return Json(AdminSaveReturn {
                    rows_affected: 0,
                    level: AlertLevel::Danger,
                    message: "You do not have developer access!".to_owned(),
                    game_data: None,
                });
            }
        }
        None => {
            return Json(AdminSaveReturn {
                rows_affected: 0,
                level: AlertLevel::Danger,
                message: "You could not be authenticated!".to_owned(),
                game_data: None,
            });
        }
    }
}
