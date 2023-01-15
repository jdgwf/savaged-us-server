
use actix_web::cookie::time::PrimitiveDateTime;
use actix_web::web::Json;
use mysql::*;
use mysql::prelude::*;
use chrono::prelude::*;
use savaged_libs::admin_libs::{FetchAdminParameters, AdminPagingStatistics};
use crate::api::notifications::get_notifications_for_user;
use crate::db::utils::{mysql_row_to_chrono_utc, admin_filter_where_clause};
use crate::utils::encrypt_password;

use actix_web::HttpRequest;
use actix_web:: {

    // web::Json,
    web::Data,

};
use savaged_libs::user::{ User, LoginToken };
use uuid::Uuid;

use super::utils::admin_current_limit_paging_sql;

const USER_SEARCH_FIELDS: &'static [&'static str]  = &[
    "first_name",
    "last_name",
    "email",
    "username",
];

// HINT: conn.last_insert_id()
// HINT: conn.rows_affected()
pub fn create_login_token(
    pool: Data<Pool>,
    user_id: u32,
    browser: String,
    ip_address: String,
) -> Option<String> {

    let user = get_user(pool.clone(), user_id).unwrap();

    let mut login_tokens = user.login_tokens;

    let new_token = user.id.to_string() + &"x".to_owned() + &Uuid::new_v4().to_string() + &"-".to_owned() + &Uuid::new_v4().to_string();

    login_tokens.push(
        LoginToken {
            token: new_token.clone(),
            last_seen: chrono::offset::Utc::now(),
            registered: chrono::offset::Utc::now(),
            browser: browser.clone(),
            last_seen_ip: ip_address.clone(),
            friendly_name: "".to_owned(),
            logged_out: false,
        }
    );

    let login_token_str = serde_json::to_string(&login_tokens).unwrap();
    match pool.clone().get_conn() {
        Ok( mut conn) => {

            let _: Option<Row>  = conn.exec_first(
                "update `users` set `login_tokens` = :login_tokens  where `id` = :user_id",
                params!{ "user_id" => user_id, "login_tokens" => login_token_str }
            ).unwrap();

            return Some(new_token.to_owned());
            // match found_user_result {
            //     Some(  mut row ) => {
            //         return Some("".to_owned());
            //     }
            //     None => {
            //         return None;
            //     }
            // }
        }

        Err( _err ) => {

        }
    }
    return None;
}

pub fn update_user_login_tokens(
    pool: Data<Pool>,
    user_id: u32,
    login_tokens: Vec<LoginToken>,
) -> Option<Vec<LoginToken>> {

    let login_token_str = serde_json::to_string(&login_tokens).unwrap();
    match pool.clone().get_conn() {
        Ok( mut conn) => {

            let _: Option<Row>  = conn.exec_first(
                "update `users` set `login_tokens` = :login_tokens  where `id` = :user_id",
                params!{ "user_id" => user_id, "login_tokens" => login_token_str }
            ).unwrap();

            return Some(login_tokens.to_owned());
        }

        Err( _err ) => {

        }
    }
    return None;
}

pub fn get_user(
    pool: Data<Pool>,
    user_id: u32
) -> Option<User> {

    match pool.get_conn() {
        Ok( mut conn) => {

            let found_user_result: Option<Row> = conn.exec_first(
                "select * from `users` where `id` = :user_id",
                params!{ "user_id" => user_id}
            ).unwrap();
            match found_user_result {
                Some(  row ) => {

                    let mut user = make_user_from_row( row, "".to_owned() );

                    let mut new_count = 0;
                    for msg in &get_notifications_for_user(pool.clone(), user.id) {
                        if msg.read < 1 {
                            new_count += 1;
                        }
                    }
                    user.unread_notifications = new_count;

                    return Some(user);
                }
                None => {
                    return None;
                }
                // Err( err ) => {
                //     println!("login_for_token Error 1 {}", err );
                // }
            }

        }
        Err( err ) => {
            println!("login_for_token Error 3 {}", err );
        }

    }
    return None;
}

