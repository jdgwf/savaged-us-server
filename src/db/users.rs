
use actix_web::cookie::time::PrimitiveDateTime;
use mysql::*;
use mysql::prelude::*;
use chrono::prelude::*;
use crate::api::notifications::get_notifications_for_user;
use crate::db::utils::mysql_row_to_chrono_utc;
use crate::utils::encrypt_password;

use actix_web::HttpRequest;
use actix_web:: {

    // web::Json,
    web::Data,

};
use savaged_libs::user::{ User, LoginToken };
use uuid::Uuid;


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

                    let user = _make_user_from_row( row );
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

                        let mut user = _make_user_from_row( row );

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

                    let user = _make_user_from_row( row );
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

fn _make_user_from_row( mut row: Row ) -> User {
    // let created_on_string: String = row.take_opt("created_on")
    //     .unwrap_or(Ok("".to_string()))
    //     .unwrap_or("".to_string());
    // let deleted_on_string: String = row.take_opt("deleted_on")
    //     .unwrap_or(Ok("".to_string()))
    //     .unwrap_or("".to_string());

    // let zombie_on_string: String = row.take_opt("zombie_on")
    //     .unwrap_or(Ok("".to_string()))
    //     .unwrap_or("".to_string());
    // let last_seen_on_string: String = row.take_opt("last_seen_on")
    //     .unwrap_or(Ok("".to_string()))
    //     .unwrap_or("".to_string());
    // // let registration_expires_string: String = row.take_opt("registration_expires")
    // //     .unwrap_or(Ok("".to_string()))
    // //     .unwrap_or("".to_string());
    // let banned_on_string: String = row.take_opt("banned_on")
    //     .unwrap_or(Ok("".to_string()))
    //     .unwrap_or("".to_string());
    // let premium_expires_string: String = row.take_opt("premium_expires")
    //     .unwrap_or(Ok("".to_string()))
    //     .unwrap_or("".to_string());
    // let reset_password_expire_string : String = row.take_opt("reset_password_expire")
    //     .unwrap_or(Ok("".to_string()))
    //     .unwrap_or("".to_string());

    let login_tokens_string: String = row.take("login_tokens").unwrap();
    let login_tokens: Vec<LoginToken> = serde_json::from_str( login_tokens_string.as_str() ).unwrap();

    let mut created_by = 0;
    let created_opt= row.take_opt("created_by").unwrap();
    match created_opt {

        Ok( val ) => {created_by = val;}
        Err( _ ) => {}

    }
    let mut updated_by = 0;
    let updated_opt = row.take_opt("updated_by").unwrap();
    match updated_opt {

        Ok( val ) => {
            println!("updated_by val {:?}", val );
            updated_by = val;
        }
        Err( err ) => {
            println!("updated_by error {:?}", err );
        }

    }
    let mut deleted_by = 0;
    let deleted_opt = row.take_opt("deleted_by").unwrap();
    match deleted_opt {

        Ok( val ) => {deleted_by = val;}
        Err( _ ) => {}

    }

    let mut share_bio = "".to_string();
    let share_bio_opt = row.take_opt("share_bio").unwrap();
    match share_bio_opt {

        Ok( val ) => {share_bio = val; }
        Err( _ ) => {}

    }


    // let mut updated_on_string: String = "".to_string();
        // let updated_on_string: String = row.take_opt("updated_on")
    //     .unwrap_or(Ok("".to_string()))
    //     .unwrap_or("".to_string());

    // let updated_on_opt = row.take_opt("updated_on").unwrap();
    // let mut updated_on: String = "".to_owned();
    // match updated_on_opt {

    //     Ok( val ) => {
    //         let naive: PrimitiveDateTime = val;

    //         updated_on = naive.to_string().replace(".0", "");
    //         // updated_on = Some(val);
    //         // println!("updated_on val {:?} {:?}", naive, &updated_on_string );
    //     }
    //     Err( _err ) => {
    //         // println!("updated_on_opt error {:?}", err );
    //     }

    // }

    let user = User{
        activated: row.take("activated").unwrap(),
        api_key: row.take("api_key").unwrap(),
        banned: row.take("banned").unwrap(),
        banned_by: row.take("banned_by").unwrap(),
        banned_on: mysql_row_to_chrono_utc(&mut row, "bannned_on"), // row.take("banned_on").unwrap(),
        banned_reason: row.take("banned_reason").unwrap(),
        created_by: created_by,
        created_on: mysql_row_to_chrono_utc(&mut row, "created_on"), // created_on_dtfo.with_timezone( &Utc),
        default_username: row.take("default_username").unwrap(),
        deleted: row.take("deleted").unwrap(),
        deleted_by: deleted_by,
        deleted_on: mysql_row_to_chrono_utc( &mut row, "deleted_on"), // row.take("deleted_on").unwrap(),
        discord_id: row.take("discord_id").unwrap(),
        email: row.take("email").unwrap(),
        first_name: row.take("first_name").unwrap(),
        group_ids: Vec::new(), //row.take("group_ids").unwrap(),
        hidden_banners: row.take("hidden_banners").unwrap(),
        id: row.take("id").unwrap(),
        is_ace: row.take("is_ace").unwrap(),
        is_admin: row.take("is_admin").unwrap(),
        is_developer: row.take("is_developer").unwrap(),
        is_premium: row.take("is_premium").unwrap(),
        last_name: row.take("last_name").unwrap(),
        last_seen_ip: row.take("last_seen_ip").unwrap(),
        last_seen_on: mysql_row_to_chrono_utc( &mut row, "last_seen_on"), // row.take("last_seen_on").unwrap(),
        lc_wildcard_reason: row.take("lc_wildcard_reason").unwrap(),
        login_tokens: login_tokens.clone(), //row.take("login_tokens").unwrap(),
        notes: "".to_string(), // row.take("notes").unwrap(),
        notify_email: row.take("notify_email").unwrap(),
        image_url: "".to_string(), // row.take("image_url").unwrap(),
        number_years: row.take("number_years").unwrap(),
        partner_id: row.take("partner_id").unwrap(),
        paypal_payment_id: row.take("paypal_payment_id").unwrap(),
        premium_expires: mysql_row_to_chrono_utc( &mut row, "premium_expires"), // row.take("premium_expires").unwrap(),
        profile_image: row.take("profile_image").unwrap(),
        reset_password_expire: mysql_row_to_chrono_utc( &mut row, "reset_password_expire"), // row.take("reset_password_expire").unwrap(),
        share_bio: share_bio,
        share_display_name: row.take("share_display_name").unwrap(),
        share_show_profile_image: row.take("share_show_profile_image").unwrap(),
        show_user_page: row.take("show_user_page").unwrap(),
        theme_css: row.take("theme_css").unwrap(),
        timezone: row.take("timezone").unwrap(),
        turn_off_advance_limits: row.take("turn_off_advance_limits").unwrap(),
        twitter: row.take("twitter").unwrap(),
        updated_by: updated_by,
        // updated_on: Central.from_local_datetime(&updated_on).with_timezone(&Utc).clone(),
        // updated_on: Some(DateTime::from_utc(DateTime::parse_from_rfc3339( &updated_on.to_string() ).unwrap().naive_utc(), Utc)),
        updated_on: mysql_row_to_chrono_utc( &mut row, "updated_on"), // updated_on_dtfo.with_timezone( &Utc),
        username: row.take("username").unwrap(),
        version_of: row.take("version_of").unwrap(),
        zombie: row.take("zombie").unwrap(),
        unread_notifications: 0, // row.take("unread_notifications").unwrap(),
        zombie_on: mysql_row_to_chrono_utc( &mut row, "zombie_on"), // row.take("zombie_on").unwrap(),
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
                    return_value.user_id = row.take("id").unwrap();
                    return_value.banned = row.take("banned").unwrap();
                    return_value.banned_reason = row.take("banned_reason").unwrap();
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