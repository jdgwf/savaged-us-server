use chrono::prelude::*;

pub fn mysql_datetime_to_chrono_utc(
    date_string: String
) ->  Option<DateTime<Utc>> {

    if date_string.is_empty() {
        return None;
    } else {
        let dt_result = Utc.datetime_from_str(
            date_string.as_ref(),
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
