use actix_web::{web, App, HttpServer};
use std::sync::Mutex;
use mongodb::{Client, options::ClientOptions};
use std::{convert::TryFrom, error::Error};


// GUIDE: include the modules to main
mod handlers;
mod routes;
mod state;
mod models;
mod lib;

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    // GUIDE: mongodb connection
    let client = Client::with_uri_str("mongodb://localhost:27015").await.unwrap();
    // let client_data = web::Data::new(client.clone());
    let db = client.database("bookshelf");

    let shared_data = web::Data::new(state::AppState {
        // Example shared state
        health_check_response: Mutex::new(String::from("I'm OK!")),
    });

    HttpServer::new(move || {
        App::new()
            // GUIDE: pass shared data to the app
            .app_data(shared_data.clone())
            .app_data(web::Data::new(db.clone()))
            // GUIDE: Configure the routes
            .configure(routes::config) 
    })

    .bind("127.0.0.1:8080")?
    .run()
    .await
}
