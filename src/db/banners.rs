use mysql::*;
use mysql::prelude::*;
use actix_web:: {

    // web::Json,
    web::Data,

};
use savaged_libs::banner::{ Banner };

pub fn get_banners(
    pool: Data<Pool>,
) -> Vec<Banner> {
    match pool.get_conn() {
        Ok( mut conn) => {
            let get_banners_result = conn
            .query_map(
                "SELECT
                    id,
                    data
                from banners",
                |(
                    id,
                    data,
                ): (u32, String) | {

                    let mut banner: Banner = serde_json::from_str( data.as_ref() ).unwrap();
                    // let banner = Banner::default();
                    banner.id = id;
                    return banner;

                },
            );
            match get_banners_result {
                Ok( get_banners ) => {
                    return get_banners;
                }

                Err( err ) => {
                    println!("get_banners Error 4 {}", err );
                }
            }
        }
        Err( err ) => {
            println!("get_banners Error 3 {}", err );
        }
    }
    return Vec::new();
}