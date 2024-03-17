use actix_web::{web, App, HttpServer};
use std::sync::Mutex;
use mongodb::{Client, options::ClientOptions};
use std::{convert::TryFrom, error::Error};


mod handlers;
mod routes;
mod state;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let client = Client::with_uri_str("mongodb://localhost:27015").await.unwrap();
    let client_data = web::Data::new(client);

    let shared_data = web::Data::new(state::AppState {
        // Example shared state
        health_check_response: Mutex::new(String::from("I'm OK!")),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(shared_data.clone())
            .configure(routes::config) // Configure the routes
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
