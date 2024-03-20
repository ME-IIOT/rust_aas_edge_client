use actix_web::{web, HttpResponse, Responder};
use mongodb::{
    bson::{self, doc, Bson, Document},
    Collection, Database,
    options::UpdateOptions,
};
use serde_json::Value;
use serde::{Serialize, Deserialize};




#[derive(Serialize, Deserialize)]
pub struct OperationResult {
    status: String,
    action: String,
    id: Option<String>, // For the ID of the found/updated/upserted document
    ack: Option<bool>, // Acknowledgment of operation success
    document: Option<Value>, // For storing the found document
}


pub async fn update_one(
// pub fn update_one(
    db: web::Data<Database>,
    json_data: web::Json<Value>,
    collection_name: &str,
    upsert: bool,
    _id: &str,
) -> impl Responder {
    let collection: Collection<Document> = db.collection(collection_name);

    // Attempt to convert json_data to a BSON Document
    match bson::to_bson(&*json_data) {
        Ok(Bson::Document(document)) => {
            let filter = doc! { "_id": _id };
            let update = doc! { "$set": document };
            let options = UpdateOptions::builder().upsert(upsert).build();

            match collection.update_one(filter, update, options).await {
                Ok(insert_result) => {
                    if let Some(upserted_id) = insert_result.upserted_id {
                        let id_str = upserted_id.as_object_id().map(|oid| oid.to_hex());
                        HttpResponse::Ok().json(OperationResult {
                            status: "Success".into(),
                            action: "upserted".into(),
                            id: id_str,
                            ack: Some(true),
                            document: None,
                        })
                    } else {
                        HttpResponse::Ok().json(OperationResult {
                            status: "Success".into(),
                            action: "updated".into(),
                            id: None,
                            ack: Some(true),
                            document: None,
                        })
                    }
                },
                Err(e) => HttpResponse::InternalServerError().json(OperationResult {
                    status: "Error".into(),
                    action: "database operation failed".into(),
                    id: None,
                    ack: Some(false),
                    document: None,
                })
            }
        },
        _ => HttpResponse::BadRequest().json(OperationResult {
            status: "Error".into(),
            action: "invalid JSON structure".into(),
            id: None,
            ack: Some(false),
            document: None,
        })
    }
}

pub async fn find_one(
// pub fn find_one(
    db: web::Data<Database>,
    collection_name: &str,
    _id: &str,
) -> impl Responder {
    let collection: Collection<Document> = db.collection(collection_name);

    match bson::oid::ObjectId::parse_str(_id) {
        Ok(oid) => {
            let filter = doc! { "_id": oid };
            match collection.find_one(filter, None).await {
                Ok(Some(document)) => {
                    let document_json: Value = bson::to_bson(&document).unwrap().into();
                    HttpResponse::Ok().json(OperationResult {
                        status: "Success".into(),
                        action: "found".into(),
                        id: Some(_id.into()),
                        ack: Some(true),
                        document: Some(document_json),
                    })
                },
                Ok(None) => HttpResponse::Ok().json(OperationResult {
                    status: "Not Found".into(),
                    action: "find".into(),
                    id: None,
                    ack: Some(false),
                    document: None,
                }),
                Err(e) => HttpResponse::InternalServerError().json(OperationResult {
                    status: "Error".into(),
                    action: "database operation failed".into(),
                    id: None,
                    ack: Some(false),
                    document: None,
                }),
            }
        },
        Err(_) => HttpResponse::BadRequest().json(OperationResult {
            status: "Error".into(),
            action: "invalid ID format".into(),
            id: None,
            ack: Some(false),
            document: None,
        }),
    }
}
