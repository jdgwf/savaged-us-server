
use actix_web::web::Data;
use actix_web::web::Json;
use crate::db::utils::{mysql_row_to_chrono_utc, admin_filter_where_clause};
use mysql::*;
use mysql::prelude::*;
use savaged_libs::admin_libs::{FetchAdminParameters, AdminPagingStatistics};
use savaged_libs::game_data_row::GameDataRow;
use savaged_libs::public_user_info::PublicUserInfo;
use super::books::get_books;
use super::users::make_user_from_row;
use super::utils::admin_current_limit_paging_sql;

const DATA_SEARCH_FIELDS: &'static [&'static str]  = &[
    "primary`.`name",
    // "summary",
];

pub fn db_admin_delete_game_data(
    pool: Data<Pool>,
    table: String,
    user_id: u32,
    row_id: u32,
) -> u32 {

    match pool.get_conn() {
        Ok( mut conn) => {
            let delete_result: Option<Row>  = conn.exec_first(
                format!("update `chargen_{}`
                    set `deleted` = 1,
                    `deleted_by` = :user_id,
                    `deleted_on` =  now()
                    where `id` = :row_id

                    limit 1
                ", table),
                params!{
                    "user_id" => user_id,
                    "row_id" => row_id,
                    // "table" => table,
                }
            ).unwrap();
            match delete_result {
                Some(_ ) => {
                    // return Json( get_notifications_for_user( pool.clone(), user.id ) );
                    return 1;
                }

                None => {
                    println!("db_admin_delete_game_data no result?" );
                    // return Json( get_notifications_for_user( pool.clone(), user.id ) );
                    return 0;
                }
            }
        }
        Err( err ) => {
            println!("db_admin_update_game_data Error 3 {}", err );
            return 0;
        }
    }
}

pub fn db_admin_update_game_data(
    pool: Data<Pool>,
    table: String,
    user_id: u32,
    row_id: u32,
    data: String,
) -> u32 {

    match pool.get_conn() {
        Ok( mut conn) => {
            let notifications_result: Option<Row>  = conn.exec_first(
                format!("update `chargen_{}`
                    set `data` = :data,
                    `updated_by` = :user_id,
                    `updated_on` =  now()
                    where `id` = :row_id

                    limit 1
                ", table),
                params!{
                    "user_id" => user_id,
                    "row_id" => row_id,
                    "data" => data,
                    // "table" => table,
                }
            ).unwrap();
            match notifications_result {
                Some(_ ) => {
                    // return Json( get_notifications_for_user( pool.clone(), user.id ) );
                    return 1;
                }

                None => {
                    println!("db_admin_update_game_data no result?" );
                    // return Json( get_notifications_for_user( pool.clone(), user.id ) );
                    return 0;
                }
            }
        }
        Err( err ) => {
            println!("db_admin_update_game_data Error 3 {}", err );
            return 0;
        }
    }
}

pub fn db_admin_insert_game_data(
    pool: Data<Pool>,
    table: String,
    user_id: u32,
    data: String,
) -> u32 {

    match pool.get_conn() {
        Ok( mut conn) => {
            let insert_result: Option<Row>  = conn.exec_first(
                format!("insert into `chargen_{}` (
                    `data`,
                    `deleted_by`,
                    `updated_by`,
                    `created_by`,
                    `created_on`,
                    `updated_on`,
                    `deleted_on`
                ) VALUES (
                    :data,
                    :user_id,
                    :user_id,
                    :user_id,

                    now(),
                    now(),
                    now()
                )

                ", table),
                params!{
                    "user_id" => user_id,
                    "data" => data,
                    // "table" => table,
                }
            ).unwrap();
            match insert_result {
                Some(_ ) => {
                    // return Json( get_notifications_for_user( pool.clone(), user.id ) );
                    return 1;
                }

                None => {
                    println!("db_admin_insert_game_data no result?" );
                    // return Json( get_notifications_for_user( pool.clone(), user.id ) );
                    return 0;
                }
            }
        }
        Err( err ) => {
            println!("db_admin_update_game_data Error 3 {}", err );
            return 0;
        }
    }
}

pub fn db_admin_get_game_data_paging_data(
    pool: Data<Pool>,
    table: String,
    paging_params: Json<FetchAdminParameters>,
) -> AdminPagingStatistics {

    let mut paging: AdminPagingStatistics = AdminPagingStatistics {
        non_filtered_count: 0,
        filtered_count: 0,
        book_list: None,
    };

    let data_query = format!("
        SELECT count(id) as `count` from `chargen_{}`
        WHERE `deleted` < 1 and `version_of` = 0
    ", &table);

    match pool.get_conn() {
        Ok( mut conn) => {

            let saves_result: Result<Option<u32>> = conn.exec_first(
                data_query,
                // data_params,
                (),
            );
            match saves_result {
                Ok(  row_opt ) => {

                    match row_opt {
                        Some( row ) => {
                            paging.non_filtered_count = row;
                        }
                        None => {},
                    }

                }
                Err( err ) => {
                    println!("get_item_saves Error 4 {}", err );
                }

            }

        }
        Err( _err ) => {
            // println!("get_item_saves Error 3 {}", err );
        }
    }
    let mut data_query = format!("
        SELECT count(id) as `count` from `chargen_{}`
        WHERE `deleted` < 1 and `version_of` = 0
    ", &table);

    data_query = data_query + admin_filter_where_clause(
        DATA_SEARCH_FIELDS,
        &paging_params,
        true,
        true,
    ).as_str();
//
    // println!("admin_get_game_data_paging_data 2 data_query:\n{}", data_query);

    match pool.get_conn() {
        Ok( mut conn) => {

            let saves_result: Result<Option<u32>> = conn.exec_first(
                data_query,
                // data_params,
                (),
            );
            match saves_result {
                Ok(  row_opt ) => {

                    match row_opt {
                        Some( row ) => {
                            paging.filtered_count = row;
                        }
                        None => {},
                    }

                    // let mut rv: Vec<User> = Vec::new();
                    // for row in rows {
                    //     // let row_data = _make_save_from_row(row, with_cached_data);

                    //     // if (&row_data.name).to_owned() == "Chi Master".to_owned() {
                    //     //     println!("row_data {:?}", row_data );
                    //     // }

                    //     let mut item = _make_item_from_row( row );
                    //     rv.push( item );
                    // }

                    // paging

                    if paging_params.needs_book_list {
                        paging.book_list = Some(get_books(&pool, 0, None, false, false, false, false, true));
                    }

                }
                Err( err ) => {
                    println!("get_item_saves Error 4 {}", err );
                    // return Vec::new();
                }

            }

        }
        Err( _err ) => {
            // println!("get_item_saves Error 3 {}", err );
        }
    }

    return paging;
}

pub fn db_admin_get_game_data(
    pool: Data<Pool>,
    table: String,
    paging_params: Json<FetchAdminParameters>,
) -> Vec<GameDataRow> {

    let mut data_query = format!("
        SELECT
        `primary`.`name` as `primary_name`,
        `primary`.`book_id` as `primary_book_id`,
        `primary`.`active` as `primary_active`,
        `primary`.`id` as `primary_id`,
        `primary`.`data` as `primary_data`,
        `primary`.`version_of` as `primary_version_of`,
        `primary`.`deleted_by` as `primary_deleted_by`,
        `primary`.`deleted_on` as `primary_deleted_on`,
        `primary`.`deleted` as `primary_deleted`,
        `primary`.`created_by` as `primary_created_by`,
        `primary`.`created_on` as `primary_created_on`,
        `primary`.`updated_on` as `primary_updated_on`,
        `primary`.`updated_by` as `primary_updated_by`,

        `created_by_user`.`id` as `created_by_user_id`,
        `created_by_user`.`zombie` as `created_by_user_zombie`,
        `created_by_user`.`zombie_on` as `created_by_user_zombie_on`,
        `created_by_user`.`first_name` as `created_by_user_first_name`,
        `created_by_user`.`last_name` as `created_by_user_last_name`,
        `created_by_user`.`email` as `created_by_user_email`,
        `created_by_user`.`username` as `created_by_user_username`,
        `created_by_user`.`default_username` as `created_by_user_default_username`,
        `created_by_user`.`api_key` as `created_by_user_api_key`,
        `created_by_user`.`discord_id` as `created_by_user_discord_id`,
        `created_by_user`.`password` as `created_by_user_password`,
        `created_by_user`.`last_seen_on` as `created_by_user_last_seen_on`,
        `created_by_user`.`last_seen_ip` as `created_by_user_last_seen_ip`,
        `created_by_user`.`last_seen_browser` as `created_by_user_last_seen_browser`,
        `created_by_user`.`version_of` as `created_by_user_version_of`,
        `created_by_user`.`banned` as `created_by_user_banned`,
        `created_by_user`.`banned_by` as `created_by_user_banned_by`,
        `created_by_user`.`banned_on` as `created_by_user_banned_on`,
        `created_by_user`.`banned_reason` as `created_by_user_banned_reason`,
        `created_by_user`.`deleted` as `created_by_user_deleted`,
        `created_by_user`.`deleted_by` as `created_by_user_deleted_by`,
        `created_by_user`.`deleted_on` as `created_by_user_deleted_on`,
        `created_by_user`.`created_by` as `created_by_user_created_by`,
        `created_by_user`.`created_on` as `created_by_user_created_on`,
        `created_by_user`.`updated_by` as `created_by_user_updated_by`,
        `created_by_user`.`updated_on` as `created_by_user_updated_on`,
        `created_by_user`.`profile_image` as `created_by_user_profile_image`,
        `created_by_user`.`is_admin` as `created_by_user_is_admin`,
        `created_by_user`.`is_premium` as `created_by_user_is_premium`,
        `created_by_user`.`number_years` as `created_by_user_number_years`,
        `created_by_user`.`is_developer` as `created_by_user_is_developer`,
        `created_by_user`.`is_ace` as `created_by_user_is_ace`,
        `created_by_user`.`theme_css` as `created_by_user_theme_css`,
        `created_by_user`.`reset_password_link` as `created_by_user_reset_password_link`,
        `created_by_user`.`reset_password_expire` as `created_by_user_reset_password_expire`,
        `created_by_user`.`activated` as `created_by_user_activated`,
        `created_by_user`.`share_show_profile_image` as `created_by_user_share_show_profile_image`,
        `created_by_user`.`twitter` as `created_by_user_twitter`,
        `created_by_user`.`share_display_name` as `created_by_user_share_display_name`,
        `created_by_user`.`premium_expires` as `created_by_user_premium_expires`,
        `created_by_user`.`lc_wildcard_reason` as `created_by_user_lc_wildcard_reason`,
        `created_by_user`.`paypal_payment_id` as `created_by_user_paypal_payment_id`,
        `created_by_user`.`notify_email` as `created_by_user_notify_email`,
        `created_by_user`.`timezone` as `created_by_user_timezone`,
        `created_by_user`.`partner_id` as `created_by_user_partner_id`,
        `created_by_user`.`group_ids` as `created_by_user_group_ids`,
        `created_by_user`.`show_user_page` as `created_by_user_show_user_page`,
        `created_by_user`.`share_bio` as `created_by_user_share_bio`,
        `created_by_user`.`hidden_banners` as `created_by_user_hidden_banners`,
        `created_by_user`.`turn_off_advance_limits` as `created_by_user_turn_off_advance_limits`,
        `created_by_user`.`notes` as `created_by_user_notes`,
        `created_by_user`.`login_tokens` as `created_by_user_login_tokens`,

        `deleted_by_user`.`id` as `deleted_by_user_id`,
        `deleted_by_user`.`zombie` as `deleted_by_user_zombie`,
        `deleted_by_user`.`zombie_on` as `deleted_by_user_zombie_on`,
        `deleted_by_user`.`first_name` as `deleted_by_user_first_name`,
        `deleted_by_user`.`last_name` as `deleted_by_user_last_name`,
        `deleted_by_user`.`email` as `deleted_by_user_email`,
        `deleted_by_user`.`username` as `deleted_by_user_username`,
        `deleted_by_user`.`default_username` as `deleted_by_user_default_username`,
        `deleted_by_user`.`api_key` as `deleted_by_user_api_key`,
        `deleted_by_user`.`discord_id` as `deleted_by_user_discord_id`,
        `deleted_by_user`.`password` as `deleted_by_user_password`,
        `deleted_by_user`.`last_seen_on` as `deleted_by_user_last_seen_on`,
        `deleted_by_user`.`last_seen_ip` as `deleted_by_user_last_seen_ip`,
        `deleted_by_user`.`last_seen_browser` as `deleted_by_user_last_seen_browser`,
        `deleted_by_user`.`version_of` as `deleted_by_user_version_of`,
        `deleted_by_user`.`banned` as `deleted_by_user_banned`,
        `deleted_by_user`.`banned_by` as `deleted_by_user_banned_by`,
        `deleted_by_user`.`banned_on` as `deleted_by_user_banned_on`,
        `deleted_by_user`.`banned_reason` as `deleted_by_user_banned_reason`,
        `deleted_by_user`.`deleted` as `deleted_by_user_deleted`,
        `deleted_by_user`.`deleted_by` as `deleted_by_user_deleted_by`,
        `deleted_by_user`.`deleted_on` as `deleted_by_user_deleted_on`,
        `deleted_by_user`.`created_by` as `deleted_by_user_created_by`,
        `deleted_by_user`.`created_on` as `deleted_by_user_created_on`,
        `deleted_by_user`.`updated_by` as `deleted_by_user_updated_by`,
        `deleted_by_user`.`updated_on` as `deleted_by_user_updated_on`,
        `deleted_by_user`.`profile_image` as `deleted_by_user_profile_image`,
        `deleted_by_user`.`is_admin` as `deleted_by_user_is_admin`,
        `deleted_by_user`.`is_premium` as `deleted_by_user_is_premium`,
        `deleted_by_user`.`number_years` as `deleted_by_user_number_years`,
        `deleted_by_user`.`is_developer` as `deleted_by_user_is_developer`,
        `deleted_by_user`.`is_ace` as `deleted_by_user_is_ace`,
        `deleted_by_user`.`theme_css` as `deleted_by_user_theme_css`,
        `deleted_by_user`.`reset_password_link` as `deleted_by_user_reset_password_link`,
        `deleted_by_user`.`reset_password_expire` as `deleted_by_user_reset_password_expire`,
        `deleted_by_user`.`activated` as `deleted_by_user_activated`,
        `deleted_by_user`.`share_show_profile_image` as `deleted_by_user_share_show_profile_image`,
        `deleted_by_user`.`twitter` as `deleted_by_user_twitter`,
        `deleted_by_user`.`share_display_name` as `deleted_by_user_share_display_name`,
        `deleted_by_user`.`premium_expires` as `deleted_by_user_premium_expires`,
        `deleted_by_user`.`lc_wildcard_reason` as `deleted_by_user_lc_wildcard_reason`,
        `deleted_by_user`.`paypal_payment_id` as `deleted_by_user_paypal_payment_id`,
        `deleted_by_user`.`notify_email` as `deleted_by_user_notify_email`,
        `deleted_by_user`.`timezone` as `deleted_by_user_timezone`,
        `deleted_by_user`.`partner_id` as `deleted_by_user_partner_id`,
        `deleted_by_user`.`group_ids` as `deleted_by_user_group_ids`,
        `deleted_by_user`.`show_user_page` as `deleted_by_user_show_user_page`,
        `deleted_by_user`.`share_bio` as `deleted_by_user_share_bio`,
        `deleted_by_user`.`hidden_banners` as `deleted_by_user_hidden_banners`,
        `deleted_by_user`.`turn_off_advance_limits` as `deleted_by_user_turn_off_advance_limits`,
        `deleted_by_user`.`notes` as `deleted_by_user_notes`,
        `deleted_by_user`.`login_tokens` as `deleted_by_user_login_tokens`,

        `updated_by_user`.`id` as `updated_by_user_id`,
        `updated_by_user`.`zombie` as `updated_by_user_zombie`,
        `updated_by_user`.`zombie_on` as `updated_by_user_zombie_on`,
        `updated_by_user`.`first_name` as `updated_by_user_first_name`,
        `updated_by_user`.`last_name` as `updated_by_user_last_name`,
        `updated_by_user`.`email` as `updated_by_user_email`,
        `updated_by_user`.`username` as `updated_by_user_username`,
        `updated_by_user`.`default_username` as `updated_by_user_default_username`,
        `updated_by_user`.`api_key` as `updated_by_user_api_key`,
        `updated_by_user`.`discord_id` as `updated_by_user_discord_id`,
        `updated_by_user`.`password` as `updated_by_user_password`,
        `updated_by_user`.`last_seen_on` as `updated_by_user_last_seen_on`,
        `updated_by_user`.`last_seen_ip` as `updated_by_user_last_seen_ip`,
        `updated_by_user`.`last_seen_browser` as `updated_by_user_last_seen_browser`,
        `updated_by_user`.`version_of` as `updated_by_user_version_of`,
        `updated_by_user`.`banned` as `updated_by_user_banned`,
        `updated_by_user`.`banned_by` as `updated_by_user_banned_by`,
        `updated_by_user`.`banned_on` as `updated_by_user_banned_on`,
        `updated_by_user`.`banned_reason` as `updated_by_user_banned_reason`,
        `updated_by_user`.`deleted` as `updated_by_user_deleted`,
        `updated_by_user`.`deleted_by` as `updated_by_user_deleted_by`,
        `updated_by_user`.`deleted_on` as `updated_by_user_deleted_on`,
        `updated_by_user`.`created_by` as `updated_by_user_created_by`,
        `updated_by_user`.`created_on` as `updated_by_user_created_on`,
        `updated_by_user`.`updated_by` as `updated_by_user_updated_by`,
        `updated_by_user`.`updated_on` as `updated_by_user_updated_on`,
        `updated_by_user`.`profile_image` as `updated_by_user_profile_image`,
        `updated_by_user`.`is_admin` as `updated_by_user_is_admin`,
        `updated_by_user`.`is_premium` as `updated_by_user_is_premium`,
        `updated_by_user`.`number_years` as `updated_by_user_number_years`,
        `updated_by_user`.`is_developer` as `updated_by_user_is_developer`,
        `updated_by_user`.`is_ace` as `updated_by_user_is_ace`,
        `updated_by_user`.`theme_css` as `updated_by_user_theme_css`,
        `updated_by_user`.`reset_password_link` as `updated_by_user_reset_password_link`,
        `updated_by_user`.`reset_password_expire` as `updated_by_user_reset_password_expire`,
        `updated_by_user`.`activated` as `updated_by_user_activated`,
        `updated_by_user`.`share_show_profile_image` as `updated_by_user_share_show_profile_image`,
        `updated_by_user`.`twitter` as `updated_by_user_twitter`,
        `updated_by_user`.`share_display_name` as `updated_by_user_share_display_name`,
        `updated_by_user`.`premium_expires` as `updated_by_user_premium_expires`,
        `updated_by_user`.`lc_wildcard_reason` as `updated_by_user_lc_wildcard_reason`,
        `updated_by_user`.`paypal_payment_id` as `updated_by_user_paypal_payment_id`,
        `updated_by_user`.`notify_email` as `updated_by_user_notify_email`,
        `updated_by_user`.`timezone` as `updated_by_user_timezone`,
        `updated_by_user`.`partner_id` as `updated_by_user_partner_id`,
        `updated_by_user`.`group_ids` as `updated_by_user_group_ids`,
        `updated_by_user`.`show_user_page` as `updated_by_user_show_user_page`,
        `updated_by_user`.`share_bio` as `updated_by_user_share_bio`,
        `updated_by_user`.`hidden_banners` as `updated_by_user_hidden_banners`,
        `updated_by_user`.`turn_off_advance_limits` as `updated_by_user_turn_off_advance_limits`,
        `updated_by_user`.`notes` as `updated_by_user_notes`,
        `updated_by_user`.`login_tokens` as `updated_by_user_login_tokens`,

        `book`.`name` as `book_name`,
        `book`.`short_name` as `book_short_name`
        from `chargen_{}` as `primary`
        left join `users` `created_by_user` on `primary`.created_by = `created_by_user`.id
        left join `users` `deleted_by_user` on `primary`.deleted_by = `deleted_by_user`.id
        left join `users` `updated_by_user` on `primary`.updated_by = `updated_by_user`.id
        left join `books` `book` on `primary`.book_id = `book`.id
        WHERE `primary`.`deleted` < 1 and `primary`.`version_of` = 0

    ", &table);

    let paging = admin_current_limit_paging_sql( &paging_params );
    // let data_params = params!{
    //      "1" => 1
    // };

    data_query = data_query + admin_filter_where_clause(
        DATA_SEARCH_FIELDS,
        &paging_params,
        false,
        true,
    ).as_str();

    data_query = data_query + &"\norder by `book`.`name` ASC, `primary`.`name` ASC\n" + &paging;

    // println!("admin_get_game_data data_query:\n{}", data_query);

    match pool.get_conn() {
        Ok( mut conn) => {

            let saves_result: Result<Vec<Row>> = conn.exec(
                data_query,
                // data_params,
                (),
            );

            match saves_result {
                Ok(  rows ) => {

                    let mut rv: Vec<GameDataRow> = Vec::new();
                    for row in rows {
                        // let row_data = _make_save_from_row(row, with_cached_data);

                        // if (&row_data.name).to_owned() == "Chi Master".to_owned() {
                        //     println!("row_data {:?}", row_data );
                        // }
                        // println!("cols {:?}", row.columns(). );
                        // for col in row.columns().into_iter() {
                        //     println!("cols {:?}", col.name_str() );
                        // }
                        // let mut item = _make_game_data_struct( row );
                        rv.push( _make_game_data_struct( row ) );
                    }

                    // println!("rv.len {}", rv.len() );
                    return rv;
                }
                Err( err ) => {
                    println!("get_item_saves Error 4 {}", err );
                    return Vec::new();
                }

            }

        }
        Err( err ) => {
            println!("get_item_saves Error 3 {}", err );
        }
    }
    return Vec::new();
}

pub fn db_admin_admin_get_item(
    pool: Data<Pool>,
    table: String,
    id: u32,
) -> Option<GameDataRow> {

    let data_query = format!("
        SELECT
        `primary`.`name` as `primary_name`,
        `primary`.`book_id` as `primary_book_id`,
        `primary`.`active` as `primary_active`,
        `primary`.`id` as `primary_id`,
        `primary`.`data` as `primary_data`,
        `primary`.`version_of` as `primary_version_of`,
        `primary`.`deleted_by` as `primary_deleted_by`,
        `primary`.`deleted_on` as `primary_deleted_on`,
        `primary`.`deleted` as `primary_deleted`,
        `primary`.`created_by` as `primary_created_by`,
        `primary`.`created_on` as `primary_created_on`,
        `primary`.`updated_on` as `primary_updated_on`,
        `primary`.`updated_by` as `primary_updated_by`,

        `created_by_user`.`id` as `created_by_user_id`,
        `created_by_user`.`zombie` as `created_by_user_zombie`,
        `created_by_user`.`zombie_on` as `created_by_user_zombie_on`,
        `created_by_user`.`first_name` as `created_by_user_first_name`,
        `created_by_user`.`last_name` as `created_by_user_last_name`,
        `created_by_user`.`email` as `created_by_user_email`,
        `created_by_user`.`username` as `created_by_user_username`,
        `created_by_user`.`default_username` as `created_by_user_default_username`,
        `created_by_user`.`api_key` as `created_by_user_api_key`,
        `created_by_user`.`discord_id` as `created_by_user_discord_id`,
        `created_by_user`.`password` as `created_by_user_password`,
        `created_by_user`.`last_seen_on` as `created_by_user_last_seen_on`,
        `created_by_user`.`last_seen_ip` as `created_by_user_last_seen_ip`,
        `created_by_user`.`last_seen_browser` as `created_by_user_last_seen_browser`,
        `created_by_user`.`version_of` as `created_by_user_version_of`,
        `created_by_user`.`banned` as `created_by_user_banned`,
        `created_by_user`.`banned_by` as `created_by_user_banned_by`,
        `created_by_user`.`banned_on` as `created_by_user_banned_on`,
        `created_by_user`.`banned_reason` as `created_by_user_banned_reason`,
        `created_by_user`.`deleted` as `created_by_user_deleted`,
        `created_by_user`.`deleted_by` as `created_by_user_deleted_by`,
        `created_by_user`.`deleted_on` as `created_by_user_deleted_on`,
        `created_by_user`.`created_by` as `created_by_user_created_by`,
        `created_by_user`.`created_on` as `created_by_user_created_on`,
        `created_by_user`.`updated_by` as `created_by_user_updated_by`,
        `created_by_user`.`updated_on` as `created_by_user_updated_on`,
        `created_by_user`.`profile_image` as `created_by_user_profile_image`,
        `created_by_user`.`is_admin` as `created_by_user_is_admin`,
        `created_by_user`.`is_premium` as `created_by_user_is_premium`,
        `created_by_user`.`number_years` as `created_by_user_number_years`,
        `created_by_user`.`is_developer` as `created_by_user_is_developer`,
        `created_by_user`.`is_ace` as `created_by_user_is_ace`,
        `created_by_user`.`theme_css` as `created_by_user_theme_css`,
        `created_by_user`.`reset_password_link` as `created_by_user_reset_password_link`,
        `created_by_user`.`reset_password_expire` as `created_by_user_reset_password_expire`,
        `created_by_user`.`activated` as `created_by_user_activated`,
        `created_by_user`.`share_show_profile_image` as `created_by_user_share_show_profile_image`,
        `created_by_user`.`twitter` as `created_by_user_twitter`,
        `created_by_user`.`share_display_name` as `created_by_user_share_display_name`,
        `created_by_user`.`premium_expires` as `created_by_user_premium_expires`,
        `created_by_user`.`lc_wildcard_reason` as `created_by_user_lc_wildcard_reason`,
        `created_by_user`.`paypal_payment_id` as `created_by_user_paypal_payment_id`,
        `created_by_user`.`notify_email` as `created_by_user_notify_email`,
        `created_by_user`.`timezone` as `created_by_user_timezone`,
        `created_by_user`.`partner_id` as `created_by_user_partner_id`,
        `created_by_user`.`group_ids` as `created_by_user_group_ids`,
        `created_by_user`.`show_user_page` as `created_by_user_show_user_page`,
        `created_by_user`.`share_bio` as `created_by_user_share_bio`,
        `created_by_user`.`hidden_banners` as `created_by_user_hidden_banners`,
        `created_by_user`.`turn_off_advance_limits` as `created_by_user_turn_off_advance_limits`,
        `created_by_user`.`notes` as `created_by_user_notes`,
        `created_by_user`.`login_tokens` as `created_by_user_login_tokens`,

        `deleted_by_user`.`id` as `deleted_by_user_id`,
        `deleted_by_user`.`zombie` as `deleted_by_user_zombie`,
        `deleted_by_user`.`zombie_on` as `deleted_by_user_zombie_on`,
        `deleted_by_user`.`first_name` as `deleted_by_user_first_name`,
        `deleted_by_user`.`last_name` as `deleted_by_user_last_name`,
        `deleted_by_user`.`email` as `deleted_by_user_email`,
        `deleted_by_user`.`username` as `deleted_by_user_username`,
        `deleted_by_user`.`default_username` as `deleted_by_user_default_username`,
        `deleted_by_user`.`api_key` as `deleted_by_user_api_key`,
        `deleted_by_user`.`discord_id` as `deleted_by_user_discord_id`,
        `deleted_by_user`.`password` as `deleted_by_user_password`,
        `deleted_by_user`.`last_seen_on` as `deleted_by_user_last_seen_on`,
        `deleted_by_user`.`last_seen_ip` as `deleted_by_user_last_seen_ip`,
        `deleted_by_user`.`last_seen_browser` as `deleted_by_user_last_seen_browser`,
        `deleted_by_user`.`version_of` as `deleted_by_user_version_of`,
        `deleted_by_user`.`banned` as `deleted_by_user_banned`,
        `deleted_by_user`.`banned_by` as `deleted_by_user_banned_by`,
        `deleted_by_user`.`banned_on` as `deleted_by_user_banned_on`,
        `deleted_by_user`.`banned_reason` as `deleted_by_user_banned_reason`,
        `deleted_by_user`.`deleted` as `deleted_by_user_deleted`,
        `deleted_by_user`.`deleted_by` as `deleted_by_user_deleted_by`,
        `deleted_by_user`.`deleted_on` as `deleted_by_user_deleted_on`,
        `deleted_by_user`.`created_by` as `deleted_by_user_created_by`,
        `deleted_by_user`.`created_on` as `deleted_by_user_created_on`,
        `deleted_by_user`.`updated_by` as `deleted_by_user_updated_by`,
        `deleted_by_user`.`updated_on` as `deleted_by_user_updated_on`,
        `deleted_by_user`.`profile_image` as `deleted_by_user_profile_image`,
        `deleted_by_user`.`is_admin` as `deleted_by_user_is_admin`,
        `deleted_by_user`.`is_premium` as `deleted_by_user_is_premium`,
        `deleted_by_user`.`number_years` as `deleted_by_user_number_years`,
        `deleted_by_user`.`is_developer` as `deleted_by_user_is_developer`,
        `deleted_by_user`.`is_ace` as `deleted_by_user_is_ace`,
        `deleted_by_user`.`theme_css` as `deleted_by_user_theme_css`,
        `deleted_by_user`.`reset_password_link` as `deleted_by_user_reset_password_link`,
        `deleted_by_user`.`reset_password_expire` as `deleted_by_user_reset_password_expire`,
        `deleted_by_user`.`activated` as `deleted_by_user_activated`,
        `deleted_by_user`.`share_show_profile_image` as `deleted_by_user_share_show_profile_image`,
        `deleted_by_user`.`twitter` as `deleted_by_user_twitter`,
        `deleted_by_user`.`share_display_name` as `deleted_by_user_share_display_name`,
        `deleted_by_user`.`premium_expires` as `deleted_by_user_premium_expires`,
        `deleted_by_user`.`lc_wildcard_reason` as `deleted_by_user_lc_wildcard_reason`,
        `deleted_by_user`.`paypal_payment_id` as `deleted_by_user_paypal_payment_id`,
        `deleted_by_user`.`notify_email` as `deleted_by_user_notify_email`,
        `deleted_by_user`.`timezone` as `deleted_by_user_timezone`,
        `deleted_by_user`.`partner_id` as `deleted_by_user_partner_id`,
        `deleted_by_user`.`group_ids` as `deleted_by_user_group_ids`,
        `deleted_by_user`.`show_user_page` as `deleted_by_user_show_user_page`,
        `deleted_by_user`.`share_bio` as `deleted_by_user_share_bio`,
        `deleted_by_user`.`hidden_banners` as `deleted_by_user_hidden_banners`,
        `deleted_by_user`.`turn_off_advance_limits` as `deleted_by_user_turn_off_advance_limits`,
        `deleted_by_user`.`notes` as `deleted_by_user_notes`,
        `deleted_by_user`.`login_tokens` as `deleted_by_user_login_tokens`,

        `updated_by_user`.`id` as `updated_by_user_id`,
        `updated_by_user`.`zombie` as `updated_by_user_zombie`,
        `updated_by_user`.`zombie_on` as `updated_by_user_zombie_on`,
        `updated_by_user`.`first_name` as `updated_by_user_first_name`,
        `updated_by_user`.`last_name` as `updated_by_user_last_name`,
        `updated_by_user`.`email` as `updated_by_user_email`,
        `updated_by_user`.`username` as `updated_by_user_username`,
        `updated_by_user`.`default_username` as `updated_by_user_default_username`,
        `updated_by_user`.`api_key` as `updated_by_user_api_key`,
        `updated_by_user`.`discord_id` as `updated_by_user_discord_id`,
        `updated_by_user`.`password` as `updated_by_user_password`,
        `updated_by_user`.`last_seen_on` as `updated_by_user_last_seen_on`,
        `updated_by_user`.`last_seen_ip` as `updated_by_user_last_seen_ip`,
        `updated_by_user`.`last_seen_browser` as `updated_by_user_last_seen_browser`,
        `updated_by_user`.`version_of` as `updated_by_user_version_of`,
        `updated_by_user`.`banned` as `updated_by_user_banned`,
        `updated_by_user`.`banned_by` as `updated_by_user_banned_by`,
        `updated_by_user`.`banned_on` as `updated_by_user_banned_on`,
        `updated_by_user`.`banned_reason` as `updated_by_user_banned_reason`,
        `updated_by_user`.`deleted` as `updated_by_user_deleted`,
        `updated_by_user`.`deleted_by` as `updated_by_user_deleted_by`,
        `updated_by_user`.`deleted_on` as `updated_by_user_deleted_on`,
        `updated_by_user`.`created_by` as `updated_by_user_created_by`,
        `updated_by_user`.`created_on` as `updated_by_user_created_on`,
        `updated_by_user`.`updated_by` as `updated_by_user_updated_by`,
        `updated_by_user`.`updated_on` as `updated_by_user_updated_on`,
        `updated_by_user`.`profile_image` as `updated_by_user_profile_image`,
        `updated_by_user`.`is_admin` as `updated_by_user_is_admin`,
        `updated_by_user`.`is_premium` as `updated_by_user_is_premium`,
        `updated_by_user`.`number_years` as `updated_by_user_number_years`,
        `updated_by_user`.`is_developer` as `updated_by_user_is_developer`,
        `updated_by_user`.`is_ace` as `updated_by_user_is_ace`,
        `updated_by_user`.`theme_css` as `updated_by_user_theme_css`,
        `updated_by_user`.`reset_password_link` as `updated_by_user_reset_password_link`,
        `updated_by_user`.`reset_password_expire` as `updated_by_user_reset_password_expire`,
        `updated_by_user`.`activated` as `updated_by_user_activated`,
        `updated_by_user`.`share_show_profile_image` as `updated_by_user_share_show_profile_image`,
        `updated_by_user`.`twitter` as `updated_by_user_twitter`,
        `updated_by_user`.`share_display_name` as `updated_by_user_share_display_name`,
        `updated_by_user`.`premium_expires` as `updated_by_user_premium_expires`,
        `updated_by_user`.`lc_wildcard_reason` as `updated_by_user_lc_wildcard_reason`,
        `updated_by_user`.`paypal_payment_id` as `updated_by_user_paypal_payment_id`,
        `updated_by_user`.`notify_email` as `updated_by_user_notify_email`,
        `updated_by_user`.`timezone` as `updated_by_user_timezone`,
        `updated_by_user`.`partner_id` as `updated_by_user_partner_id`,
        `updated_by_user`.`group_ids` as `updated_by_user_group_ids`,
        `updated_by_user`.`show_user_page` as `updated_by_user_show_user_page`,
        `updated_by_user`.`share_bio` as `updated_by_user_share_bio`,
        `updated_by_user`.`hidden_banners` as `updated_by_user_hidden_banners`,
        `updated_by_user`.`turn_off_advance_limits` as `updated_by_user_turn_off_advance_limits`,
        `updated_by_user`.`notes` as `updated_by_user_notes`,
        `updated_by_user`.`login_tokens` as `updated_by_user_login_tokens`,

        `book`.`name` as `book_name`,
        `book`.`short_name` as `book_short_name`
        from `chargen_{}` as `primary`
        left join `users` `created_by_user` on `primary`.created_by = `created_by_user`.id
        left join `users` `deleted_by_user` on `primary`.deleted_by = `deleted_by_user`.id
        left join `users` `updated_by_user` on `primary`.updated_by = `updated_by_user`.id
        left join `books` `book` on `primary`.book_id = `book`.id
        WHERE `primary`.`deleted` < 1 and `primary`.`version_of` = 0 and `primary`.`id` = :id
        limit 1

    ", &table);

    // let paging = admin_current_limit_paging_sql( &paging_params );
    let data_params = params!{
         "id" => id
    };

    // data_query = data_query + admin_filter_where_clause(
    //     DATA_SEARCH_FIELDS,
    //     &paging_params,
    //     false,
    //     true,
    // ).as_str();

    // data_query = data_query + &"\norder by `book`.`name` ASC, `primary`.`name` ASC\n" + &paging;

    // println!("admin_get_game_data data_query:\n{}", data_query);

    match pool.get_conn() {
        Ok( mut conn) => {

            let saves_result: Result<Vec<Row>> = conn.exec(
                data_query,
                data_params,

            );

            match saves_result {
                Ok(  rows ) => {

                    for row in rows {
                        // let row_data = _make_save_from_row(row, with_cached_data);

                        // if (&row_data.name).to_owned() == "Chi Master".to_owned() {
                        //     println!("row_data {:?}", row_data );
                        // }
                        // println!("cols {:?}", row.columns(). );
                        // for col in row.columns().into_iter() {
                        //     println!("cols {:?}", col.name_str() );
                        // }
                        // let mut item = _make_game_data_struct( row );
                        return Some( _make_game_data_struct( row ) );
                    }

                    // println!("rv.len {}", rv.len() );
                    return None;
                }
                Err( err ) => {
                    println!("get_item_saves Error 4 {}", err );
                    return None;
                }

            }

        }
        Err( err ) => {
            println!("get_item_saves Error 3 {}", err );
        }
    }
    return None;
}

fn _make_game_data_struct( mut row: Row ) -> GameDataRow {

    let mut created_by = 0;
    let mut created_by_user: Option<PublicUserInfo> = None;
    let created_opt= row.take_opt("primary_created_by").unwrap();
    match created_opt {

        Ok( val ) => {
            // println!("created_by val {:?}", val );
            created_by = val;
            if val > 0 {
                let user = make_user_from_row(row.clone(), "created_by_user_".to_owned());
                created_by_user = Some(user.get_public_info(true));
            }
        }
        Err( _ ) => {}

    }
    let mut updated_by = 0;
    let mut updated_by_user: Option<PublicUserInfo> = None;
    let updated_opt = row.take_opt("primary_updated_by").unwrap();
    match updated_opt {

        Ok( val ) => {
            // println!("updated_by val {:?}", val );
            updated_by = val;
            if val > 0 {
                let user = make_user_from_row(row.clone(), "updated_by_user_".to_owned());
                updated_by_user = Some(user.get_public_info(true));
            }
        }
        Err( err ) => {
            println!("updated_by error {:?}", err );
        }

    }
    let mut deleted_by = 0;
    let mut deleted_by_user: Option<PublicUserInfo> = None;
    let deleted_opt = row.take_opt("primary_deleted_by").unwrap();
    match deleted_opt {

        Ok( val ) => {
            deleted_by = val;
            if val > 0 {
                let user = make_user_from_row(row.clone(), "deleted_by_user_".to_owned());
                deleted_by_user = Some(user.get_public_info(true));
            }
        }
        Err( _ ) => {}

    }

    let mut data= "".to_string();
    let data_opt = row.take_opt("primary_data").unwrap();
    match data_opt {

        Ok( val ) => {data = val; }
        Err( _ ) => {}

    }

    let mut book_name:Option<String> = None;
    let book_name_opt = row.take_opt("book_name").unwrap();
    match book_name_opt {

        Ok( val ) => {book_name = val;}
        Err( _err ) => {
            println!("game data book_name err {:?}", _err);
        }

    }

    let mut book_short_name:Option<String> = None;
    let book_short_name_opt = row.take_opt("book_short_name").unwrap();
    match book_short_name_opt {

        Ok( val ) => {book_short_name = val;}
        Err( _err ) => {
            println!("game data book_name err {:?}", _err);
        }

    }

    return GameDataRow {
        active: row.take("primary_active").unwrap(),
        name: row.take("primary_name").unwrap(),

        data: data,

        created_by: created_by,
        created_on: mysql_row_to_chrono_utc(&mut row, "primary_created_on"), // created_on_dtfo.with_timezone( &Utc),

        deleted: row.take("primary_deleted").unwrap(),
        deleted_by: deleted_by,
        deleted_on: mysql_row_to_chrono_utc( &mut row, "primary_deleted_on"), // row.take("deleted_on").unwrap(),

        updated_by: updated_by,
        updated_on: mysql_row_to_chrono_utc( &mut row, "primary_updated_on"), // updated_on_dtfo.with_timezone( &Utc),
        version_of: row.take("primary_version_of").unwrap(),

        book_id: row.take("primary_book_id").unwrap(),
        id: row.take("primary_id").unwrap(),

        book_name: book_name,
        book_short_name: book_short_name,

        created_by_user: created_by_user,
        deleted_by_user: deleted_by_user,
        updated_by_user: updated_by_user,

    };
}