use mysql::*;

use actix_web:: {
    get,
    web::Json,
    web::Data,

};
use savaged_libs::banner::Banner;
use super::super::db::banners::{
    get_banners,
};

#[get("/_api/banners-get")]
pub async fn banners_get(
    pool: Data<Pool>,
) -> Json<Vec<Banner>> {
    return actix_web::web::Json(get_banners( pool ).await);
}

