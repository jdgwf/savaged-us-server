use crate::db::books::get_books;
use actix_web::{get, web::Data, web::Json};
use mysql_async::Pool;
use savaged_libs::book::Book;

// #[get("/_api/books-get")]
// pub async fn books_get(
//     pool: Data<Pool>
// ) -> Json<Vec<Book>> {
//     let rows = get_books(
//         &pool,
//         0,
//         None,
//         false,
//         false,
//         false,
//         false,
//         false,

//     );

//     return actix_web::web::Json(
//         rows
//     );
// }