pub fn admin_get_users(
    pool: Data<Pool>,
    paging_params: Json<FetchAdminParameters>,
) -> Vec<User> {

    let mut data_query = format!("
        SELECT * from `users`
        WHERE 1 = 1
    ");

    let paging = admin_current_limit_paging_sql( &paging_params );
    // let data_params = params!{
    //      "1" => 1
    // };

    data_query = data_query + admin_filter_where_clause(
        USER_SEARCH_FIELDS,
        &paging_params,
        false,
        false,
    ).as_str();

    data_query = data_query + &paging;

    println!("admin_get_users data_query:\n{}", data_query);

    match pool.get_conn() {
        Ok( mut conn) => {

            let saves_result: Result<Vec<Row>> = conn.exec(
                data_query,
                // data_params,
                (),
            );
            match saves_result {
                Ok(  rows ) => {

                    let mut rv: Vec<User> = Vec::new();
                    for row in rows {
                        // let row_data = _make_save_from_row(row, with_cached_data);

                        // if (&row_data.name).to_owned() == "Chi Master".to_owned() {
                        //     println!("row_data {:?}", row_data );
                        // }

                        let mut user = make_user_from_row( row, "".to_owned() );
                        rv.push( user );
                    }

                    // println!("rv.len {}", rv.len() );
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

pub fn admin_get_users_paging_data(
    pool: Data<Pool>,
    paging_params: Json<FetchAdminParameters>,
) -> AdminPagingStatistics {

    let mut paging: AdminPagingStatistics = AdminPagingStatistics {
        non_filtered_count: 0,
        filtered_count: 0,
        book_list: None,
    };

    let mut data_query = format!("
        SELECT count(id) as `count` from `users`
        WHERE 1 = 1
    ");

    // let paging = admin_current_limit_paging_sql( paging_params );
    // let data_params = params!{
    //      "1" => 1
    // };

    // data_query = data_query + &paging;

    // println!("admin_get_users_paging_data 1 data_query:\n{}", data_query);

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

                    // let mut rv: Vec<User> = Vec::new();
                    // for row in rows {
                    //     // let row_data = _make_save_from_row(row, with_cached_data);

                    //     // if (&row_data.name).to_owned() == "Chi Master".to_owned() {
                    //     //     println!("row_data {:?}", row_data );
                    //     // }

                    //     let mut user = make_user_from_row( row );
                    //     rv.push( user );
                    // }

                    // paging

                }
                Err( err ) => {
                    println!("get_user_saves Error 4 {}", err );
                    // return Vec::new();
                }

            }

        }
        Err( err ) => {
            // println!("get_user_saves Error 3 {}", err );
        }
    }
    let mut data_query = format!("
        SELECT count(id) as `count` from `users`
        WHERE 1 = 1
    ");

    // let paging = admin_current_limit_paging_sql( paging_params );
    // let data_params = params!{
    //      "1" => 1
    // };

    // data_query = data_query + &paging;
    data_query = data_query + admin_filter_where_clause(
        USER_SEARCH_FIELDS,
        &paging_params,
        false,
        false,
    ).as_str();
//
    // println!("admin_get_users_paging_data 2 data_query:\n{}", data_query);

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

                    //     let mut user = make_user_from_row( row );
                    //     rv.push( user );
                    // }

                    // paging

                }
                Err( err ) => {
                    println!("get_user_saves Error 4 {}", err );
                    // return Vec::new();
                }

            }

        }
        Err( err ) => {
            // println!("get_user_saves Error 3 {}", err );
        }
    }

    return paging;
}

