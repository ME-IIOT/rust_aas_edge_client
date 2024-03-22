// use crate::database::{aas_find_one, aas_update_one};

// use actix_web::{web, App, HttpServer};
// use mongodb::{self, bson::{doc, Document}, Collection};
use mongodb;
use reqwest;
// use serde_json::Value;
use serde_json;
// use std::collections::HashMap;
// use futures::stream::{FuturesUnordered, StreamExt};
// use std::sync::Arc;
use base64;
// use std::error::Error;
use std;
use anyhow::{Context, Result};

pub async fn fetch_single_submodel(
    submodel_uid: &str,
    submodels_collection: mongodb::Collection<mongodb::bson::Document>,
    aasx_server_url: &str,
    aas_id_short: &str,
    // submodels_dictionary: mongodb::bson::Document,
    aas_uid: &str,
) -> Result<()> {
    let submodel_url = format!(
        "{}/shells/{}/submodels/{}",
        aasx_server_url,
        base64::encode_config(aas_uid, base64::URL_SAFE),
        base64::encode_config(submodel_uid, base64::URL_SAFE)
    );

    let response = reqwest::get(&submodel_url)
        .await
        .context("Failed to send request to fetch submodel")?;

    if response.status().is_success() {
        let body: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse response body as JSON")?;
        let submodel_id_short = body["idShort"]
            .as_str()
            .context("Failed to extract idShort from response body")?;

        let bson_value = mongodb::bson::to_bson(&body)
            .context("Failed to convert JSON body to BSON")?;
        if let mongodb::bson::Bson::Document(document) = bson_value {
            submodels_collection
                .update_one(
                    mongodb::bson::doc! { "_id": format!("{}:{}", aas_id_short, submodel_id_short) },
                    mongodb::bson::doc! { "$set": document },
                    mongodb::options::UpdateOptions::builder().upsert(true).build(),
                )
                .await
                .context("Failed to update submodel in the database")?;
        } else {
            return Err(anyhow::anyhow!("Conversion to Document failed."));
        }
    } else {
        return Err(anyhow::anyhow!(
            "Failed to fetch URL {}. Status code: {:?}",
            submodel_url,
            response.status()
        ));
    }

    Ok(())
}
// async fn fetch_single_submodel(
//     submodel_uid: &str,
//     submodels_collection: Collection<Value>,
//     aasx_server_url: &str,
//     aas_id_short: &str,
//     submodels_dictionary: Document,
//     aas_uid: &str,
// ) {
//     let submodel_url = format!("{}/shells/{}/submodels/{}", aasx_server_url, encode_config(aas_uid, base64::URL_SAFE), encode_config(submodel_uid, base64::URL_SAFE));
//     let response = reqwest::get(&submodel_url).await;

//     match response {
//         Ok(resp) => {
//             if resp.status().is_success() {
//                 let body: Value = resp.json().await.unwrap();
//                 let submodel_id_short = body["idShort"].as_str().unwrap();
//                 submodels_collection.update_one(
//                     doc! { "_id": format!("{}:{}", aas_id_short, submodel_id_short) },
//                     doc! { "$set": body },
//                     mongodb::options::UpdateOptions::builder().upsert(true).build(),
//                 ).await.unwrap();

//                 let mut dictionary = submodels_dictionary.lock().unwrap();
//                 dictionary[submodel_id_short] = json!(submodel_uid);
//             } else {
//                 println!("Failed to fetch URL {}. Status code: {:?}", submodel_url, resp.status());
//             }
//         },
//         Err(e) => println!("Failed to send request: {}", e),
//     }
// }

// async fn onboarding_submodels(
//     aasx_server_url: String,
//     aas_id_short: String,
//     submodels_id: Vec<String>,
//     submodels_collection: Collection<Value>,
//     aas_uid: String,
// ) {
//     let submodels_dictionary = Arc::new(std::sync::Mutex::new(json!({})));

//     let futures: FuturesUnordered<_> = submodels_id.into_iter().map(|submodel_id| {
//         let submodels_collection = submodels_collection.clone();
//         let submodels_dictionary = Arc::clone(&submodels_dictionary);
//         fetch_single_submodel(&submodel_id, submodels_collection, &aasx_server_url, &aas_id_short, submodels_dictionary, &aas_uid)
//     }).collect();

//     futures.collect::<Vec<()>>().await;

//     let submodels_dictionary = Arc::try_unwrap(submodels_dictionary).unwrap().into_inner().unwrap();
//     submodels_collection.update_one(
//         doc! { "_id": format!("{}:submodels_dictionary", aas_id_short) },
//         doc! { "$set": {"submodels": submodels_dictionary} },
//         mongodb::options::UpdateOptions::builder().upsert(true).build(),
//     ).await.unwrap();
// }

// pub async fn edge_device_onboarding(
//     aasx_server: String,
//     aas_uid: String,
//     aas_id_short: String,
//     shells_collection: Collection<Value>,
//     submodels_collection: Collection<Value>,
// ) {
//     let url = format!("{}/shells/{}", aasx_server, encode_config(&aas_uid, base64::URL_SAFE));
//     let response = reqwest::get(&url).await;

//     match response {
//         Ok(resp) => {
//             if resp.status().is_success() {
//                 let insert_data: Value = resp.json().await.unwrap();
//                 shells_collection.update_one(
//                     doc! { "_id": &aas_id_short },
//                     doc! { "$set": insert_data },
//                     mongodb::options::UpdateOptions::builder().upsert(true).build(),
//                 ).await.unwrap();

//                 let submodels_id: Vec<String> = extract_submodels_id(&insert_data); // Assume this function is implemented and returns Vec<String>
//                 onboarding_submodels(aasx_server, aas_id_short, submodels_id, submodels_collection, aas_uid).await;
//             } else {
//                 println!("Failed to fetch URL. Status code: {:?}", resp.status());
//             }
//         },
//         Err(e) => println!("Failed to send request: {}", e),
//     }
// }

// fn extract_submodels_id(data: &Value) -> Vec<String> {
//     let mut filtered_values: Vec<String> = Vec::new();

//     // Check if "submodels" key exists and is an array
//     if let Some(submodels) = data.get("submodels").and_then(|v| v.as_array()) {
//         // Iterate through the submodels
//         for submodel in submodels {
//             // Check if the submodel type is "ModelReference"
//             if submodel.get("type").and_then(|v| v.as_str()) == Some("ModelReference") {
//                 // Attempt to get the first item of "keys" array and its "value" key
//                 let value = submodel.get("keys")
//                     .and_then(|v| v.as_array())
//                     .and_then(|arr| arr.get(0))
//                     .and_then(|item| item.get("value"))
//                     .and_then(|v| v.as_str());

//                 if let Some(value_str) = value {
//                     filtered_values.push(value_str.to_string());
//                 }
//             }
//         }
//     }

//     filtered_values
// }