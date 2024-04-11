use actix_web::{web::Data, HttpResponse, Error};

use tokio::fs::File;
use tokio::io::AsyncReadExt;

use crate::state::AppState;

pub async fn get_thumbnail(
    app_data : Data<AppState>
) -> Result<HttpResponse, Error>{
    // using get_ref() to get the reference to the inner data
    let aas_id_short = &app_data.aas_id_short;
    let image_path = format!("./static/asset_images/{}.png", aas_id_short);
    
    // Asynchronously read the file from the filesystem
    let mut file = match File::open(image_path).await {
        Ok(file) => file,
        Err(_) => return Ok(HttpResponse::NotFound().body("Image not found")),
    };

    let mut contents = Vec::new();
    match file.read_to_end(&mut contents).await {
        Ok(_) => {},
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    }

    // Respond with the image
    Ok(HttpResponse::Ok().content_type("image/png").body(contents))
}