pub fn get_user_from_login_token(
    pool: Data<Pool>,
    token: Option<String>,
    _request: HttpRequest,
) -> Option<User> {

    match &token {
        Some( token_match ) => {
            // Login Tokens are at least 75 characters longs
            if token_match.is_empty() {
                return None;
            }
            if token_match.len() < 75 {
                println!("get_user_from_login_token token length too small! {}", &token_match);
                return None;
            }
        }
        None => {
            // println!("get_user_from_login_token no token provided!");
            return None;
        }
    }

    match pool.get_conn() {
        Ok( mut conn) => {

            if token != None {
                let token = token.unwrap().clone();

                let found_user_result: Option<Row> = conn.exec_first(
                    "SELECT * FROM `users` where (`version_of` < 1 and `deleted` < 1 and `activated` > 0) and (

                    `login_tokens` like :token

                    ) limit 1",
                    params!{ "token" => "%".to_owned() + &token + &"%".to_owned()}
                ).unwrap();
                match found_user_result {
                    Some(  row ) => {

                        let mut user = make_user_from_row( row, "".to_owned() );

                        let mut new_count = 0;
                        for msg in &get_notifications_for_user(pool.clone(), user.id) {
                            if msg.read < 1 {
                                new_count += 1;
                            }
                        }
                        user.unread_notifications = new_count;

                        return Some(user);
                    }
                    None => {
                        return None;
                    }
                    // Err( err ) => {
                    //     println!("login_for_token Error 1 {}", err );
                    // }
                }
            }

        }
        Err( err ) => {
            println!("login_for_token Error 3 {}", err );
        }

    }
    return None;
}

pub fn get_user_from_api_key(
    pool: Data<Pool>,
    api_key: String,
    _request: HttpRequest,
) -> Option<User> {

    match pool.get_conn() {
        Ok( mut conn) => {
            let api_key = api_key.clone();
            let found_user_result: Option<Row> = conn.exec_first(
                "SELECT * FROM `users` where (`version_of` < 1 and `deleted` < 1 and `activated` > 0) and (

                `api_key` like :api_key

                ) limit 1",
                params!{ "api_key" => api_key}
            ).unwrap();
            match found_user_result {
                Some(  row ) => {

                    let user = make_user_from_row( row, "".to_owned() );
                    return Some(user);
                }
                None => {
                    return None;
                }
                // Err( err ) => {
                //     println!("login_for_token Error 1 {}", err );
                // }
            }

        }
        Err( err ) => {
            println!("login_for_token Error 3 {}", err );
        }

    }
    return None;
}

pub fn get_remote_user(
    pool: Data<Pool>,
    api_key: Option<String>,
    token: Option<String>,
    request: HttpRequest,
) -> Option<User> {

    let conn_info = request.connection_info();

    let mut real_remote_addy = "".to_string();
    let mut user_agent = "".to_string();
    let mut x_forwarded_for = "".to_string();

    let real_remote_addy_option = conn_info.realip_remote_addr();
    match real_remote_addy_option {
        Some( val ) => {
            real_remote_addy = val.to_string();
        }
        None => {

        }
    }

    let user_agent_option = request.headers().get("user-agent");
    match user_agent_option {
        Some( val ) => {
            user_agent = format!("{:?}", val).to_string().replace("\"", "");
        }
        None => {

        }
    }

    let x_forwarded_for_option = request.headers().get("x-forwarded-for");
    match x_forwarded_for_option {
        Some( val ) => {
            x_forwarded_for = format!("{:?}", val).to_string().replace("\"", "");
        }
        None => {

        }
    }

    // println!("real_remote_addy {}", real_remote_addy);
    // println!("user_agent {}", user_agent);
    // println!("x_forwarded_for {}", x_forwarded_for);

    if !x_forwarded_for.is_empty() {
        real_remote_addy = x_forwarded_for;
    }

    if token != None && !token.as_ref().unwrap().is_empty() {
        let token_user_result = get_user_from_login_token(
            pool.clone(),
            token.to_owned(),
            request.clone()
        );
        match token_user_result {
            Some( user ) => {

                return Some(
                    _update_user_last_seen(
                        pool.clone(),
                        user.clone(),
                        token.unwrap().to_owned(),
                        user_agent.to_owned(),
                        real_remote_addy.to_owned()
                    )
                );
            }
            None => {

            }
        }
    } else {
        if api_key != None && !api_key.as_ref().unwrap().is_empty() {
            let api_key_result = get_user_from_api_key(
                pool.clone(),
                api_key.unwrap().to_owned(),
                request.clone()
            );
            match api_key_result {
                Some( user ) => {

                    return Some(
                        _update_user_last_seen(
                            pool.clone(),
                            user.clone(),
                            "".to_owned(),
                            user_agent.to_owned(),
                            real_remote_addy.to_owned()
                        )
                    );
                }
                None => {

                }
            }
        }
    }

    return None;
}

