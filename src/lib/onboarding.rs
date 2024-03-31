use mongodb;
use reqwest;
use serde_json;
use base64;
use std;
use anyhow;
use anyhow::Context;
use tokio;
use futures;

// TODO: Missing shell collection
async fn fetch_single_submodel(
    aasx_server_url: &str,
    aas_id_short: &str,
    aas_uid: &str,
    submodel_uid: &str,
    submodels_collection: std::sync::Arc<tokio::sync::Mutex<mongodb::Collection<mongodb::bson::Document>>>,
    submodels_dictionary: std::sync::Arc<tokio::sync::Mutex<mongodb::bson::Document>>,
) -> std::result::Result<(), actix_web::Error> {
    let submodel_url_id_short = format!(
        "{}shells/{}/submodels/{}",
        aasx_server_url,
        base64::encode_config(aas_uid, base64::URL_SAFE_NO_PAD),
        base64::encode_config(submodel_uid, base64::URL_SAFE_NO_PAD)
    );

    let submodel_url_value = format!(
        "{}shells/{}/submodels/{}/$value",
        aasx_server_url,
        base64::encode_config(aas_uid, base64::URL_SAFE_NO_PAD),
        base64::encode_config(submodel_uid, base64::URL_SAFE_NO_PAD)
    );

    let client = reqwest::Client::new();
    let response_id_short = client.get(&submodel_url_id_short)
        .send()
        .await
        .with_context(|| format!("Failed to send request to fetch submodel id short from URL: {}", submodel_url_id_short))
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    let response_value = client.get(&submodel_url_value)
        .send()
        .await
        .with_context(|| format!("Failed to send request to fetch submodel value from URL: {}", submodel_url_value))
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    if response_id_short.status().is_success() & response_value.status().is_success(){
        let body_id_short: serde_json::Value = response_id_short.json().await
            .with_context(|| "Failed to parse response body as JSON")
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

        let body_value: serde_json::Value = response_value.json().await
            .with_context(|| "Failed to parse response body as JSON")
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

        let submodel_id_short = body_id_short["idShort"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Failed to extract idShort from response body"))
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

        // bson value collect from $value
        let bson_value = mongodb::bson::to_bson(&body_value)
            .with_context(|| "Failed to convert JSON body to BSON")
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

        // if let mongodb::bson::Bson::Document(document) = bson_value {
        //     let collection_lock = submodels_collection.lock().await;
        //     collection_lock.update_one(
        //         mongodb::bson::doc! { "_id": format!("{}:{}", aas_id_short, submodel_id_short) },
        //         mongodb::bson::doc! { "$set": document },
        //         mongodb::options::UpdateOptions::builder().upsert(false).build(),
        //     ).await
        //     .with_context(|| "Failed to update submodel in the database")
        //     .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

        //     println!("Successfully fetched submodel: {}", submodel_id_short)
        // } else {
        //     return Err(actix_web::error::ErrorInternalServerError("Conversion to Document failed."));
        // }
        if let mongodb::bson::Bson::Document(document) = bson_value {
            let collection_lock = submodels_collection.lock().await;
            collection_lock.replace_one(
                mongodb::bson::doc! { "_id": format!("{}:{}", aas_id_short, submodel_id_short) },
                document,
                mongodb::options::ReplaceOptions::builder().upsert(false).build(),
            ).await
            .with_context(|| "Failed to replace submodel in the database")
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
        
            println!("Successfully replaced submodel: {}", submodel_id_short)
        } else {
            return Err(actix_web::error::ErrorInternalServerError("Conversion to Document failed."));
        }

        let mut dictionary_lock = submodels_dictionary.lock().await;
        dictionary_lock.insert(submodel_id_short.to_string(), mongodb::bson::Bson::String(submodel_uid.to_string()));
    }
    else if !response_id_short.status().is_success(){
        // Clone the status code before consuming the response with `.text().await`
        let status_code = response_id_short.status().clone();

        let response_body = match response_id_short.text().await {
            Ok(text) => text,
            Err(_) => String::new(), // In case of error, default to an empty string
        };
        
        println!("Failed to fetch URL {}. Status code: {}. Response body: {}", submodel_url_id_short, status_code, response_body);
        return Err(actix_web::error::ErrorInternalServerError(format!(
            "Failed to fetch URL {}. Status code: {}. Response body: {}",
            submodel_url_id_short,
            status_code,
            response_body
        )));
    } 
    else {
        let status_code = response_value.status().clone();

        let response_body = match response_value.text().await {
            Ok(text) => text,
            Err(_) => String::new(), // In case of error, default to an empty string
        };

        println!("Failed to fetch URL {}. Status code: {}. Response body: {}", submodel_url_value, status_code, response_body);
        return Err(actix_web::error::ErrorInternalServerError(format!(
            "Failed to fetch URL {}. Status code: {}. Response body: {}",
            submodel_url_value,
            status_code,
            response_body
        )));  
    }

    Ok(())
}


async fn fetch_all_submodels(
    aasx_server_url: &str,
    aas_id_short: &str,
    submodel_uids: Vec<String>,
    submodels_collection: std::sync::Arc<tokio::sync::Mutex<mongodb::Collection<mongodb::bson::Document>>>,
    aas_uid: &str,
) -> Result<(), actix_web::Error> {
    println!("Fetching submodels from {}", aas_id_short);
    let submodels_dictionary = std::sync::Arc::new(tokio::sync::Mutex::new(mongodb::bson::Document::new()));

    let fetch_tasks: Vec<_> = submodel_uids.into_iter().map(|submodel_uid| {
        let submodels_collection_cloned = std::sync::Arc::clone(&submodels_collection);
        let submodels_dictionary_cloned = std::sync::Arc::clone(&submodels_dictionary);
        let aasx_server_url_cloned = aasx_server_url.to_string();
        let aas_id_short_cloned = aas_id_short.to_string();
        let aas_uid_cloned = aas_uid.to_string();

        tokio::spawn(async move {
            match fetch_single_submodel(
                &aasx_server_url_cloned,
                &aas_id_short_cloned,
                &aas_uid_cloned,
                &submodel_uid,
                submodels_collection_cloned,
                submodels_dictionary_cloned,
            ).await {
                Ok(_) => {},
                Err(e) => {
                    // Handle error appropriately, ensuring it doesn't require passing non-Send types across threads
                }
            }
        })
    }).collect();

    let _results = futures::future::join_all(fetch_tasks).await;

    let dictionary_lock = submodels_dictionary.lock().await;
    let bson_dictionary = mongodb::bson::to_bson(&*dictionary_lock)
                                        .with_context(|| "Failed to convert submodels dictionary to BSON")
                                        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    // if let mongodb::bson::Bson::Document(document) = bson_dictionary {
    //     let collection_lock = submodels_collection.lock().await;
    //     collection_lock.update_one(
    //         mongodb::bson::doc! { "_id": format!("{}:submodels_dictionary", aas_id_short) },
    //         mongodb::bson::doc! { "$set": document  },
    //         mongodb::options::UpdateOptions::builder().upsert(true).build(),
    //     ).await
    //     .with_context(|| "Failed to update submodels dictionary in the database")
    //     .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    // }
    if let mongodb::bson::Bson::Document(document) = bson_dictionary {
        let collection_lock = submodels_collection.lock().await;
        collection_lock.replace_one(
            mongodb::bson::doc! { "_id": format!("{}:submodels_dictionary", aas_id_short) },
            document,
            mongodb::options::ReplaceOptions::builder().upsert(true).build(),
        ).await
        .with_context(|| "Failed to replace submodels dictionary in the database")
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    }
    Ok(())
}

pub async fn edge_device_onboarding(
    aasx_server: &str,
    aas_uid: &str,
    aas_id_short: &str,
    shells_collection: std::sync::Arc<tokio::sync::Mutex<mongodb::Collection<mongodb::bson::Document>>>,
    submodels_collection: std::sync::Arc<tokio::sync::Mutex<mongodb::Collection<mongodb::bson::Document>>>,
) -> Result<(), actix_web::Error> {
    let url: String = format!(
        "{}/shells/{}",
        aasx_server,
        base64::encode_config(aas_uid, base64::URL_SAFE_NO_PAD)
    );

    println!("Fetching URL: {}", url);

    let client: reqwest::Client = reqwest::Client::new();
    let response: reqwest::Response = client.get(&url).send().await.map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    if response.status().is_success() {
        let insert_data: mongodb::bson::Document = response.json().await.map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

        // Assuming the logic to extract submodels ID is defined somewhere
        let submodels_id: Vec<String> = extract_submodels_id(&insert_data)?;

        {
            let collection_lock: tokio::sync::MutexGuard<'_, mongodb::Collection<mongodb::bson::Document>> = shells_collection.lock().await;
            collection_lock.replace_one(
                mongodb::bson::doc! { "_id": aas_id_short },
                insert_data,
                mongodb::options::ReplaceOptions::builder().upsert(true).build(),
            ).await.map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
        }
        

        fetch_all_submodels(
            aasx_server,
            aas_id_short,
            submodels_id,
            submodels_collection,
            aas_uid,
        ).await?;
    } else {
        println!("Failed to fetch URL. Status code: {}", response.status());
        return Err(actix_web::error::ErrorInternalServerError(format!(
            "Failed to fetch URL. Status code: {}",
            response.status()
        )));
    }

    Ok(())
}

// fn extract_submodels_id(data: &serde_json::Value) -> Result<Vec<String>, actix_web::Error> {
//     let mut filtered_values = Vec::new();

//     // Access "submodels" field, defaulting to an empty array if not found
//     let submodels = data.get("submodels").and_then(|v| v.as_array()).unwrap_or(&vec![]);

//     for submodel in submodels {
//         // Check if the submodel's type is "ModelReference"
//         if submodel.get("type").and_then(|t| t.as_str()) == Some("ModelReference") {
//             // Extract the value from the first element in "keys", if any
//             let value = submodel.get("keys")
//                 .and_then(|k| k.as_array())
//                 .and_then(|arr| arr.get(0))
//                 .and_then(|k| k.get("value"))
//                 .and_then(|v| v.as_str());

//             if let Some(value_str) = value {
//                 filtered_values.push(value_str.to_string());
//             }
//         }
//     }
//     Ok(filtered_values)
// }

fn extract_submodels_id(data: &mongodb::bson::Document) -> Result<Vec<String>, actix_web::Error> {
    let mut filtered_values = Vec::new();

    // Try to access "submodels" field as an array. If not present or not an array, default to empty.
    let submodels = match data.get_array("submodels") {
        Ok(submodels) => submodels,
        Err(_) => return Ok(filtered_values), // If there's an issue, return the empty vector.
    };

    for submodel in submodels.iter() {
        // Cast each submodel as a Document to access its fields.
        if let mongodb::bson::Bson::Document(submodel_doc) = submodel {
            // Check if the submodel's type is "ModelReference"
            if submodel_doc.get_str("type").unwrap_or_default() == "ModelReference" {
                // Try to extract the value from the first element in "keys"
                if let Ok(keys) = submodel_doc.get_array("keys") {
                    if let Some(mongodb::bson::Bson::Document(first_key_doc)) = keys.first() {
                        if let Ok(value_str) = first_key_doc.get_str("value") {
                            filtered_values.push(value_str.to_string());
                        }
                    }
                }
            }
        }
    }

    Ok(filtered_values)
}

// fn extract_submodels_id(data: &mongodb::bson::Document) -> Result<Vec<String>, actix_web::Error> {
//     let mut filtered_values = Vec::new();

//     // Access "submodels" field, defaulting to an empty array if not found
//     let submodels = data.get_array("submodels").unwrap_or(&vec![]);

//     for submodel in submodels {
//         // Attempt to cast each submodel as a Document to access its fields
//         if let Some(submodel_doc) = submodel.as_document() {
//             // Check if the submodel's type is "ModelReference"
//             if submodel_doc.get_str("type").unwrap_or_default() == "ModelReference" {
//                 // Extract the value from the first element in "keys", if any
//                 if let Ok(keys) = submodel_doc.get_array("keys") {
//                     if let Some(first_key) = keys.first() {
//                         if let Some(mongodb::bson::Bson::Document(key_doc)) = first_key.as_document() {
//                             if let Ok(value_str) = key_doc.get_str("value") {
//                                 filtered_values.push(value_str.to_string());
//                             }
//                         }
//                     }
//                 }
//             }
//         }
//     }

//     Ok(filtered_values)
// }

// fn convert_json_to_document(value: &serde_json::Value) -> Result<mongodb::bson::Document, actix_web::Error> {
//     // First, convert serde_json::Value to mongodb::bson::Bson
//     let bson_value = mongodb::bson::to_bson(value)
//         .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Error converting JSON to BSON: {}", e)))?;

//     // Then, try to convert mongodb::bson::Bson to mongodb::bson::Document
//     let document = mongodb::bson::from_bson::<mongodb::bson::Document>(bson_value)
//         .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Error converting BSON to Document: {}", e)))?;

//     Ok(document)
// }