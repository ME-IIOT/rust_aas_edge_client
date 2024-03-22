use actix_web::{web, App, HttpServer, middleware::Logger};
use std::sync::Mutex;
use mongodb::{Client, options::ClientOptions, Collection};
use std::{convert::TryFrom, error::Error};
use std::env;
use serde_json::Value;

// Load environment variables from aas_client.env file
async fn load_env() {
    if let Err(e) = dotenv::from_filename("aas_client.env") {
        eprintln!("Error loading .env file: {}", e);
    }
}


// GUIDE: include the modules to main
mod handlers;
mod routes;
mod state;
mod models;
mod lib;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables from .env file
    load_env().await;
    
    


    // GUIDE: set env var for logging
    std::env::set_var("RUST_LOG", "actix_web=info");
    std::env::set_var("RUST_BACKTRACE", "1");
    // GUIDE: add logger
    env_logger::init();

    // GUIDE: mongodb connection
    let database_url: String = env::var("MONGO_URI").expect("MONGO_URI must be set");
    let client = Client::with_uri_str(database_url).await.unwrap();
    // let client_data = web::Data::new(client.clone());
    let db = client.database("aas_edge_database"); // Assuming the database name is 'aas_edge_database'
    let shells_collection: mongodb::Collection<mongodb::bson::Document> = db.collection("shells");
    let submodels_collection: mongodb::Collection<mongodb::bson::Document> = db.collection("submodels");

    let mongo_uri = env::var("MONGO_URI").unwrap_or(String::from("MONGO_URI=mongodb://mongodb:27017/mydatabase"));
    let aas_id_short  = env::var("AAS_IDSHORT ").unwrap_or(String::from("Murrelektronik_V000_CTXQ0_0100001_AAS"));
    let aas_identifier  = env::var("AAS_IDENTIFIER ").unwrap_or(String::from("https://aas.murrelektronik.com/V000-CTXQ0-0100001/aas/1/0"));
    let aasx_server = env::var("AASX_SERVER").unwrap_or(String::from("https://ca-aasxserverv3-dev-001.yellowtree-6659c4fd.northeurope.azurecontainerapps.io/"));

    lib::onboarding::fetch_single_submodel(
        &"aHR0cHM6Ly9leGFtcGxlLmNvbS9pZHMvc20vOTM5MF80MTYwXzAxMzJfMDk0MA",
        submodels_collection,
        &aasx_server.to_string(),
        &aas_id_short.to_string(),
        // shells_collection,
        &aas_identifier.to_string(),
    ).await;

    // let db = client.database("bookshelf");
    let shared_data = web::Data::new(state::AppState {
        // Example shared state
        health_check_response: Mutex::new(String::from("I'm OK!")),
    });

    HttpServer::new(move || {
        App::new()
            // GUIDE: add logger middleware
            .wrap(Logger::default())
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