pub fn update_user(
    pool: Data<Pool>,
    user: User,
) -> u64 {
    // println!("update_user (db) called ");
    match pool.get_conn() {
        Ok( mut conn) => {

        let sql = "UPDATE `users` set
            `first_name` = :first_name,
            `last_name` = :last_name,
            `email` = :email,
            `api_key` = :api_key,
            `hidden_banners` = :hidden_banners,
            `turn_off_advance_limits` = :turn_off_advance_limits,
            `notify_email` = :notify_email,
            `theme_css` = :theme_css,
            `profile_image` = :profile_image,
            `twitter` = :twitter,
            `show_user_page` = :show_user_page,
            `share_display_name` = :share_display_name,
            `share_show_profile_image` = :share_show_profile_image,
            `share_bio` = :share_bio,
            `timezone` = :timezone,
            `updated_on` = now(),
            `updated_by` =:updated_by

            WHERE `id` = :id

            limit 1";

            let params = params!{
                "first_name" => &user.first_name,
                "last_name" => &user.last_name,
                "email" => &user.email,
                "api_key" => &user.api_key,
                "hidden_banners" => &user.hidden_banners,
                "turn_off_advance_limits" => &user.turn_off_advance_limits,
                "notify_email" => &user.notify_email,
                "theme_css" => &user.theme_css,
                "profile_image" => &user.profile_image,
                "twitter" => &user.twitter,
                "show_user_page" => &user.show_user_page,
                "share_display_name" => &user.share_display_name,
                "share_bio" => &user.share_bio,
                "share_show_profile_image" => &user.share_show_profile_image,
                "timezone" => &user.timezone,
                "updated_by" => &user.updated_by,
                "id" => &user.id,
            };

            conn.exec_drop( sql, params ).unwrap();

            return conn.affected_rows();
        }
        Err( err ) => {
            // println!("update_user Error 3 {}", err );
            return 0;
        }

    }
}

pub fn username_available(
    pool: Data<Pool>,
    user: User,
    username: String,
) -> bool {
    match pool.get_conn() {
        Ok( mut conn) => {

            let sql = "select `id` from `users`
            where
                `id` != :id
                and
                `username` like :username";

            let params = params!{
                "username" => &username,
                "id" => &user.id,
            };

            let rows: Option<Row> = conn.exec_first( sql, params ).unwrap();
            match rows {
                Some( _row ) => {
                    return false;
                }
                None => {
                    return true;
                }
            }

        }
        Err( err ) => {
            println!("username_available Error 3 {}", err );
        }

    }
    return false;
}

pub fn save_username(
    pool: Data<Pool>,
    user: User,
    username: String,
) -> u64 {
    match pool.get_conn() {
        Ok( mut conn) => {

            let sql = "update `users` SET
            `username` = :username
            where
                `id` = :id
            LIMIT 1
            ";

            let params = params!{
                "username" => &username,
                "id" => &user.id,
            };

            conn.exec_drop( sql, params ).unwrap();

            return conn.affected_rows();
        }
        Err( err ) => {
            println!("save_username Error 3 {}", err );
            return 0;
        }

    }

}

pub fn update_password(
    pool: Data<Pool>,
    user: User,
    new_password: Option<String>,
) -> u64 {
    println!("update_password (db) called {:?}", new_password);
    match pool.get_conn() {
        Ok( mut conn) => {

            let sql = "UPDATE `users` set
            `password` = :password,

            `updated_on` = now(),
            `updated_by` =:updated_by

            WHERE `id` = :id

            limit 1";

            let params = params!{
                "password" => &new_password,

                "updated_by" => &user.updated_by,
                "id" => &user.id,
            };

            conn.exec_drop( sql, params ).unwrap();

            return conn.affected_rows();
        }
        Err( err ) => {
            println!("update_password Error 3 {}", err );
            return 0;
        }

    }
}

