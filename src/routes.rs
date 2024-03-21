use actix_web::web;
use crate::handlers; // Correctly import the handlers module

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg
        .service(web::resource("/").route(web::get().to(handlers::home::index)))
        .service(
            web::resource("/book")
                .route(web::get().to(handlers::book::get_books))
                .route(web::post().to(handlers::book::add_book))
                // .route(web::get().to(handlers::book::get_book_by_id))
                // .route(web::put().to(handlers::book::update_book))
                // .route(web::delete().to(handlers::book::delete_book)),
        )
        .service(
            web::resource("/book/{id}")
                .route(web::get().to(handlers::book::get_book_by_id))
                .route(web::put().to(handlers::book::update_book))
                .route(web::delete().to(handlers::book::delete_book)),
        )
        .service(
            web::resource(
                "/database/{id}").route(web::get().to(handlers::database::get_database)));
}