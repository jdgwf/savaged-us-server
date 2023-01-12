use actix_web::{cookie::time::PrimitiveDateTime, web::Json};
use chrono::prelude::*;
use mysql::Row;
use savaged_libs::admin_libs::FetchAdminParameters;


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

pub fn admin_current_limit_paging_sql(
    params: &Json<FetchAdminParameters>,
) -> String {


    let limit = format!("\nLIMIT {}, {}", params.current_page  * params.number_per_page, params.number_per_page);
    match &params.sort_by {
        Some( sort_by ) => {
            let mut sort_dir = "DESC".to_owned();
            if( params.sort_by_ascending ) {
                sort_dir = "ASC".to_owned();
            }
            return format!("{}\nSORT BY `{}`, {}\n", limit, sort_by, sort_dir);
        }
        None => {
            return limit
        }
    }
}

pub fn admin_filter_where_clause(
    search_fields: &'static [&'static str],
    params: &Json<FetchAdminParameters>,
) -> String {


    match &params.filter {
        Some( filter ) => {
            let mut rv = "".to_string();
            if filter.trim() != "" && search_fields.len() > 0 {
                rv += "\nAND\n (
                    '1' = '2'
                ";


                for field in search_fields.iter() {
                    rv += format!("\tOR `{}` like '%{}%' ", field, filter).as_str();
                }

                rv += ")\n";
            }

            return rv;
        }
        None => {
            return "".to_string();
        }
    }
}


