use actix_web::{web, HttpResponse, Responder};
use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    Database,
};
use mongodb::bson;
use serde_json::{json, Value};
use futures_util::stream::TryStreamExt;

use crate::lib::database::find_one;

pub async fn get_database(db: web::Data<Database>, path: web::Path<(String,)>) -> impl Responder {
    let _id = path.into_inner().0;
    let collection_name = "books";
    let collection = db.collection::<Document>(collection_name);

    match bson::oid::ObjectId::parse_str(&_id) {
        Ok(oid) => {
            let filter = doc! { "_id": oid };
            match collection.find_one(filter, None).await {
                Ok(Some(mut document)) => {
                    document.remove("_id");
                    // Convert BSON document to JSON
                    let json: Value = bson::from_bson(bson::Bson::Document(document))
                        .expect("Failed to convert BSON to JSON");

                    HttpResponse::Ok().json(json) // Sending back the JSON response
                }
                Ok(None) => HttpResponse::NotFound().body("Document not found"),
                Err(e) => HttpResponse::InternalServerError().body(format!("Error finding document: {}", e)),
            }
        },
        Err(_) => HttpResponse::BadRequest().body("Invalid ObjectId"),
    }
}