use crate::db::utils::mysql_datetime_to_chrono_utc;
use actix_web::web::Data;
use chrono::prelude::*;
use mysql_async::prelude::*;
use mysql_async::Pool;
use savaged_libs::book::Book;
use savaged_libs::utils::bool_from_int_or_bool;
use serde::{Deserialize, Serialize};

pub async fn get_books(
    pool: &Data<Pool>,
    current_user_id: u32,
    updated_on: Option<DateTime<Utc>>,
    access_registered: bool,
    access_wildcard: bool,
    access_developer: bool,
    access_admin: bool,
    all: bool,
) -> Vec<Book> {
    let mut data_query = "
    SELECT
        id,
        data,

        created_on,
        created_by,

        updated_on,
        updated_by,

        deleted,
        deleted_on,
        deleted_by

        from books

        where  version_of = 0
        and deleted < 1
"
    .to_owned();

    if !all {
        data_query += &" and `books`.`active` > 0\n";
    }
    data_query += &" and ((\n";
    if access_admin {
        data_query += &" `books`.`access_admin` > 0\n";
    } else if access_developer {
        data_query += &" `books`.`access_developer` > 0\n";
    } else if access_wildcard {
        data_query += &" `books`.`access_wildcard` > 0\n";
    } else if access_registered {
        data_query += &" `books`.`access_registered` > 0\n";
    } else {
        data_query += &" `books`.`access_anonymous` > 0\n";
    }
    data_query += format!(" ) or  `books`.`created_by` like {})", current_user_id).as_ref();


    match updated_on {
        Some( updated_date_string ) => {
            data_query = format!("{}\n  AND `updated_on` > '{}'", data_query, updated_date_string);
        }
        None => {}
    }
    // println!("{}", data_query);
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

                    match data {
                        Some(row_data) => {
                            let book_result: Result<Book, serde_json::Error> =
                                serde_json::from_str(row_data.as_ref());
                            match book_result {
                                Ok(mut book) => {
                                    book.created_on = mysql_datetime_to_chrono_utc(created_on);
                                    book.updated_on = mysql_datetime_to_chrono_utc(updated_on);
                                    book.deleted_on = mysql_datetime_to_chrono_utc(deleted_on);
                                    book.created_by = created_by;
                                    book.deleted = deleted_bool;
                                    book.deleted_by = deleted_by;
                                    book.updated_by = updated_by;
                                    book.id = id;

                                    return book;
                                }
                                Err(err) => {
                                    println!(
                                        "Error with data on book {}, {}, {}",
                                        id,
                                        err.to_string(),
                                        row_data
                                    );
                                    return Book::default();
                                }
                            }
                        }
                        None => {
                            return Book::default();
                        }
                    }
                },
            ).await;
            match get_row_data_result {
                Ok(get_row_data) => {
                    return get_row_data;
                }

                Err(err) => {
                    println!("get_books Error 4 {}", err);
                }
            }
        }
        Err(err) => {
            println!("get_books Error 3 {}", err);
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
