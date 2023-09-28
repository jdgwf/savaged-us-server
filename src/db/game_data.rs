use crate::db::utils::mysql_datetime_to_chrono_utc;
use actix_web::web::Data;
use chrono::prelude::*;
use mysql_async::prelude::*;
use mysql_async::Pool;
use savaged_libs::player_character::armor::Armor;
use savaged_libs::player_character::edge::Edge;
use savaged_libs::player_character::game_data_package::{GameDataPackage, GameDataPackageLevel};
use savaged_libs::player_character::gear::Gear;
use savaged_libs::player_character::hindrance::Hindrance;
use savaged_libs::player_character::weapon::Weapon;
use savaged_libs::utils::bool_from_int_or_bool;
use serde::{Deserialize, Serialize};

use super::books::get_books;

pub async fn get_game_data_package(
    pool: &Data<Pool>,
    current_user_id: u32,
    updated_on: Option<DateTime<Utc>>,
    access_registered: bool,
    access_wildcard: bool,
    access_developer: bool,
    access_admin: bool,
    all: bool,
) -> GameDataPackage {
    let mut book_ids: Vec<u32> = Vec::new();

    let books = get_books(
        &pool,
        current_user_id,
        updated_on,
        access_registered,
        access_wildcard,
        access_developer,
        access_admin,
        all,
    ).await;

    for book in &books {
        book_ids.push(book.id);
    }

    let hindrances = get_hindrances(&pool, updated_on, &book_ids, all).await;

    let edges = get_edges(&pool, updated_on, &book_ids, all).await;

    let gear = get_gear(&pool, updated_on, &book_ids, all).await;

    let armor = get_armor(&pool, updated_on, &book_ids, all).await;
    let weapons = get_weapons(&pool, updated_on, &book_ids, all).await;
    // let weapons: Vec<Weapon> = Vec::new();
    // let armor: Vec<Armor> = Vec::new();

    let mut data_level: GameDataPackageLevel = GameDataPackageLevel::Anonymous;

    if access_admin {
        data_level = GameDataPackageLevel::Admin;
    } else if access_developer {
        data_level = GameDataPackageLevel::Developer;
    } else if access_wildcard {
        data_level = GameDataPackageLevel::WildCard;
    } else if access_registered {
        data_level = GameDataPackageLevel::Registered;
    } else {
        data_level = GameDataPackageLevel::Anonymous;
    }

    return GameDataPackage {
        data_level: data_level,
        books: books,

        edges: edges,
        hindrances: hindrances,

        gear: gear,
        armor: armor,
        weapons: weapons,
        // settings:  Vec::new(),
    };
}

