use savaged_libs::web_content::WebContent;

use self::{help_articles::get_active_help_articles, announcement::get_active_announcements, partners::get_active_partners, banners::get_active_banners};
use mysql::Pool;
pub mod admin_data;
pub mod banners;
pub mod books;
pub mod game_data;
pub mod hindrances;
pub mod saves;
pub mod users;
pub mod utils;
pub mod announcement;
pub mod partners;
pub mod help_articles;
use actix_web::web::Data;


pub fn get_web_content(pool: &Data<Pool>) -> WebContent {
    let banners = Some(get_active_banners(pool));
    let help_articles = Some(get_active_help_articles(pool));
    let announcements = Some(get_active_announcements(pool));
    let partners = Some(get_active_partners(pool));
    WebContent {
        banners: banners,
        partners: partners,
        help_articles: help_articles,
        announcements: announcements,
        user: None,
    }
}