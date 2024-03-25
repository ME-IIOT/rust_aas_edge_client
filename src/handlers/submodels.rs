use actix_web::{HttpResponse, Responder, web::{Path, Data}};
use mongodb::{bson::Document, Collection};
use std::sync::Arc;
use tokio::sync::Mutex;

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
    let aas_submodel = match aas_interfaces::get_submodel(submodels_collection, 
                                        &app_data.aas_id_short, 
                                        &submodel_id).await{
        Ok(aas_submodel) => aas_submodel,
        Err(e) => return actix_web::HttpResponse::InternalServerError().body(format!("Error getting submodel: {}", e)),
    };
    HttpResponse::Ok().json(aas_submodel)
}