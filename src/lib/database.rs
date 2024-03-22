use std::clone;

use actix_web::{web, HttpResponse, Responder};
use mongodb::{
    bson::{self, doc, Bson, Document},
    Collection, Database,
    options::UpdateOptions,
};
use serde_json::Value;
use serde::{Serialize, Deserialize};

// Adjusted to return a Result<Value, String> to better handle success and error states.
pub async fn aas_find_one(_id: String, collection: Collection<Document>) -> Result<Value, String> {
    let filter = doc! { "_id": &_id };

    match collection.find_one(filter, None).await {
        Ok(Some(document)) => {
            // Optionally remove the _id field from the document if not needed
            let mut document = document.clone(); // Clone if you need to modify the document.
            document.remove("_id");

            match bson::to_bson(&document) {
                Ok(bson::Bson::Document(doc)) => match bson::from_bson::<Value>(bson::Bson::Document(doc)) {
                    Ok(json) => Ok(json),
                    Err(_) => Err("Failed to convert BSON to JSON".into()),
                },
                _ => Err("Failed to handle BSON document".into()),
            }
        },
        Ok(None) => Err("Document not found".into()),
        Err(e) => Err(format!("Error finding document: {}", e)),
    }
}

pub async fn aas_update_one(_id: String, collection: Collection<Document>, new_document: Document, upsert: bool) -> Result<String, String> {
    let filter = doc! { "_id": _id };
    let options = UpdateOptions::builder().upsert(upsert).build();

    // GUIDE: Use the '$set' operator for the update, which requires modifying the document structure
    let update = doc! { "$set": new_document };

    // Perform the update operation
    match collection.update_one(filter, update, options).await {
        Ok(update_result) => {
            if let Some(upserted_id) = update_result.upserted_id {
                Ok(format!("Document upserted with id: {:?}", upserted_id))
            } else {
                Ok("Document updated successfully".into())
            }
        },
        Err(e) => Err(format!("Error upserting document: {}", e)),
    }
}