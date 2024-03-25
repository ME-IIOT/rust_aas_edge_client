use actix_web::{
    web, HttpResponse, Responder};
use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    Database,
};
use mongodb::bson;
use serde_json::{json, Value};
use futures_util::stream::TryStreamExt;

use crate::lib::aas_interfaces::{aas_find_one, aas_update_one};

// GUIDE -> db: web::Data<Database> from the shared data
// pub async fn get_database(db: web::Data<Database>, path: web::Path<(String,)>) -> impl Responder {
//     // The path.into_inner().0 part extracts the first (and in this case, the only) parameter from the path
//     let _id = path.into_inner();
//     let collection_name = "books";
//     let collection = db.collection::<Document>(collection_name);

//     match bson::oid::ObjectId::parse_str(&_id) {
//         Ok(oid) => {
//             let filter = doc! { "_id": oid };
//             match collection.find_one(filter, None).await {
//                 Ok(Some(mut document)) => {
//                     document.remove("_id");
//                     // Convert BSON document to JSON
//                     let json: Value = bson::from_bson(bson::Bson::Document(document))
//                         .expect("Failed to convert BSON to JSON");

//                     HttpResponse::Ok().json(json) // Sending back the JSON response
//                 }
//                 Ok(None) => HttpResponse::NotFound().body("Document not found"),
//                 Err(e) => HttpResponse::InternalServerError().body(format!("Error finding document: {}", e)),
//             }
//         },
//         Err(_) => HttpResponse::BadRequest().body("Invalid ObjectId"),
//     }
// }


// GUIDE -> db: web::Data<Database> from the shared data
//#[get("/database/{id}")]
pub async fn get_database(db: web::Data<Database>, path: web::Path<String>) -> impl Responder {
    // GUIDE: For multiple: let (user_id, book_id) = path.into_inner(); // Destructure the tuple to get both parameters
    let _id = path.into_inner(); // Here, _id is directly used as a string
    println!("ID: {}", _id);
    let collection_name = "submodels";
    let collection = db.collection::<Document>(collection_name);

    // let filter = doc! { "_id": &_id }; // Use _id as a string directly in the filter
    // match collection.find_one(filter, None).await {
    //     Ok(Some(mut document)) => {
    //         document.remove("_id"); // Remove _id key from the document if needed
    //         // Convert the modified BSON document to a dynamic JSON Value
    //         match bson::to_bson(&document) {
    //             Ok(bson::Bson::Document(doc)) => match bson::from_bson::<Value>(bson::Bson::Document(doc)) {
    //                 Ok(json) => HttpResponse::Ok().json(json), // Sending back the JSON response
    //                 Err(_) => HttpResponse::InternalServerError().body("Failed to convert BSON to JSON"),
    //             },
    //             _ => HttpResponse::InternalServerError().body("Failed to handle BSON document"),
    //         }
    //     },
    //     Ok(None) => HttpResponse::NotFound().body("Document not found"),
    //     Err(e) => HttpResponse::InternalServerError().body(format!("Error finding document: {}", e)),
    // }

    // Directly use the result of aas_find_one to construct the HTTP response.
    match aas_find_one(_id, collection).await {
        Ok(json) => HttpResponse::Ok().json(json),
        Err(error_message) => match error_message.as_str() {
            "Document not found" => HttpResponse::NotFound().body(error_message),
            _ => HttpResponse::InternalServerError().body(error_message),
        },
    }
}

// For multiple: let (user_id, book_id) = path.into_inner(); // Destructure the tuple to get both parameters
//#[post("/database/{id}")]
pub async fn add_database(db: web::Data<Database>, json: web::Json<Value>, path: web::Path<String,>) -> impl Responder {
    let collection_name = "books";
    let collection = db.collection::<Document>(collection_name);
    let _id = path.into_inner();
    // match bson::to_bson(&json.0) {
    //     Ok(bson) => {
    //         let mut document = bson.as_document().unwrap().clone();
    //         let _id: String = path.into_inner();
    //         document.insert("_id", _id);

    //         match collection.insert_one(document, None).await {
    //             Ok(_) => HttpResponse::Created().body("Document added"),
    //             Err(e) => HttpResponse::InternalServerError().body(format!("Error adding document: {}", e)),
    //         }
    //     },
    //     Err(e) => HttpResponse::BadRequest().body(format!("Invalid JSON: {}", e)),
    // }
    // Convert serde_json::Value to mongodb::bson::Document
    let new_document = match bson::to_document(&json.0) {
        Ok(doc) => doc,
        Err(e) => return HttpResponse::BadRequest().body(format!("Error converting JSON to BSON: {}", e)),
    };

    match aas_update_one(_id, collection, new_document, false).await {
        Ok(_) => HttpResponse::Created().body("Document added"),
        Err(error_message) => HttpResponse::InternalServerError().body(format!("Error adding document: {}", error_message)),
        
    }
}

//#[put("/database/{id}")]
pub async fn update_database(db: web::Data<Database>, json: web::Json<Value>, path: web::Path<String,>) -> impl Responder {
    let collection_name = "books";
    let collection = db.collection::<Document>(collection_name);
    let _id = path.into_inner();
    // match bson::to_bson(&json.0) {
    //     Ok(bson) => {
    //         let mut document = bson.as_document().unwrap().clone();
    //         let _id: String = path.into_inner();
    //         document.insert("_id", _id);

    //         match collection.insert_one(document, None).await {
    //             Ok(_) => HttpResponse::Created().body("Document added"),
    //             Err(e) => HttpResponse::InternalServerError().body(format!("Error adding document: {}", e)),
    //         }
    //     },
    //     Err(e) => HttpResponse::BadRequest().body(format!("Invalid JSON: {}", e)),
    // }
    // Convert serde_json::Value to mongodb::bson::Document
    let new_document = match bson::to_document(&json.0) {
        Ok(doc) => doc,
        Err(e) => return HttpResponse::BadRequest().body(format!("Error converting JSON to BSON: {}", e)),
    };

    match aas_update_one(_id, collection, new_document, true).await {
        Ok(_) => HttpResponse::Created().body("Document updated"),
        Err(error_message) => HttpResponse::InternalServerError().body(format!("Error adding document: {}", error_message)),
        
    }
}