pub async fn get_game_data_table_data(
    pool: &Data<Pool>,
    table_name: String,
    updated_on: Option<DateTime<Utc>>,
    book_ids: &Vec<u32>,
    all: bool,
) -> Vec<RowData> {
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
        deleted_by

        from {}

        WHERE deleted < 1
        and version_of < 1

        ",
        table_name
    );

    if !all {
        if book_ids.len() > 0 {
            data_query = data_query + &" AND book_id in (";

            for book_id in book_ids {
                data_query = data_query + &book_id.to_string() + &",";
            }

            data_query = data_query + &" 99999999) ";
        }
    }

    match updated_on {
        Some( updated_date_string ) => {
            data_query = format!("{}\n  AND `updated_on` > '{}'", data_query, updated_date_string);
        }
        None => {}
    }

    // println!("get_game_data_table_data data_query{}", data_query);
    // let data_params = params!{ "user_id" => user_id};
    // let data_params = params!{ "1" => "1"};
    match pool.get_conn().await {
        Ok(mut conn) => {
            let get_row_data_result = conn.query_map(
                data_query,
                |(
                    id,
                    data,
                    created_on,
                    created_by,
                    updated_on,
                    updated_by,
                    deleted,
                    deleted_on,
                    deleted_by,
                ): (
                    u32,
                    Option<String>,
                    String,
                    u32,
                    String,
                    u32,
                    u32,
                    String,
                    u32,
                )| {
                    let mut deleted_bool = false;
                    if deleted > 0 {
                        deleted_bool = true;
                    }

                    return RowData {
                        id: id,
                        data: data,

                        created_on: mysql_datetime_to_chrono_utc(created_on),
                        created_by: created_by,

                        updated_on: mysql_datetime_to_chrono_utc(updated_on),
                        updated_by: updated_by,

                        deleted: deleted_bool,
                        deleted_on: mysql_datetime_to_chrono_utc(deleted_on),
                        deleted_by: deleted_by,
                    };
                },
            ).await;
            match get_row_data_result {
                Ok(get_row_data) => {
                    println!("get_game_data_table_data {} get_row_data len {}", &table_name, get_row_data.len());
                    return get_row_data;
                }

                Err(err) => {
                    println!("get_game_data_table_data Error 4 {}", err);
                }
            }
        }
        Err(err) => {
            println!("get_game_data_table_data Error 3 {}", err);
        }
    }
    return Vec::new();
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RowData {
    pub id: u32,
    pub data: Option<String>,

    pub created_by: u32,
    #[serde(default)]
    pub created_on: Option<DateTime<Utc>>,

    #[serde(deserialize_with = "bool_from_int_or_bool")]
    pub deleted: bool,
    pub deleted_by: u32,
    #[serde(default)]
    pub deleted_on: Option<DateTime<Utc>>,

    pub updated_by: u32,
    #[serde(default)]
    pub updated_on: Option<DateTime<Utc>>,
}

pub async fn get_hindrances(
    pool: &Data<Pool>,
    updated_on: Option<DateTime<Utc>>,
    book_ids: &Vec<u32>,
    all: bool,
) -> Vec<Hindrance> {
    let rows = get_game_data_table_data(
        pool,
        "chargen_hindrances".to_owned(),
        updated_on,
        book_ids,
        all,
    ).await;

    let mut parsed_data: Vec<Hindrance> = Vec::new();
    for row in rows {
        match row.data {
            Some(row_data) => {
                let data_result: Result<Hindrance, serde_json::Error> =
                    serde_json::from_str(row_data.as_ref());
                match data_result {
                    Ok(mut data) => {
                        data.id = row.id;
                        data.created_on = row.created_on;
                        data.updated_on = row.updated_on;
                        data.deleted_on = row.deleted_on;

                        data.deleted = row.deleted;

                        data.created_by = row.created_by;
                        data.updated_by = row.updated_by;
                        data.deleted_by = row.deleted_by;

                        parsed_data.push(data);
                    }
                    Err(err) => {
                        println!(
                            "Error with data on get_hindrances {}, {}, {}",
                            row.id,
                            err.to_string(),
                            row_data
                        );
                    }
                }
            }
            None => {}
        }
    }
    return parsed_data;
}

pub async fn get_edges(
    pool: &Data<Pool>,
    updated_on: Option<DateTime<Utc>>,
    book_ids: &Vec<u32>,
    all: bool,
) -> Vec<Edge> {
    let rows =
        get_game_data_table_data(pool, "chargen_edges".to_owned(), updated_on, book_ids, all).await;

    let mut parsed_data: Vec<Edge> = Vec::new();
    for row in rows {
        match row.data {
            Some(row_data) => {
                let data_result: Result<Edge, serde_json::Error> =
                    serde_json::from_str(row_data.as_ref());
                match data_result {
                    Ok(mut data) => {
                        data.id = row.id;
                        data.created_on = row.created_on;
                        data.updated_on = row.updated_on;
                        data.deleted_on = row.deleted_on;

                        data.deleted = row.deleted;

                        data.created_by = row.created_by;
                        data.updated_by = row.updated_by;
                        data.deleted_by = row.deleted_by;

                        parsed_data.push(data);
                    }
                    Err(err) => {
                        println!(
                            "Error with data on get_edges {}, {}, {}",
                            row.id,
                            err.to_string(),
                            row_data
                        );
                    }
                }
            }
            None => {}
        }
    }
    return parsed_data;
}

