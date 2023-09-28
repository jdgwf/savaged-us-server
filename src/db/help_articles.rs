use actix_web::web::Data;
use mysql_async::prelude::*;
use mysql_async::*;
use savaged_libs::help_article::{HelpArticle, SimpleHelpArticle};

pub async fn get_active_help_articles(pool: &Data<Pool>) -> Vec<SimpleHelpArticle> {
    match pool.get_conn().await {
        Ok(mut conn) => {
            let get_help_articles_result = conn.query_map(
                "
                select
                id, data
                from help_articles
                where active > 0
                and deleted < 1
                -- and (start = 0 or start <= now())
                -- and (end >= now() or end = 0)
                ",
                |(id, data): (u32, String)| {
                    let mut help_article: SimpleHelpArticle = serde_json::from_str(data.as_ref()).unwrap();
                    // let help_article = HelpArticle::default();
                    help_article.id = id;
                    return help_article;
                },
            ).await;
            match get_help_articles_result {
                Ok(get_help_articles) => {
                    return get_help_articles;
                }

                Err(err) => {
                    println!("get_help_articles Error 4 {}", err);
                }
            }
        }
        Err(err) => {
            println!("get_help_articles Error 3 {}", err);
        }
    }
    return Vec::new();
}

pub async fn get_help_articles(pool: &Data<Pool>) -> Vec<HelpArticle> {
    match pool.get_conn().await {
        Ok(mut conn) => {
            let get_help_articles_result = conn.query_map(
                "SELECT
                    id,
                    data
                from help_articles where deleted < 1",
                |(id, data): (u32, String)| {
                    let mut help_article: HelpArticle = serde_json::from_str(data.as_ref()).unwrap();
                    // let help_article = HelpArticle::default();
                    help_article.id = id;
                    return help_article;
                },
            ).await;
            match get_help_articles_result {
                Ok(get_help_articles) => {
                    return get_help_articles;
                }

                Err(err) => {
                    println!("get_help_articles Error 4 {}", err);
                }
            }
        }
        Err(err) => {
            println!("get_help_articles Error 3 {}", err);
        }
    }
    return Vec::new();
}
