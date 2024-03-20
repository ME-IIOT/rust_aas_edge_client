use actix_web::{HttpResponse, Responder};
use mongodb::bson::{self, doc, Bson};
use serde_json::{json, Value};

pub async fn index() -> impl Responder {
    let bson_data = doc! {
        "message": "Homepage landing",
    };

    // Convert BSON to a serde_json Value
    let json_data: Value = match bson::to_bson(&bson_data) {
        Ok(bson) => serde_json::to_value(bson).unwrap(),
        Err(_) => json!({"error": "Failed to convert BSON to JSON"}),
    };

    HttpResponse::Ok()
        .content_type("application/json")
        .body(json_data.to_string()) // Convert Value to a String to set as the body
}
