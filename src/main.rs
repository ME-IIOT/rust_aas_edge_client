use actix_web::{web, App, HttpServer, middleware::Logger};
use std::sync::Mutex;
use mongodb::{Client, options::ClientOptions, Collection};
use std::{convert::TryFrom, error::Error};
use std::env;
use serde_json::Value;
use anyhow::Context;


async fn init_mongodb() -> (mongodb::Database, mongodb::Collection<mongodb::bson::Document>, mongodb::Collection<mongodb::bson::Document>) {
    let database_url = std::env::var("MONGO_URI").expect("MONGO_URI must be set");

    // Parse the MongoDB connection string into client options, panicking on failure.
    let client_options = mongodb::options::ClientOptions::parse(&database_url).await
        .expect("Failed to parse MongoDB URI into client options");

    // Attempt to create a MongoDB client with the specified options, panicking on failure.
    let client = mongodb::Client::with_options(client_options)
        .expect("Failed to initialize MongoDB client with given options");

    // Perform a simple operation to ensure the MongoDB server is accessible.
    // This operation tries to list database names, forcing a connection to be established.
    // Panics if the MongoDB server is not accessible.
    client.list_database_names(None, None).await
        .expect("Failed to connect to MongoDB. Ensure MongoDB is running and accessible.");

    // Access the specific database.
    let db = client.database("aas_edge_database");

    // Access the specific collections.
    let shells_collection = db.collection::<mongodb::bson::Document>("shells");
    let submodels_collection = db.collection::<mongodb::bson::Document>("submodels");

    // Return the database and collections; if any of the above steps fail, the function will have already panicked.
    (db, shells_collection, submodels_collection)
}

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

    let (db, shells_collection, submodels_collection) = init_mongodb().await;
        
    // Fetch the environment variables
    let mongo_uri = env::var("MONGO_URI").expect("MONGO_URI must be set");
    let aas_id_short = env::var("AAS_IDSHORT").expect("AAS_IDSHORT must be set");
    let aas_identifier = env::var("AAS_IDENTIFIER").expect("AAS_IDENTIFIER must be set");
    let aasx_server = env::var("AASX_SERVER").expect("AASX_SERVER must be set");
    let device_name = env::var("DEVICE_NAME").expect("DEVICE_NAME must be set");

    // let db = client.database("bookshelf");
    // Initialize AppState with all necessary data
    let app_state = web::Data::new(state::AppState {
        health_check_response: std::sync::Mutex::new(String::from("I'm OK!")),
        mongo_uri,
        aas_identifier,
        aas_id_short,
        aasx_server,
        device_name,
    });

    let submodels_collection_arc = std::sync::Arc::new(tokio::sync::Mutex::new(submodels_collection));
    let shells_collection_arc = std::sync::Arc::new(tokio::sync::Mutex::new(shells_collection));
    
    if let Err(e) = lib::onboarding::edge_device_onboarding(
        &app_state.aasx_server,
        &app_state.aas_identifier,
        &app_state.aas_id_short,
        shells_collection_arc.clone(),
        submodels_collection_arc.clone(),
    ).await {
        eprintln!("Failed to onboard submodels: {}", e);
    }

    HttpServer::new(move || {
        App::new()
            // GUIDE: add logger middleware
            .wrap(Logger::default())
            // GUIDE: pass shared data to the app
            .app_data(app_state.clone())
            .app_data(web::Data::new(db.clone()))
            .app_data(web::Data::new(shells_collection_arc.clone()))
            .app_data(web::Data::new(submodels_collection_arc.clone()))
            // GUIDE: Configure the routes
            .configure(routes::config) 
    })

    .bind("127.0.0.1:8080")?
    .run()
    .await
}
