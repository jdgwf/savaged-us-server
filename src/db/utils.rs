use actix_web::cookie::time::PrimitiveDateTime;
use chrono::prelude::*;
use mysql::Row;


pub fn mysql_row_to_chrono_utc (
    row: &mut Row,
    field_name: &str,
) ->  Option<DateTime<Utc>> {

    let date_opt_opt = row.take_opt(field_name);


    match date_opt_opt {

        Some( date_opt ) => {
            match date_opt {

                Ok( val ) => {
                    let primitive: PrimitiveDateTime = val;

                    return mysql_datetime_to_chrono_utc(primitive.to_string().replace(".0", ""));
                }
                Err( err ) => {
                    println!("mysql_row_to_chrono_utc error {:?}", err );
                    return None;
                }

            }
        }
        None => {
            // println!("mysql_row_to_chrono_utc error {:?}", err );
            return None;
        }
    }
}

pub fn mysql_datetime_to_chrono_utc(
    date_string: String
) ->  Option<DateTime<Utc>> {
    // println!("mysql_datetime_to_chrono_utc {}", &date_string);
    if date_string.is_empty() {
        return None;
    } else {
        let dt_result = Utc.datetime_from_str(
            date_string.replace(".0", "").as_ref(),
            "%Y-%m-%d %H:%M:%S",
        );

        match dt_result {
            Ok(dt) => {
                return Some(DateTime::from_utc(dt.naive_utc(), Utc));
            }

            Err( err) => {
                println!("mysql_datetime_to_chrono_utc Parse Error {}, {}", err, date_string);
                return None;
            }
        }
        // return None;
    }

}

