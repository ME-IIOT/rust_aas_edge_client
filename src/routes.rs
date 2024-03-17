use actix_web::web;
use crate::handlers; // Correctly import the handlers module

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/")
            .route(web::get().to(handlers::home::index)), // Now correctly references home::index
    );
}