fn _update_user_last_seen(
    pool: Data<Pool>,
    user: User,
    the_token: String,
    user_agent: String,
    real_remote_addy: String,
) -> User {

    let mut updated_tokens = user.login_tokens.clone();

    if !the_token.is_empty() {
        for token in  &mut updated_tokens.iter_mut() {
            if token.token == the_token {
                token.last_seen = Utc::now();
                token.last_seen_ip = real_remote_addy.to_owned();
                token.browser = user_agent.to_owned();
            }
        }
    }
    let mut altered_user = user.clone();

    let mut new_count = 0;
    for msg in &get_notifications_for_user(pool.clone(), user.id) {
        if msg.read < 1 {
            new_count += 1;
        }
    }

    altered_user.unread_notifications = new_count;

    altered_user.login_tokens = updated_tokens.clone();
    let login_token_str = serde_json::to_string( &updated_tokens ).unwrap();
    match pool.clone().get_conn() {
        Ok( mut conn) => {

            let _: Option<Row>  = conn.exec_first(
                "update `users` set
                `login_tokens` = :login_tokens,
                `last_seen_on` = now(),
                `last_seen_ip` = :last_seen_ip,
                `last_seen_browser` = :last_seen_browser
                where `id` = :user_id",
                params!{
                    "user_id" => user.id,
                    "login_tokens" => login_token_str,
                    "last_seen_ip" => real_remote_addy,
                    "last_seen_browser" => user_agent,
                }
            ).unwrap();

            return altered_user.clone();
        }

        Err( _err ) => {
            return altered_user.clone();
        }
    }
}

