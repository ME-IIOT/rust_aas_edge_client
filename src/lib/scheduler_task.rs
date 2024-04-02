use clokwerk::{Scheduler, TimeUnits};
use actix_web::web;
use std::time::Duration;
use tokio::{self, sync::Mutex};
use serde_json::{Value, Error, json};
use std::sync::Arc;
use mongodb::{bson::Document, Collection};
use chrono::{DateTime, Utc, TimeZone};

use crate::state::AppState;
use crate::lib::{aas_interfaces,bash_command};

// use super::onboarding;
fn parse_date_time_from_string(date_time_str: &str) -> Result<DateTime<Utc>, chrono::ParseError> {
    let date_time = DateTime::parse_from_rfc3339(date_time_str)?;
    Ok(date_time.with_timezone(&Utc))
}

async fn server_pushing(app_data: web::Data<AppState>, 
    submodels_collection_arc: Arc<Mutex<Collection<Document>>>
) {
    let submodels_collection = submodels_collection_arc.clone();
    
    // println!("Task one is running.");
    // Run bash script
    let script_output = bash_command::run_bash_script("./scripts/aas_client/sysInfo.sh").await;
    let output = match script_output {
        Ok(output) =>  output,
        Err(e) =>{
            eprintln!("Failed to execute script: {}", e);
            return;
        }
    };
    let submodel_id_short = "SystemInformation";
    let json: Value = serde_json::from_str(&output).unwrap();
    match aas_interfaces::patch_submodel_database(
        submodels_collection.clone(), 
        &app_data.aas_id_short, 
        &submodel_id_short, 
        &json).await{
            Ok(_) => (),
            Err(e) => {
                eprintln!("Failed to patch submodel: {}", e);
                return;
            }
    };

    let table_id = format!("{}:{}", &app_data.aas_id_short, "ManagedDevice");
    
    let managed_device = match aas_interfaces::aas_find_one(table_id,         
        submodels_collection_arc.clone()
    ).await{
        Ok(managed_device) => managed_device,
        Err(e) => {
            eprintln!("Failed to find managed device: {}", e);
            return;
        }
    };
    

    let boarding_status = match managed_device.get("BoardingStatus"){
        Some(boarding_status) => boarding_status.as_str().unwrap(),
        None => {
            eprintln!("Failed to get boarding status");
            return;
        }
    };

    let last_update_str = match managed_device.get("LastUpdate"){
        Some(last_update_str) => last_update_str.as_str().unwrap(),
        None => {
            eprintln!("Failed to get last update");
            return;
        }
    };

    let last_update = parse_date_time_from_string(last_update_str).unwrap();
    let time_now = Utc::now();
    
    if boarding_status == "OFFBOARDING_REQUESTED"{
        let submodel_id_short = "ManagedDevice";
        let json = json!({
            "BoardingStatus": "OFFBOARDED",
            "LastUpdate": time_now.to_rfc3339()
        });
        match aas_interfaces::patch_submodel_database(
            submodels_collection.clone(), 
            &app_data.aas_id_short, 
            &submodel_id_short, 
            &json
        ).await{
            Ok(_) => (),
            Err(e) => {
                eprintln!("Failed to offboard: {}", e);
                return;
            }
        };

        match aas_interfaces::patch_submodel_server(
            submodels_collection.clone(), 
            &app_data.aas_id_short, 
            &submodel_id_short, 
            &app_data.aasx_server, 
            &app_data.aas_identifier, 
            &json
        ).await{
            Ok(_) => println!("Successful offboarding"),
            Err(e) => {
                eprintln!("Failed to offboard: {}", e);
                return;
            }
        };

        return;
    }
    else if (boarding_status == "OFFBOARDED") && ((time_now - last_update).num_seconds() < 120){
        return ;
    }
    else if (boarding_status == "OFFBOARDED") && ((time_now - last_update).num_seconds() >= 120){
        let managed_device_json = json!({
            "BoardingStatus": "ONBOARDED",
            "LastUpdate": time_now.to_rfc3339()
        });

        match aas_interfaces::patch_submodel_database(
            submodels_collection.clone(), 
            &app_data.aas_id_short, 
            &"ManagedDevice", 
            &managed_device_json
        ).await{
            Ok(_) => (),
            Err(e) => {
                eprintln!("Failed to onboard: {}", e);
                return;
            }
        };

        match aas_interfaces::patch_submodel_server(
            submodels_collection.clone(), 
            &app_data.aas_id_short, 
            &"ManagedDevice", 
            &app_data.aasx_server, 
            &app_data.aas_identifier, 
            &managed_device_json
        ).await{
            Ok(_) => println!("Successful onboarding"),
            Err(e) => {
                eprintln!("Failed to onboard: {}", e);
                return;
            }
        };

        match aas_interfaces::patch_submodel_server(
            submodels_collection.clone(), 
            &app_data.aas_id_short, 
            &submodel_id_short, 
            &app_data.aasx_server, 
            &app_data.aas_identifier, 
            &json
        ).await{
            Ok(_) => println!("Successful pushing system information to server"),
            Err(e) => {
                eprintln!("Failed to patch submodel server: {}", e);
                return;
            }
        };
    }
    else if boarding_status == "ONBOARDED" {
        match aas_interfaces::patch_submodel_server(
            submodels_collection.clone(), 
            &app_data.aas_id_short, 
            &submodel_id_short, 
            &app_data.aasx_server, 
            &app_data.aas_identifier, 
            &json
        ).await{
            Ok(_) => println!("Successful pushing system information to server"),
            Err(e) => {
                eprintln!("Failed to patch submodel server: {}", e);
                return;
            }
        };
    }
    
    // Task one logic here
}

async fn server_polling(app_data: web::Data<AppState>, 
    submodels_collection_arc: Arc<Mutex<Collection<Document>>>) {
    // Task two logic here
    let submodels_collection = submodels_collection_arc.clone();
    let submodel_id_short = "ManagedDevice";
    // let submodel_id_short = "SystemInformation";

    match aas_interfaces::fetch_single_submodel_from_server(
        &app_data.aasx_server, 
        &app_data.aas_id_short, 
        &app_data.aas_identifier, 
        &submodel_id_short, 
        submodels_collection,
    ).await{
        Ok(_) => {
            println!("Successful fetching submodel {} from server", submodel_id_short);
            return;
        },
        Err(e) => {
            eprintln!("Failed to fetch submodel {} from server: {}", submodel_id_short, e);
            return;
        }
    };
}

pub async fn submodels_scheduler(app_state: web::Data<AppState>, 
                submodels_collection_arc: Arc<Mutex<Collection<Document>>>
){
    // println!("{}", app_state.aas_id_short);
    let mut scheduler = Scheduler::with_tz(chrono::Utc);

    // Clone before moving into the closure
    let app_state_cloned = app_state.clone();
    let submodels_collection_arc_cloned = submodels_collection_arc.clone();
    
    // Use move to take ownership of the cloned references
    scheduler.every(5.seconds()).run(move || {
        let task = server_pushing(app_state.clone(), submodels_collection_arc.clone());
        tokio::spawn(task);
    });

    // Schedule task_two to run every 10 seconds
    scheduler.every(10.seconds()).run(move || {
        let task = server_polling(app_state_cloned.clone(), submodels_collection_arc_cloned.clone());
        tokio::spawn(task); // Spawn the task asynchronously
    });

    // Scheduler tick loop
    tokio::spawn(async move {
        loop {
            scheduler.run_pending();
            tokio::time::sleep(Duration::from_millis(100)).await; // Short sleep between checks
        }
    });
}