pub async fn get_weapons(
    pool: &Data<Pool>,
    updated_on: Option<DateTime<Utc>>,
    book_ids: &Vec<u32>,
    all: bool,
) -> Vec<Weapon> {
    let rows = get_game_data_table_data(
        pool,
        "chargen_weapons".to_owned(),
        updated_on,
        book_ids,
        all,
    ).await;

    let mut parsed_data: Vec<Weapon> = Vec::new();
    for row in rows {
        match row.data {
            Some(row_data) => {
                let data_result: Result<Weapon, serde_json::Error> =
                    serde_json::from_str(row_data.as_ref());
                match data_result {
                    Ok(mut data) => {
                        data.id = row.id;
                        data.created_on = row.created_on;
                        data.updated_on = row.updated_on;
                        data.deleted_on = row.deleted_on;

                        data.deleted = row.deleted;

                        data.created_by = row.created_by;
                        data.updated_by = row.updated_by;
                        data.deleted_by = row.deleted_by;

                        parsed_data.push(data);
                    }
                    Err(err) => {
                        println!(
                            "Error with data on get_weapons {}, {}, {}",
                            row.id,
                            err.to_string(),
                            row_data
                        );
                    }
                }
            }
            None => {}
        }
    }
    return parsed_data;
}

pub async fn get_gear(
    pool: &Data<Pool>,
    updated_on: Option<DateTime<Utc>>,
    book_ids: &Vec<u32>,
    all: bool,
) -> Vec<Gear> {
    let rows = get_game_data_table_data(pool, "chargen_gear".to_owned(), updated_on, book_ids, all).await;

    let mut parsed_data: Vec<Gear> = Vec::new();
    for row in rows {
        match row.data {
            Some(row_data) => {
                let data_result: Result<Gear, serde_json::Error> =
                    serde_json::from_str(row_data.as_ref());
                match data_result {
                    Ok(mut data) => {
                        data.id = row.id;
                        data.created_on = row.created_on;
                        data.updated_on = row.updated_on;
                        data.deleted_on = row.deleted_on;

                        data.deleted = row.deleted;

                        data.created_by = row.created_by;
                        data.updated_by = row.updated_by;
                        data.deleted_by = row.deleted_by;

                        parsed_data.push(data);
                    }
                    Err(err) => {
                        println!(
                            "Error with data on get_gear {}, {}, {}",
                            row.id,
                            err.to_string(),
                            row_data
                        );
                    }
                }
            }
            None => {}
        }
    }
    return parsed_data;
}

pub async fn get_armor(
    pool: &Data<Pool>,
    updated_on: Option<DateTime<Utc>>,
    book_ids: &Vec<u32>,
    all: bool,
) -> Vec<Armor> {
    let rows =
        get_game_data_table_data(pool, "chargen_armor".to_owned(), updated_on, book_ids, all).await;

    let mut parsed_data: Vec<Armor> = Vec::new();
    for row in rows {
        match row.data {
            Some(row_data) => {
                let data_result: Result<Armor, serde_json::Error> =
                    serde_json::from_str(row_data.as_ref());
                match data_result {
                    Ok(mut data) => {
                        data.id = row.id;
                        data.created_on = row.created_on;
                        data.updated_on = row.updated_on;
                        data.deleted_on = row.deleted_on;

                        data.deleted = row.deleted;

                        data.created_by = row.created_by;
                        data.updated_by = row.updated_by;
                        data.deleted_by = row.deleted_by;
                        // println!("{:?}", data.pf_armor_type);
                        parsed_data.push(data);
                    }
                    Err(err) => {
                        println!(
                            "Error with data on get_armor {}, {}, {}",
                            row.id,
                            err.to_string(),
                            row_data
                        );
                    }
                }
            }
            None => {}
        }
    }
    return parsed_data;
}
