use actix_web::{web::{self, Data, Json, Path}, HttpResponse, Responder};
use mongodb::{bson::Document, Collection};
use std::sync::Arc;
use tokio::sync::Mutex;
use serde_json::{json, Value};
use chrono;

use crate::lib::aas_interfaces;
use crate::state::AppState;
pub async fn get_submodel(
    submodels_collection_arc: Data<Arc<Mutex<Collection<Document>>>>,
    path: Path<String>,
    app_data : Data<AppState>
) -> impl Responder {
    let submodel_id = path.into_inner();
    
    // using get_ref() to get the reference to the inner data
    let submodels_collection = submodels_collection_arc.get_ref().clone();
    let aas_submodel = match aas_interfaces::get_submodel_database(submodels_collection, 
                                        &app_data.aas_id_short, 
                                        &submodel_id).await{
        Ok(aas_submodel) => aas_submodel,
        Err(e) => return actix_web::HttpResponse::InternalServerError().body(format!("Error getting submodel: {}", e)),
    };
    HttpResponse::Ok().json(aas_submodel)
}

pub async fn patch_submodel(
    submodels_collection_arc: Data<Arc<Mutex<Collection<Document>>>>,
    path: Path<String>,
    app_data : Data<AppState>,
    json: web::Json<Value>
) -> impl Responder {
    let submodel_id_short = path.into_inner();
    // Handle LastUpdate only for SystemInformation and NetworkConfiguration
    // To modify the `json` value, work with its inner `Value` directly
    let mut json = json.into_inner();

    // add more submodels to the list if needed (can move it to AppState if needed in future)
    let submodel_id_short_list = vec!["SystemInformation",
                                                 "NetworkConfiguration"];
    if submodel_id_short_list.contains(&submodel_id_short.as_str()){
        json["LastUpdate"] = json!(chrono::Utc::now().to_rfc3339());
    }
    
    // using get_ref() to get the reference to the inner data
    let submodels_collection = submodels_collection_arc.get_ref().clone();
    // let aas_submodel = match aas_interfaces::patch_submodel_database(submodels_collection, 
    //                                     &app_data.aas_id_short, 
    //                                     &submodel_id, 
    //                                     json).await{
    //     Ok(aas_submodel) => aas_submodel,
    //     Err(e) => return actix_web::HttpResponse::InternalServerError().body(format!("Error patching submodel: {}", e)),
    // };

    match aas_interfaces::patch_submodel_database(
        submodels_collection, 
        &app_data.aas_id_short, 
        &submodel_id_short, 
        &json).await{
            Ok(_) => (),
            Err(e) => return actix_web::HttpResponse::InternalServerError().body(format!("Error patching submodel in database: {}", e)),
    };

    match aas_interfaces::patch_submodel_server(
        submodels_collection_arc.get_ref().clone(), 
        &app_data.aas_id_short, 
        &submodel_id_short, 
        &app_data.aasx_server, 
        &app_data.aas_identifier, 
        &json
    ).await{
        Ok(_) => HttpResponse::Ok().body("Submodel patched successfully"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error patching submodel on server: {}", e)),
    };

    HttpResponse::Ok().body("Submodel patched successfully")
}