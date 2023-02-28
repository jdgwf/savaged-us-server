use actix_web::web::Data;
use mysql::prelude::*;
use mysql::*;
use savaged_libs::partner::{Partner, SimplePartner};

pub fn get_active_partners(pool: &Data<Pool>) -> Vec<SimplePartner> {
    match pool.get_conn() {
        Ok(mut conn) => {
            let get_partners_result = conn.query_map(
                "
                select
                id, data
                from partners
                where active > 0
                and deleted < 1
                -- and (start = 0 or start <= now())
                -- and (end >= now() or end = 0)
                ",
                |(id, data): (u32, String)| {
                    let mut partner: SimplePartner = serde_json::from_str(data.as_ref()).unwrap();
                    // let partner = Partner::default();
                    partner.id = id;
                    return partner;
                },
            );

            match get_partners_result {
                Ok(get_partners) => {
                    // println!("get_active_partners {}", get_partners.len());
                    return get_partners;
                }

                Err(err) => {
                    println!("get_active_partners Error 4 {}", err);
                }
            }
        }
        Err(err) => {
            println!("get_active_partners Error 3 {}", err);
        }
    }
    return Vec::new();
}

pub fn get_partners(pool: &Data<Pool>) -> Vec<Partner> {
    match pool.get_conn() {
        Ok(mut conn) => {
            let get_partners_result = conn.query_map(
                "SELECT
                    id,
                    data
                from partners where deleted < 1",
                |(id, data): (u32, String)| {
                    let mut partner: Partner = serde_json::from_str(data.as_ref()).unwrap();
                    // let partner = Partner::default();
                    partner.id = id;
                    return partner;
                },
            );
            match get_partners_result {
                Ok(get_partners) => {
                    return get_partners;
                }

                Err(err) => {
                    println!("get_partners Error 4 {}", err);
                }
            }
        }
        Err(err) => {
            println!("get_partners Error 3 {}", err);
        }
    }
    return Vec::new();
}
