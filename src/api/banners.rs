use mysql::Pool;

use actix_web:: {
    get,
    web::Json,
    web::Data,

};
use savaged_libs::banner::SimpleBanner;
use crate::db::banners::get_active_banners;

#[get("/_api/banners/get")]
pub async fn api_banners_get(
    pool: Data<Pool>,
) -> Json<Vec<SimpleBanner>> {
    return actix_web::web::Json(get_active_banners( pool ));
}

