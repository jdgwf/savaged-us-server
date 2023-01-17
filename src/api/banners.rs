use mysql::Pool;

use actix_web:: {
    get,
    web::Json,
    web::Data,

};
use savaged_libs::banner::Banner;
use savaged_libs::player_character::hindrance::Hindrance;
use crate::db::hindrances::SmallHindrance;

use super::super::db::banners::{
    get_banners,
};
use super::super::db::hindrances::{
    get_hindrances,
};

#[get("/_api/banners/get")]
pub async fn api_banners_get(
    pool: Data<Pool>,
) -> Json<Vec<SmallHindrance>> {
    return actix_web::web::Json(get_hindrances( pool ));
}


