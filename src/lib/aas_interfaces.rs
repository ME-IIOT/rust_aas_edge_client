// use std::clone;

use anyhow::Context;
use mongodb::{
    bson::{ doc, Bson, Document},
    Collection,
    options::UpdateOptions,
};
use tokio::sync::Mutex;
use std::sync::Arc;
use serde_json::Value;
use reqwest;

// use serde::{Serialize, Deserialize};

// Adjusted to return a Result<Value, String> to better handle success and error states.
pub async fn aas_find_one(
    _id: String, 
    // collection: mongodb::Collection<mongodb::bson::Document>
    submodels_collection_arc: Arc<Mutex<Collection<Document>>>
) 
    -> Result<mongodb::bson::Document, String> {
    let submodels_collection_lock = submodels_collection_arc.lock().await;
    // println!("Finding document with id: {}", _id);
    let filter = doc! { "_id": &_id };

    match submodels_collection_lock.find_one(filter, None).await {
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

pub async fn aas_update_one(_id: String, 
    submodels_collection: Arc<Mutex<Collection<Document>>>, 
    new_document: Document, upsert: bool) 
    -> Result<String, String> 
{
    
    let filter = doc! { "_id": _id };
    let options = UpdateOptions::builder().upsert(upsert).build();

    let submodels_collection_lock = submodels_collection.lock().await;
    // GUIDE: Use the '$set' operator for the update, which requires modifying the document structure
    let update = doc! { "$set": new_document };

    // Perform the update operation
    match submodels_collection_lock.update_one(filter, update, options).await {
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

pub async fn get_submodel_database(
    submodels_collection_arc: std::sync::Arc<tokio::sync::Mutex<mongodb::Collection<mongodb::bson::Document>>>,
    aas_id_short: &str,
    submodel_id_short: &str
) -> Result<mongodb::bson::Document, String> {
    
    let _id_submodel = format!("{}:{}", aas_id_short, submodel_id_short);

    let aas_submodel_result = aas_find_one(_id_submodel, submodels_collection_arc.clone()).await;
    let aas_submodel = match aas_submodel_result {
        Ok(aas_submodel) => aas_submodel,
        Err(e) => return Err(format!("Error getting submodel: {}", e)),
    };

    Ok(aas_submodel)
}

pub async fn patch_submodel_database(
    submodels_collection_arc: std::sync::Arc<tokio::sync::Mutex<mongodb::Collection<mongodb::bson::Document>>>,
    aas_id_short: &str,
    submodel_id_short: &str,
    // json: &web::Json<Value> // Use reference since content of json is not changed
    json: &Value
) -> Result<String, String> {
    
    let second_submodels_collection_arc = submodels_collection_arc.clone();
    let _id_submodel = format!("{}:{}", aas_id_short, submodel_id_short);

    let aas_submodel_result = aas_find_one(_id_submodel.clone(), submodels_collection_arc.clone()).await;
    let aas_submodel = match aas_submodel_result {
        Ok(aas_submodel) => aas_submodel,
        Err(e) => return Err(format!("Error getting submodel: {}", e)),
    };

    let mut patch_document: mongodb::bson::Document = match mongodb::bson::to_document(&json) {
        Ok(document) => document,
        Err(e) => return Err(format!("Error parsing request body: {}", e)),
    };

    merge_documents(&aas_submodel, &mut patch_document);

    let update_result = aas_update_one(_id_submodel, second_submodels_collection_arc.clone(), patch_document, false).await;
    match update_result {
        Ok(message) => Ok(message),
        Err(e) => Err(format!("Error patching submodel: {}", e)),
    }

}

fn merge_documents(old_doc: &Document, new_doc: &mut Document) {
    let keys_to_remove: Vec<String> = new_doc.keys()
        .filter(|k| !old_doc.contains_key(*k))
        .cloned()
        .collect();

    for key in keys_to_remove {
        new_doc.remove(&key);
    }

    for (key, old_value) in old_doc {
        match new_doc.get(key) {
            Some(new_value) => {
                if let (Bson::Document(old_subdoc), Bson::Document(new_subdoc)) = (old_value, new_value) {
                    let mut new_subdoc = new_subdoc.clone();
                    merge_documents(old_subdoc, &mut new_subdoc);
                    new_doc.insert(key, Bson::Document(new_subdoc));
                }
            }
            None => {
                new_doc.insert(key, old_value.clone());
            },
        }
    }
}



pub async fn patch_submodel_server(
    submodels_collection_arc: std::sync::Arc<tokio::sync::Mutex<Collection<Document>>>,
    aas_id_short: &str,
    submodel_id_short: &str,
    aasx_server_url: &str,
    aas_uid: &str,
    // json: &web::Json<Value>, // Use reference since content of json is not changed
    json: &Value
) -> Result<String, String> {
    // let submodels_collection_lock = submodels_collection_arc.lock().await;
    let _id_submodel = format!("{}:{}", aas_id_short, submodel_id_short);

    let submodels_collection = submodels_collection_arc.clone();
    let aas_submodel = aas_find_one(_id_submodel, submodels_collection_arc.clone()).await
        .map_err(|e| format!("Error getting submodel: {}", e))?;

    let mut patch_document: Document = mongodb::bson::to_document(&json)
        .map_err(|e| format!("Error parsing request body: {}", e))?;

    merge_documents(&aas_submodel, &mut patch_document);

    // let submodels_collection = submodels_collection_arc.clone();
    let submodels_dictionary = aas_find_one(format!("{}:submodels_dictionary", aas_id_short), submodels_collection.clone()).await
        .map_err(|e| format!("Error getting submodels dictionary: {}", e))?;

    let submodel_uid = submodels_dictionary.get_str(submodel_id_short)
        .map_err(|_| "Submodel not found in dictionary".to_string())?;

    let client = reqwest::Client::new();
    let url = format!(
        "{}shells/{}/submodels/{}/$value",
        aasx_server_url,
        base64::encode_config(aas_uid, base64::URL_SAFE_NO_PAD),
        base64::encode_config(submodel_uid, base64::URL_SAFE_NO_PAD),
    );

    let response = client.patch(&url)
        .json(&patch_document)
        .send()
        .await
        .map_err(|e| format!("Error sending patch request: {}", e))?;

    match response.status() {
        reqwest::StatusCode::NO_CONTENT => Ok("Submodel patched successfully".into()),
        _ => Err(response.text().await.unwrap_or_else(|_| "Unknown error".into())),
    }
}

pub async fn fetch_single_submodel_from_server(
    aasx_server_url: &str,
    aas_id_short: &str,
    aas_uid: &str,
    submodel_id_short: &str,
    submodels_collection_arc: std::sync::Arc<tokio::sync::Mutex<Collection<Document>>>
) -> Result<(), String> {
    // let submodels_collection_lock = submodels_collection_arc.lock().await;
    let submodels_dictionary = aas_find_one(format!("{}:submodels_dictionary", aas_id_short), 
                                                        submodels_collection_arc.clone()).await;
    
    let submodel_uid = match submodels_dictionary {
        Ok(submodels_dictionary) => {
            match submodels_dictionary.get(submodel_id_short) {
                Some(Bson::String(submodel_uid_str)) => submodel_uid_str.to_owned(), // Convert Bson::String to Rust String
                _ => return Err("Submodel not found in dictionary".into()), // Handle both None and non-string cases
            }
        },
        Err(e) => return Err(format!("Error getting submodels dictionary: {}", e)),
    };



    let client = reqwest::Client::new();
    let submodel_value_url = format!(
        "{}/shells/{}/submodels/{}/$value",
        aasx_server_url,
        base64::encode_config(aas_uid, base64::URL_SAFE_NO_PAD),
        base64::encode_config(submodel_uid, base64::URL_SAFE_NO_PAD),
    );

    let response = client.get(&submodel_value_url)
        .send()
        .await
        .with_context(|| format!("Failed to send request to fetch submodel value from URL: {}", submodel_value_url))
        .map_err(|e| format!("Error sending GET request: {}", e))?;

    if response.status().is_success() {
        let body_value: Value = response.json().await
            .with_context(|| "Failed to parse JSON response")
            .map_err(|e| format!("Error parsing JSON response: {}", e))?;
        let bson_value = mongodb::bson::to_bson(&body_value)
            .with_context(|| "Failed to convert JSON to BSON")
            .map_err(|e| format!("Error converting JSON to BSON: {}", e))?;

        if let mongodb::bson::Bson::Document(document) = bson_value {
            let submodels_collection_lock = submodels_collection_arc.lock().await;
            submodels_collection_lock.replace_one(
                mongodb::bson::doc! { "_id": format!("{}:{}", aas_id_short, submodel_id_short) },
                document,
                mongodb::options::ReplaceOptions::builder().upsert(false).build(),
            ).await
            .with_context(|| "Failed to replace submodel in the database")
            .map_err(|e| format!("Error replacing submodel in the database: {}", e))?;

            println!("Successfully replaced submodel: {}", submodel_id_short)
        } else {
            return Err("Response body is not a BSON document".into())
        }


    } else {
        return Err(format!("Error fetching submodel: {:?}", response))
    }
    Ok(())
    
}

// No need 
// pub fn aas_sm_2_client_json(submodel_elements: Vec<mongodb::bson::Document>) -> mongodb::bson::Document {
//     let mut client_json = mongodb::bson::Document::new();

//     for element in submodel_elements {
//         if let Some(model_type) = element.get_str("modelType").ok() {
//             match model_type {
//                 "MultiLanguageProperty" => {
//                     if let Some(values) = element.get_array("value").ok() {
//                         let mut language_json = mongodb::bson::Document::new();
//                         for value in values {
//                             if let Bson::Document(value_doc) = value {
//                                 if let (Ok(language), Ok(text)) = (value_doc.get_str("language"), value_doc.get_str("text")) {
//                                     language_json.insert(language, text);
//                                 }
//                             }
//                         }
//                         if let Ok(id_short) = element.get_str("idShort") {
//                             client_json.insert(id_short, mongodb::bson::Bson::Document(language_json));
//                         }
//                     }
//                 },
//                 "Property" => {
//                     if let (Ok(id_short), Some(value)) = (element.get_str("idShort"), element.get("value")) {
//                         client_json.insert(id_short, value.clone());
//                     }
//                 },
//                 "SubmodelElementCollection" => {
//                     if let Ok(id_short) = element.get_str("idShort") {
//                         if let Some(values) = element.get_array("value").ok() {
//                             let nested_elements: Vec<mongodb::bson::Document> = values.iter().filter_map(|v| {
//                                 if let Bson::Document(doc) = v {
//                                     Some(doc.clone())
//                                 } else {
//                                     None
//                                 }
//                             }).collect();
                            
//                             client_json.insert(id_short, mongodb::bson::Bson::Document(aas_sm_2_client_json(nested_elements)));
//                         }
//                     }
//                 },
//                 _ => {}
//             }
//         }
//     }

//     client_json
// }

// // send  data to server
// pub async fn patch_submodel_server(
//     submodels_collection_arc: std::sync::Arc<tokio::sync::Mutex<mongodb::Collection<mongodb::bson::Document>>>,
//     aas_id_short: &str,
//     submodel_id_short: &str,
//     aasx_server_url: &str,
//     aas_uid: &str,
//     json: web::Json<Value>
// ) -> Result<String, String> {
// {
//     let submodels_collection_lock = submodels_collection_arc.lock().await;
    
//     let _id_submodel = format!("{}:{}", aas_id_short, submodel_id_short);

//     let aas_submodel_result = aas_find_one(_id_submodel.clone(), submodels_collection_lock.clone()).await;
//     let mut aas_submodel = match aas_submodel_result {
//         Ok(aas_submodel) => aas_submodel,
//         Err(e) => return Err(format!("Error getting submodel: {}", e)),
//     };

//     let mut patch_document: mongodb::bson::Document = match mongodb::bson::to_document(&json.0) {
//         Ok(document) => document,
//         Err(e) => return Err(format!("Error parsing request body: {}", e)),
//     };

//     merge_documents(&aas_submodel, &mut patch_document);
//     let submodels_dictionary = aas_find_one(format!("{}:submodels_dictionary", aas_id_short), submodels_collection_lock.clone()).await;
//     let submodel_uid = match submodels_dictionary {
//         Ok(submodels_dictionary) => {
//             match submodels_dictionary.get(&submodel_id_short) {
//                 Some(submodel_uid) => submodel_uid,
//                 None => return Err("Submodel not found in dictionary".into()),
//             }
//         },
//         Err(e) => return Err(format!("Error getting submodels dictionary: {}", e)),
//     };

//     let client = reqwest::Client::new();
//     let url = format!(
//         "{}shells/{}/submodels/{}/$value",
//         aasx_server_url,
//         base64::encode_config(aas_uid, base64::URL_SAFE_NO_PAD),
//         base64::encode_config(submodel_uid, base64::URL_SAFE_NO_PAD)
//     )
    
//     let response = client.patch(&url)
//         .json(&patch_document)
//         .send()
//         .await;

//     if response.status().is_success() {
//         Ok("Submodel patched successfully".into())
//     } else {
//         Err(format!("Error patching submodel: {:?}", response))
//     }
// }
// }