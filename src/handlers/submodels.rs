use actix_web::{HttpResponse, Responder, web::{self, Path, Data}};
use mongodb::{bson::{doc, Document}, Collection};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::lib::aas_interfaces;
use crate::state::AppState;
pub async fn get_submodel(
    submodels_collection_arc: web::Data<Arc<Mutex<Collection<Document>>>>,
    path: web::Path<String>,
    app_data : web::Data<AppState>
) -> impl actix_web::Responder {
    let submodels_collection_lock = submodels_collection_arc.lock().await;
    let submodel_id = path.into_inner();

    let _id_submodel = format!("{}:{}", &app_data.aas_id_short, submodel_id);
    let aas_submodel_result = aas_interfaces::aas_find_one(_id_submodel, submodels_collection_lock.clone()).await;
    let aas_submodel =  match aas_submodel_result {
        Ok(aas_submodel) => aas_submodel,
        // Ok(submodel) => actix_web::HttpResponse::Ok().json(submodel),
        Err(e) => return actix_web::HttpResponse::InternalServerError().body(format!("Error getting submodel: {}", e)),
    };
    actix_web::HttpResponse::Ok().json(aas_submodel)
}