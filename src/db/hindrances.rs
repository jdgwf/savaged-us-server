use serde::{Deserialize, Serialize};

use actix_web::web::Data;
use mysql_async::prelude::*;
use mysql_async::*;
use savaged_libs::player_character::hindrance::Hindrance;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SmallHindrance {
    id: u32,
    name: String,
    book_id: String,
}
pub async fn get_hindrances(pool: &Data<Pool>) -> Vec<SmallHindrance> {
    match pool.get_conn().await {
        Ok(mut conn) => {
            let get_hindrances_result = conn.query_map(
                "SELECT
                    id,
                    name,
                    book_id
                from chargen_hindrances where  book_id like '9'",
                |(id, name, book_id): (u32, String, String)| {
                    let hindrance: SmallHindrance = SmallHindrance {
                        id: id,
                        name: name,
                        book_id: book_id,
                    };
                    // let hindrance = Hindrance::default();
                    return hindrance;
                },
            ).await;
            match get_hindrances_result {
                Ok(get_hindrances) => {
                    return get_hindrances;
                }

                Err(err) => {
                    println!("get_hindrances Error 4 {}", err);
                }
            }
        }
        Err(err) => {
            println!("get_hindrances Error 3 {}", err);
        }
    }
    return Vec::new();
}
