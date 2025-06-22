use actix_web::{post, web, App, HttpResponse, HttpServer};
use aws_sdk_s3::Client;

fn generate_waveform(audio_path: &str, json_path: &str) -> std::io::Result<()> {
    let status = Command::new("audiowaveform")
        .args(&["-i", audio_path, "-o", json_path, "-b", "8"])
        .status()?;

    if !status.success() {
        eprintln!("audiowaveform failed!")
    }
    Ok(())
}

#[post("/upload")]
async fn upload_audio(audio: web::Bytes, s3: web::Data<Client>) -> HttpResponse {
    s3.put_object()
        .bucket("voicelogs")
        .body(audio.into())
        .send()
        .await;

    // added patern matching for error handling
    match result {
        Ok(_) => HttpResponse::Ok().body("Uploaded"),
        Err(e) => {
            eprintln!("Error uploading to S3: {}", e);
            HttpResponse::InternalServerError().body("Failed to upload")
        }
    }
    HttpResponse::Ok().body("Uploaded")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //example config
    let config = aws_sdk_s3::Config::builder()
        .region("us-east-1") // replace
        .credentials_provider(aws_sdk_s3::credentials::EnvironmentProvider::default())
        .build();

    let s3_client = Client::from_conf(config);
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(s3_client.clone()))
            .service(upload_audio)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
