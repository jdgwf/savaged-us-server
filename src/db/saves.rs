use mysql::*;
use mysql::prelude::*;
use chrono::prelude::*;
use actix_web:: {
    // web::Json,
    web::Data,
};

use savaged_libs::save_db_row::SaveDBRow;
use crate::db::utils::mysql_datetime_to_chrono_utc;

pub fn get_user_saves(
    pool: &Data<Pool>,
    user_id: u32,
    updated_on: Option<DateTime<Utc>>,
    with_cached_data: bool,
) -> Vec<SaveDBRow> {

    let mut data_query = format!(
        "SELECT
        id,
        data,

        created_on,
        created_by,

        updated_on,
        updated_by,

        deleted,
        deleted_on,
        deleted_by,

        -- session_id,
        name,
        sort_order,
        type,

        export_generic_json,
        share_html,

        -- setting_name,
        shareurl,
        short_desc,

        share_public,
        share_copy,
        imageurl,
        folder,

        rifts_living_campaign,
        hits,
        total_hits,

        show_character_sheet,
        allow_download,
        session_id,

        co_owner,
        co_owner_folder

        -- co_owner_public,
        -- created_by_public,
        -- updated_by_public


        from saves

        WHERE created_by = :user_id
        or
        co_owner = :user_id

        "

    );

    let data_params = params!{ "user_id" => user_id};
    // let data_params = params!{ "1" => "1"};

    println!("data_query {}", &data_query);
    match pool.get_conn() {
        Ok( mut conn) => {

            let saves_result: Result<Vec<Row>> = conn.exec(
                data_query,
                data_params,
            );
            match saves_result {
                Ok(  rows ) => {

                    let mut rv: Vec<SaveDBRow> = Vec::new();
                    for row in rows {
                        rv.push( _make_save_from_row(row, with_cached_data) );
                    }
                    // let user = _make_user_from_row( row );
                    return rv;
                }
                Err( err ) => {
                    println!("get_user_saves Error 4 {}", err );
                    return Vec::new();
                }

            }

        }
        Err( err ) => {
            println!("get_user_saves Error 3 {}", err );
        }
    }
    return Vec::new();
}



fn _make_save_from_row(
    mut row: Row,
    with_cached_data: bool,
) -> SaveDBRow {


    let mut export_generic_json_send = "".to_owned();
    let mut export_share_html = "".to_owned();
    if with_cached_data {
        export_generic_json_send = row.take("export_generic_json").unwrap();
        export_share_html = row.take("share_html").unwrap();
    }

    let mut created_by = 0;
    let created_opt= row.take_opt("created_by").unwrap();
    match created_opt {

        Ok( val ) => {created_by = val;}
        Err( _ ) => {}

    }
    let mut updated_by = 0;
    let updated_opt = row.take_opt("updated_by").unwrap();
    match updated_opt {

        Ok( val ) => {updated_by = val;}
        Err( _ ) => {}

    }
    let mut deleted_by = 0;
    let deleted_opt = row.take_opt("deleted_by").unwrap();
    match deleted_opt {

        Ok( val ) => {deleted_by = val;}
        Err( _ ) => {}

    }

    let created_on_string: String = row.take_opt("created_on")
    .unwrap_or(Ok("".to_string()))
    .unwrap_or("".to_string());
let deleted_on_string: String = row.take_opt("deleted_on")
    .unwrap_or(Ok("".to_string()))
    .unwrap_or("".to_string());
let updated_on_string: String = row.take_opt("updated_on")
    .unwrap_or(Ok("".to_string()))
    .unwrap_or("".to_string());


    let mut share_url = "".to_string();

    let share_url_result = row.take_opt("shareurl");

    match share_url_result {
        Some( val ) => {
            match val {
                Ok( val_val ) => {
                    share_url = val_val;
                }
                Err( _ ) => {

                }
            }

        }
        None => {

        }
    }

    return SaveDBRow{
        id: row.take("id").unwrap(),
        data: row.take("data").unwrap(),

        created_on: mysql_datetime_to_chrono_utc(created_on_string),
        created_by: created_by,

        updated_on: mysql_datetime_to_chrono_utc(updated_on_string),
        updated_by: updated_by,

        deleted: row.take("deleted").unwrap(),
        deleted_on: mysql_datetime_to_chrono_utc(deleted_on_string),
        deleted_by: deleted_by,


        // session_id: row.take("session_id").unwrap(),
        name: row.take("name").unwrap(),
        sort_order: row.take("sort_order").unwrap(),
        save_type: row.take("type").unwrap(),

        export_generic_json: export_generic_json_send,
        share_html: export_share_html,

        // setting_name: row.take("setting_name").unwrap(),
        shareurl: share_url,
        short_desc: row.take("short_desc").unwrap(),

        share_public: row.take("share_public").unwrap(),
        share_copy: row.take("share_copy").unwrap(),
        imageurl: row.take("imageurl").unwrap(),
        folder: row.take("folder").unwrap(),

        rifts_living_campaign: row.take("rifts_living_campaign").unwrap(),
        hits: row.take("hits").unwrap(),
        total_hits: row.take("total_hits").unwrap(),

        show_character_sheet: row.take("show_character_sheet").unwrap(),
        allow_download: row.take("allow_download").unwrap(),
        session_id: row.take("session_id").unwrap(),

        co_owner: row.take("co_owner").unwrap(),
        co_owner_folder: row.take("co_owner_folder").unwrap(),

        co_owner_public: None,
        created_by_public: None,
        updated_by_public: None,
        deleted_by_public: None,
    }
}