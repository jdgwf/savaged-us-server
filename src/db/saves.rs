use mysql::*;
use mysql::prelude::*;
use chrono::prelude::*;
use actix_web:: {
    // web::Json,
    web::Data,
};
use std::path::Path;
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

        name,
        sort_order,
        type,

        export_generic_json,
        share_html,

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
        co_owner_folder,

        uuid_virtual



        from saves

        WHERE created_by = :user_id
        or
        co_owner = :user_id

        "

    );

    let data_params = params!{ "user_id" => user_id};
    // let data_params = params!{ "1" => "1"};

    // println!("data_query {}", &data_query);
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
                        let row_data = _make_save_from_row(row, with_cached_data);

                        // if (&row_data.name).to_owned() == "Chi Master".to_owned() {
                        //     println!("row_data {:?}", row_data );
                        // }

                        rv.push( row_data );
                    }
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
        let export_generic_json = row.take_opt("export_generic_json");
        match export_generic_json {
            Some( val_option ) => {
                match val_option {
                    Ok( val ) => {
                        export_generic_json_send = val;
                    }
                    Err ( _) => {}
                }
            }
            None => {}
        }

        let share_html_opt = row.take_opt("share_html");
        match share_html_opt {
            Some( val_option ) => {
                match val_option {
                    Ok( val ) => {
                        export_share_html = val;
                    }
                    Err ( _) => {}
                }
            }
            None => {}
        }
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

    let mut uuid_string = "".to_owned();
    match row.take_opt("uuid_virtual") {
        Some( val ) => {
            match val {
                Ok( val_val ) => {
                    uuid_string = val_val;
                }
                Err( _ ) => {

                }
            }

        }
        None => {

        }
    }
    let save_type: String = row.take("type").unwrap();
    let id: u32 = row.take("id").unwrap();
    let updated_on = mysql_datetime_to_chrono_utc(updated_on_string);

    let mut imageurl = "".to_owned();

    let data_dir_path = "./data/uploads/";
    let base_file_name = save_type.to_string() + &"s/".to_owned() + &created_by.to_string() + &"-".to_owned() + &id.to_string();
    let png_filename = data_dir_path.to_owned() + &base_file_name + &".png".to_owned();
    let jpg_filename = data_dir_path.to_owned() + &base_file_name + &".jpg".to_owned();
    let webp_filename = data_dir_path.to_owned() + &base_file_name + &".webp".to_owned();

    if Path::new(&webp_filename).exists() {
        match updated_on {
            Some(updated) => {
                imageurl = "/data-images/".to_owned() + &base_file_name + &".webp?v=" + &updated.timestamp().to_string();
            }
            None => {
                imageurl = "/data-images/".to_owned() + &base_file_name + &".webp";
            }
        }
    } else {

        if Path::new(&jpg_filename).exists() {
            match updated_on {
                Some(updated) => {
                    imageurl = "/data-images/".to_owned() + &base_file_name + &".jpg?v=" + &updated.timestamp().to_string();
                }
                None => {
                    imageurl = "/data-images/".to_owned() + &base_file_name + &".jpg";
                }
            }
        } else {

            if Path::new(&png_filename).exists() {
                match updated_on {
                    Some(updated) => {
                        imageurl = "/data-images/".to_owned() + &base_file_name + &".png?v=" + &updated.timestamp().to_string();
                    }
                    None => {
                        imageurl = "/data-images/".to_owned() + &base_file_name + &".png";
                    }
                }
            }
        }
    }

    let deleted: i32 = row.take("deleted").unwrap();
    let name: String = row.take("name").unwrap();

    let mut deleted_bool = false;
    if deleted > 0 {
        deleted_bool = true;
    }

    // println!("save {} {} {} {}", id, deleted, deleted_bool, name);
    return SaveDBRow{
        id: id,
        data: row.take("data").unwrap(),

        created_on: mysql_datetime_to_chrono_utc(created_on_string),
        created_by: created_by,

        updated_on: updated_on,
        updated_by: updated_by,

        deleted: deleted_bool,
        deleted_on: mysql_datetime_to_chrono_utc(deleted_on_string),
        deleted_by: deleted_by,


        // session_id: row.take("session_id").unwrap(),
        name: name,
        sort_order: row.take("sort_order").unwrap(),
        save_type: save_type,

        export_generic_json: export_generic_json_send,
        share_html: export_share_html,

        // setting_name: row.take("setting_name").unwrap(),
        shareurl: share_url,
        short_desc: row.take("short_desc").unwrap(),

        share_public: row.take("share_public").unwrap(),
        share_copy: row.take("share_copy").unwrap(),
        imageurl: imageurl,
        folder: row.take("folder").unwrap(),

        rifts_living_campaign: row.take("rifts_living_campaign").unwrap(),
        hits: row.take("hits").unwrap(),
        total_hits: row.take("total_hits").unwrap(),

        show_character_sheet: row.take("show_character_sheet").unwrap(),
        allow_download: row.take("allow_download").unwrap(),
        session_id: row.take("session_id").unwrap(),

        co_owner: row.take("co_owner").unwrap(),
        co_owner_folder: row.take("co_owner_folder").unwrap(),

        uuid: uuid_string,
        co_owner_public: None,
        created_by_public: None,
        updated_by_public: None,
        deleted_by_public: None,
    }
}