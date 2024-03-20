use actix_web::{web, HttpResponse, Responder};
use mongodb::{
    bson::{doc, oid::ObjectId},
    Database,
};
use mongodb::bson;
use serde_json::json;
use futures_util::stream::TryStreamExt;

// mod lib;
// use lib::{update_one, find_one, OperationResult};
use rust_web_mongo::find_one;

pub async fn get_database(db: web::Data<Database>, path: web::Path<(String,)>) -> impl Responder {
    let _id = path.into_inner().0;

    // Define the collection name where you want to search the document.
    // This should match one of your MongoDB collection names.
    let collection_name = "books";

    // Call the find_one function with the database, collection name, and _id.
    // This example assumes find_one is an async function and thus we await its result.
    match find_one(db, collection_name, &_id).await {
        // Assuming find_one returns a Result<OperationResult, mongodb::error::Error>
        Ok(operation_result) => HttpResponse::Ok().json(operation_result),
        Err(_) => HttpResponse::InternalServerError().json("An error occurred while fetching the document."),
    }
}