pub fn make_user_from_row(
    mut row: Row,
    prefix: String,
) -> User {

    let mut login_tokens_string = "".to_string();
    let login_tokens_string_opt = row.take_opt( (prefix.to_owned() + &"login_tokens").as_str() ).unwrap();
    match login_tokens_string_opt {

        Ok( val ) => {login_tokens_string = val; }
        Err( _ ) => {}

    }
    let login_tokens: Vec<LoginToken> = serde_json::from_str( login_tokens_string.as_str() ).unwrap_or( Vec::new() );

    let mut created_by = 0;
    let created_opt= row.take_opt( (prefix.to_owned() + &"created_by").as_str() ).unwrap();
    match created_opt {

        Ok( val ) => {created_by = val;}
        Err( _ ) => {}

    }
    let mut updated_by = 0;
    let updated_opt = row.take_opt( (prefix.to_owned() + &"updated_by").as_str() ).unwrap();
    match updated_opt {

        Ok( val ) => {
            // println!("updated_by val {:?}", val );
            updated_by = val;
        }
        Err( err ) => {
            println!("updated_by error {:?}", err );
        }

    }
    let mut deleted_by = 0;
    let deleted_opt = row.take_opt( (prefix.to_owned() + &"deleted_by").as_str() ).unwrap();
    match deleted_opt {

        Ok( val ) => {deleted_by = val;}
        Err( _ ) => {}

    }

    let mut share_bio = "".to_string();
    let share_bio_opt = row.take_opt( (prefix.to_owned() + &"share_bio").as_str() ).unwrap();
    match share_bio_opt {

        Ok( val ) => {share_bio = val; }
        Err( _ ) => {}

    }

    let mut hidden_banners = "".to_string();
    let hidden_banners_opt = row.take_opt( (prefix.to_owned() + &"hidden_banners").as_str() ).unwrap();
    match hidden_banners_opt {

        Ok( val ) => {hidden_banners = val; }
        Err( _ ) => {}

    }

    let mut profile_image = "".to_string();
    let profile_image_opt = row.take_opt( (prefix.to_owned() + &"profile_image").as_str() ).unwrap();
    match profile_image_opt {

        Ok( val ) => {profile_image = val; }
        Err( _ ) => {}

    }

    let mut timezone = "".to_string();
    let timezone_opt = row.take_opt( (prefix.to_owned() + &"timezone").as_str() ).unwrap();
    match timezone_opt {

        Ok( val ) => {timezone = val; }
        Err( _ ) => {}

    }

    let mut last_seen_ip= "".to_string();
    let last_seen_ip_opt = row.take_opt( (prefix.to_owned() + &"last_seen_ip").as_str() ).unwrap();
    match last_seen_ip_opt {

        Ok( val ) => {last_seen_ip = val; }
        Err( _ ) => {}

    }

    let mut discord_id= "".to_string();
    let discord_id_opt = row.take_opt( (prefix.to_owned() + &"discord_id").as_str() ).unwrap();
    match discord_id_opt {

        Ok( val ) => {discord_id = val; }
        Err( _ ) => {}

    }

    let user = User{
        activated: row.take( ( prefix.to_owned() + &"activated").as_str() ).unwrap(),
        api_key: row.take( ( prefix.to_owned() + &"api_key").as_str() ).unwrap(),
        banned: row.take( ( prefix.to_owned() + &"banned").as_str() ).unwrap(),
        banned_by: row.take( ( prefix.to_owned() + &"banned_by").as_str() ).unwrap(),
        banned_on: mysql_row_to_chrono_utc(&mut row, "banned_on"), // row.take( ( prefix.to_owned() + &"banned_on").unwrap(),
        banned_reason: row.take( ( prefix.to_owned() + &"banned_reason").as_str() ).unwrap(),
        created_by: created_by,
        created_on: mysql_row_to_chrono_utc(&mut row, "created_on"), // created_on_dtfo.with_timezone( &Utc),
        default_username: row.take( ( prefix.to_owned() + &"default_username").as_str() ).unwrap(),
        deleted: row.take( ( prefix.to_owned() + &"deleted").as_str() ).unwrap(),
        deleted_by: deleted_by,
        deleted_on: mysql_row_to_chrono_utc( &mut row, "deleted_on"), // row.take( ( prefix.to_owned() + &"deleted_on").unwrap(),
        discord_id: discord_id,
        email: row.take( ( prefix.to_owned() + &"email").as_str() ).unwrap(),
        first_name: row.take( ( prefix.to_owned() + &"first_name").as_str() ).unwrap(),
        group_ids: Vec::new(), //row.take( ( prefix.to_owned() + &"group_ids").unwrap(),
        hidden_banners: hidden_banners.clone(),
        id: row.take( ( prefix.to_owned() + &"id").as_str() ).unwrap(),
        is_ace: row.take( ( prefix.to_owned() + &"is_ace").as_str() ).unwrap(),
        is_admin: row.take( ( prefix.to_owned() + &"is_admin").as_str() ).unwrap(),
        is_developer: row.take( ( prefix.to_owned() + &"is_developer").as_str() ).unwrap(),
        is_premium: row.take( ( prefix.to_owned() + &"is_premium").as_str() ).unwrap(),
        last_name: row.take( ( prefix.to_owned() + &"last_name").as_str() ).unwrap(),
        last_seen_ip: last_seen_ip,
        last_seen_on: mysql_row_to_chrono_utc( &mut row, "last_seen_on"), // row.take( ( prefix.to_owned() + &"last_seen_on").unwrap(),
        lc_wildcard_reason: row.take( ( prefix.to_owned() + &"lc_wildcard_reason").as_str() ).unwrap(),
        login_tokens: login_tokens.clone(), //row.take( ( prefix.to_owned() + &"login_tokens").unwrap(),
        notes: "".to_string(), // row.take( ( prefix.to_owned() + &"notes").unwrap(),
        notify_email: row.take( ( prefix.to_owned() + &"notify_email").as_str() ).unwrap(),
        image_url: "".to_string(), // row.take( ( prefix.to_owned() + &"image_url").unwrap(),
        number_years: row.take( ( prefix.to_owned() + &"number_years").as_str() ).unwrap(),
        partner_id: row.take( ( prefix.to_owned() + &"partner_id").as_str() ).unwrap(),
        paypal_payment_id: row.take( ( prefix.to_owned() + &"paypal_payment_id").as_str() ).unwrap(),
        premium_expires: mysql_row_to_chrono_utc( &mut row, "premium_expires"), // row.take( ( prefix.to_owned() + &"premium_expires").unwrap(),
        profile_image: profile_image,
        reset_password_expire: mysql_row_to_chrono_utc( &mut row, "reset_password_expire"), // row.take( ( prefix.to_owned() + &"reset_password_expire").unwrap(),
        share_bio: share_bio,
        share_display_name: row.take( ( prefix.to_owned() + &"share_display_name").as_str() ).unwrap(),
        share_show_profile_image: row.take( ( prefix.to_owned() + &"share_show_profile_image").as_str() ).unwrap(),
        show_user_page: row.take( ( prefix.to_owned() + &"show_user_page").as_str() ).unwrap(),
        theme_css: row.take( ( prefix.to_owned() + &"theme_css").as_str() ).unwrap(),
        timezone: timezone,
        turn_off_advance_limits: row.take( ( prefix.to_owned() + &"turn_off_advance_limits").as_str() ).unwrap(),
        twitter: row.take( ( prefix.to_owned() + &"twitter").as_str() ).unwrap(),
        updated_by: updated_by,
        // updated_on: Central.from_local_datetime(&updated_on).with_timezone(&Utc).clone(),
        // updated_on: Some(DateTime::from_utc(DateTime::parse_from_rfc3339( &updated_on.to_string() ).unwrap().naive_utc(), Utc)),
        updated_on: mysql_row_to_chrono_utc( &mut row, "updated_on"), // updated_on_dtfo.with_timezone( &Utc),
        username: row.take( ( prefix.to_owned() + &"username").as_str() ).unwrap(),
        version_of: row.take( ( prefix.to_owned() + &"version_of").as_str() ).unwrap(),
        zombie: row.take( ( prefix.to_owned() + &"zombie").as_str() ).unwrap(),
        unread_notifications: 0, // row.take( ( prefix.to_owned() + &"unread_notifications").unwrap(),
        zombie_on: mysql_row_to_chrono_utc( &mut row, "zombie_on"), // row.take( ( prefix.to_owned() + &"zombie_on").unwrap(),
    };
    user.get_image("");
    return user;
}

