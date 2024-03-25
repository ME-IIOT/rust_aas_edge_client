// use std::clone;
use actix_web::{web, HttpResponse, Responder};
use mongodb::{
    bson::{self, doc, Bson, Document},
    Collection, Database,
    options::UpdateOptions,
};
// use serde_json::Value;
// use serde::{Serialize, Deserialize};

// Adjusted to return a Result<Value, String> to better handle success and error states.
pub async fn aas_find_one(
    _id: String, 
    collection: mongodb::Collection<mongodb::bson::Document>) 
    -> Result<mongodb::bson::Document, String> {
    println!("Finding document with id: {}", _id);
    let filter = doc! { "_id": &_id };

    match collection.find_one(filter, None).await {
        Ok(Some(document)) => {
            // Optionally remove the _id field from the document if not needed
            let mut document = document.clone(); // Clone if you need to modify the document.
            document.remove("_id");
            Ok(document)
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


pub fn aas_sm_2_client_json(submodel_elements: Vec<mongodb::bson::Document>) -> mongodb::bson::Document {
    let mut client_json = mongodb::bson::Document::new();

    for element in submodel_elements {
        if let Some(model_type) = element.get_str("modelType").ok() {
            match model_type {
                "MultiLanguageProperty" => {
                    if let Some(values) = element.get_array("value").ok() {
                        let mut language_json = mongodb::bson::Document::new();
                        for value in values {
                            if let Bson::Document(value_doc) = value {
                                if let (Ok(language), Ok(text)) = (value_doc.get_str("language"), value_doc.get_str("text")) {
                                    language_json.insert(language, text);
                                }
                            }
                        }
                        if let Ok(id_short) = element.get_str("idShort") {
                            client_json.insert(id_short, mongodb::bson::Bson::Document(language_json));
                        }
                    }
                },
                "Property" => {
                    if let (Ok(id_short), Some(value)) = (element.get_str("idShort"), element.get("value")) {
                        client_json.insert(id_short, value.clone());
                    }
                },
                "SubmodelElementCollection" => {
                    if let Ok(id_short) = element.get_str("idShort") {
                        if let Some(values) = element.get_array("value").ok() {
                            let nested_elements: Vec<mongodb::bson::Document> = values.iter().filter_map(|v| {
                                if let Bson::Document(doc) = v {
                                    Some(doc.clone())
                                } else {
                                    None
                                }
                            }).collect();
                            
                            client_json.insert(id_short, mongodb::bson::Bson::Document(aas_sm_2_client_json(nested_elements)));
                        }
                    }
                },
                _ => {}
            }
        }
    }

    client_json
}

pub async fn get_submodel(
    submodels_collection_arc: std::sync::Arc<tokio::sync::Mutex<mongodb::Collection<mongodb::bson::Document>>>,
    aas_id_short: &str,
    submodel_id_short: &str
) -> Result<mongodb::bson::Document, String> {
    let submodels_collection_lock = submodels_collection_arc.lock().await;
    
    let _id_submodel = format!("{}:{}", aas_id_short, submodel_id_short);

    let aas_submodel_result = aas_find_one(_id_submodel, submodels_collection_lock.clone()).await;
    let aas_submodel = match aas_submodel_result {
        Ok(aas_submodel) => aas_submodel,
        Err(e) => return Err(format!("Error getting submodel: {}", e)),
    };

    let submodel_elements = match aas_submodel.get_array("submodelElements"){
        Ok(submodel_elements) => submodel_elements,
        Err(e) => return Err(format!("Error getting submodel elements: {}", e)),
    };
    let submodel_template: Vec<Document> = submodel_elements.iter().filter_map(|bson_item| {
        if let Bson::Document(doc) = bson_item {
            Some(doc.clone())
        } else {
            None // Handle non-Document items appropriately
        }
    }).collect();
    
    let client_bson = aas_sm_2_client_json(submodel_template);
    Ok(client_bson)
// ) -> Result<> {
    // let submodels_collection_lock = submodels_collection_arc.lock();
    // let id_submodel = format!("{}.{}", aas_id_short, submodel_id_short);

    // let aas_submodel_result = aas_find_one(id_submodel, submodels_collection_lock.clone());
    // let aas_submodel = match aas_submodel_result {
    //     Ok(aas_submodel) => aas_submodel,
    //     Err(e) => return Err(format!("Error getting submodel: {}", e)),
    // };
    // // let client_bson = aas_sm_2_client_json(aas_submodel);
    // // Ok(client_bson)
}
