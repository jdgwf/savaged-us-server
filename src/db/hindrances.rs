use serde::{Serialize, Deserialize};

use mysql::*;
use mysql::prelude::*;
use actix_web:: {

    // web::Json,
    web::Data,

};
use savaged_libs::player_character::hindrance::Hindrance;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SmallHindrance {
    id: u32,
    name: String,
    book_id: String,
}
pub fn get_hindrances(
    pool: Data<Pool>,
) -> Vec<SmallHindrance> {
    match pool.get_conn() {
        Ok( mut conn) => {
            let get_hindrances_result = conn
            .query_map(
                "SELECT
                    id,
                    name,
                    book_id
                from chargen_hindrances where  book_id like '9'",
                |(
                    id,
                    name,
                    book_id,
                ): (u32, String, String) | {

                    let hindrance: SmallHindrance = SmallHindrance {
                        id: id,
                        name: name,
                        book_id: book_id,
                    };
                    // let hindrance = Hindrance::default();
                    return hindrance;

                },
            );
            match get_hindrances_result {
                Ok( get_hindrances ) => {
                    return get_hindrances;
                }

                Err( err ) => {
                    println!("get_hindrances Error 4 {}", err );
                }
            }
        }
        Err( err ) => {
            println!("get_hindrances Error 3 {}", err );
        }
    }
    return Vec::new();
}