pub struct LoginResult {
    pub user_id: u32,
    pub banned: bool,
    pub banned_reason: String,
    pub error: String,
}

pub fn log_user_in(
    pool: Data<Pool>,
    email: String,
    password: String,
) -> LoginResult {

    // println!("email {}", form.email.to_owned() );
    // println!("sha_secret_key {}", sha_secret_key.to_owned() );
    // println!("password {}", password.to_owned() );

    // let mut hasher = Sha224::new();
    // hasher.update( password.to_owned());
    // hasher.update( sha_secret_key.to_owned() );
    // let data = hasher.finalize();
    // let encrypted_pass= format!("b'{}'", base64::encode(data) );

    let encrypted_pass = encrypt_password( password );

    let mut return_value = LoginResult {
        user_id: 0,
        banned: false,
        banned_reason: "".to_owned(),
        error: "Database Error".to_owned(),
    };

    match pool.get_conn() {
        Ok( mut conn) => {
            // let selected_payments_result = conn
            // .query_map(
            //     "SELECT * from users where `email` like ? and password = ? limit 1",
            //     |(id, name,)| {
            //         User { id, name, }
            //     },
            // );
            // match selected_payments_result {
            //     Ok( selected_payments ) => {
            //         // return actix_web::web::Json( selected_payments );
            //     }

            //     Err( err ) => {
            //         println!("login_for_token Error 4 {}", err );
            //     }
            // }
            let found_user_result: Option<Row> = conn.exec_first("select id, banned, banned_reason from `users` where `email` like ? and password = ?", (email.to_owned(), encrypted_pass.to_owned())).unwrap();
            match found_user_result {
                Some(  mut row ) => {
                    return_value.error = "".to_string();
                    return_value.user_id = row.take(  "id" ).unwrap();
                    return_value.banned = row.take(  "banned" ).unwrap();
                    return_value.banned_reason = row.take( "banned_reason" ).unwrap();
                }
                None => {
                    return_value.error = "".to_string();
                }
                // Error( err ) => {

                // }
            }
        }
        Err( err ) => {
            println!("login_for_token Error 3 {}", err );
            return_value.error = format!("login_for_token Error 2 {}", err );
        }
    }

    return return_value;

}