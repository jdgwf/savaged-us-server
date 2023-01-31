use mysql::Pool;

use crate::db::banners::get_active_banners;
use actix_web::{get, web::Data, web::Json};
use savaged_libs::banner::SimpleBanner;

#[get("/_api/banners/get")]
pub async fn api_banners_get(pool: Data<Pool>) -> Json<Vec<SimpleBanner>> {
    return actix_web::web::Json(get_active_banners(&pool));
}
