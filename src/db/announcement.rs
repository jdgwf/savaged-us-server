use actix_web::web::Data;
use mysql::prelude::*;
use mysql::*;
use savaged_libs::announcement::{Announcement, SimpleAnnouncement};

pub fn get_active_announcements(pool: &Data<Pool>) -> Vec<SimpleAnnouncement> {
    match pool.get_conn() {
        Ok(mut conn) => {
            let get_announcements_result = conn.query_map(
                "
                select
                id, data
                from announcements
                where active > 0
                and deleted < 1
                -- and (start = 0 or start <= now())
                -- and (end >= now() or end = 0)
                ",
                |(id, data): (u32, String)| {
                    let mut announcement: SimpleAnnouncement = serde_json::from_str(data.as_ref()).unwrap();
                    // let announcement = Announcement::default();
                    announcement.id = id;
                    return announcement;
                },
            );
            match get_announcements_result {
                Ok(get_announcements) => {
                    return get_announcements;
                }

                Err(err) => {
                    println!("get_announcements Error 4 {}", err);
                }
            }
        }
        Err(err) => {
            println!("get_announcements Error 3 {}", err);
        }
    }
    return Vec::new();
}

pub fn get_announcements(pool: &Data<Pool>) -> Vec<Announcement> {
    match pool.get_conn() {
        Ok(mut conn) => {
            let get_announcements_result = conn.query_map(
                "SELECT
                    id,
                    data
                from announcements where deleted < 1",
                |(id, data): (u32, String)| {
                    let mut announcement: Announcement = serde_json::from_str(data.as_ref()).unwrap();
                    // let announcement = Announcement::default();
                    announcement.id = id;
                    return announcement;
                },
            );
            match get_announcements_result {
                Ok(get_announcements) => {
                    return get_announcements;
                }

                Err(err) => {
                    println!("get_announcements Error 4 {}", err);
                }
            }
        }
        Err(err) => {
            println!("get_announcements Error 3 {}", err);
        }
    }
    return Vec::new();
}
