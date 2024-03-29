use actix_web::web::Data;
use mysql_async::prelude::*;
use mysql_async::*;
use savaged_libs::banner::{Banner, SimpleBanner};

pub async fn get_active_banners(pool: &Data<Pool>) -> Vec<SimpleBanner> {
    match pool.get_conn().await {
        Ok(mut conn) => {
            let get_banners_result = conn.query_map(
                "
                select
                id, data
                from banners
                where active > 0
                and deleted < 1
                and (start = 0 or start <= now())
                and (end >= now() or end = 0)",
                |(id, data): (u32, String)| {
                    let mut banner: SimpleBanner = serde_json::from_str(data.as_ref()).unwrap();
                    // let banner = Banner::default();
                    banner.id = id;
                    return banner;
                },
            ).await;
            match get_banners_result {
                Ok(get_banners) => {
                    return get_banners;
                }

                Err(err) => {
                    println!("get_banners Error 4 {}", err);
                }
            }
        }
        Err(err) => {
            println!("get_banners Error 3 {}", err);
        }
    }
    return Vec::new();
}

pub async fn get_banners(pool: &Data<Pool>) -> Vec<Banner> {
    match pool.get_conn().await {
        Ok(mut conn) => {
            let get_banners_result = conn.query_map(
                "SELECT
                    id,
                    data
                from banners where deleted < 1",
                |(id, data): (u32, String)| {
                    let mut banner: Banner = serde_json::from_str(data.as_ref()).unwrap();
                    // let banner = Banner::default();
                    banner.id = id;
                    return banner;
                },
            ).await;
            match get_banners_result {
                Ok(get_banners) => {
                    return get_banners;
                }

                Err(err) => {
                    println!("get_banners Error 4 {}", err);
                }
            }
        }
        Err(err) => {
            println!("get_banners Error 3 {}", err);
        }
    }
    return Vec::new();
}
