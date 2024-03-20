use actix_web::{web, HttpResponse, Responder};
use mongodb::{
    bson::{doc, oid::ObjectId},
    Database,
};
use mongodb::bson;
use serde_json::json;
use futures_util::stream::TryStreamExt;

use crate::models::book::Book;

// Handler for GET /book
pub async fn get_books(db: web::Data<Database>) -> HttpResponse {
    let collection = db.collection::<Book>("books");

    let books = collection.find(doc! {}, None)
        .await
        .unwrap()
        .try_collect::<Vec<_>>()
        .await
        .unwrap();

    HttpResponse::Ok().json(books)
}

// Handler for GET /book/{id}
pub async fn get_book_by_id(db: web::Data<Database>, path: web::Path<(String,)>) -> impl Responder {
    let collection = db.collection::<Book>("books");
    let id = ObjectId::parse_str(&path.0).expect("Failed to parse ObjectId");
    
    match collection.find_one(doc! { "_id": id }, None).await {
        Ok(Some(book)) => HttpResponse::Ok().json(book),
        Ok(None) => HttpResponse::NotFound().body("Book not found"),
        Err(_) => HttpResponse::InternalServerError().body("Internal server error"),
    }
}

// Handler for POST /book
pub async fn add_book(db: web::Data<Database>, new_book: web::Json<Book>) -> HttpResponse {
    let collection = db.collection::<Book>("books");

    match collection.insert_one(new_book.into_inner(), None).await {
        Ok(insert_result) => HttpResponse::Created().json(insert_result.inserted_id),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

// Handler for UPDATE /book/{id}
pub async fn update_book(db: web::Data<Database>, path: web::Path<(String,)>, updated_book: web::Json<Book>) -> impl Responder {
    let collection = db.collection::<Book>("books");
    let id = ObjectId::parse_str(&path.0).expect("Failed to parse ObjectId");
    let book_doc = bson::to_bson(&updated_book.into_inner()).expect("Failed to serialize book").as_document().unwrap().clone();

    match collection.update_one(doc! { "_id": id }, doc! { "$set": book_doc }, None).await {
        Ok(update_result) => {
            if update_result.matched_count == 1 {
                HttpResponse::Ok().body("Book updated successfully")
            } else {
                HttpResponse::NotFound().body("Book not found")
            }
        },
        Err(_) => HttpResponse::InternalServerError().body("Internal server error"),
    }
}

// Handler for DELETE /book/{id}
pub async fn delete_book(db: web::Data<Database>, path: web::Path<(String,)>) -> impl Responder {
    let collection = db.collection::<Book>("books");
    let id = ObjectId::parse_str(&path.0).expect("Failed to parse ObjectId");
    
    match collection.delete_one(doc! { "_id": id }, None).await {
        Ok(delete_result) => {
            if delete_result.deleted_count == 1 {
                HttpResponse::Ok().body("Book deleted successfully")
            } else {
                HttpResponse::NotFound().body("Book not found")
            }
        },
        Err(_) => HttpResponse::InternalServerError().body("Internal server error"),
    }
}

