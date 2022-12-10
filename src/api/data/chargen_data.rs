use crate::{db::{chargen_data::{get_chargen_data, get_hindrances}, users::get_remote_user}, api::auth::ApiKeyOrToken};
use mysql::Pool;
use actix_web:: {
    get,
    post,
    web::Json,
    web::Data, HttpRequest,

};
use savaged_libs::player_character::{hindrance::Hindrance, chargen_data::ChargenData};

#[get("/_api/hindrances-get")]
pub async fn hindrances_get(
    pool: Data<Pool>,
) -> Json<Vec<Hindrance>> {

    let parsed_data = get_hindrances(&pool, None, &Vec::new(), false);

    return actix_web::web::Json(
        parsed_data
    );
}

#[post("/_api/chargen-data-get")]
pub async fn chargen_data_get(
    pool: Data<Pool>,
    form: Json<ApiKeyOrToken>,
    body: String,
    request: HttpRequest,
) -> Json<ChargenData> {

    let mut login_token: Option<String> = None;
    let mut api_key: Option<String> = None;
    match &form.login_token {
        Some( val ) => {
            login_token = Some(val.to_owned());
        }
        None => {}
    }
    match &form.api_key {
        Some( val ) => {
            api_key = Some(val.to_owned());
        }
        None => {}
    }

    // println!("form {:?}", form);
    // println!("body {:?}", body);
    // println!("api_key {:?}", api_key);
    // println!("login_token {:?}", login_token);

    let current_user = get_remote_user(
        pool.clone(),
        api_key,
        login_token,
        request,
    );

    let mut access_registered = false;
    let mut access_wildcard = false;
    let mut access_developer = false;
    let mut access_admin = false;

    match current_user {
        Some( user ) => {

            access_registered = true;

            if user.has_premium_access() {
                access_wildcard = true;
            }
            if user.has_developer_access() {
                access_developer = true;
            }
            if user.has_admin_access() {
                access_admin = true;
            }
        }
        None => {

        }
    }

    let chargen_data = get_chargen_data(
        &pool,
        0,
        None,
        access_registered,
        access_wildcard,
        access_developer,
        access_admin,
        false,
    );

    return actix_web::web::Json(
        chargen_data
    );
}
