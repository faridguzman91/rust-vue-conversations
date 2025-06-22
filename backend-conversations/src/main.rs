use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;
use aws_sdk_s3::config::Credentials;
use std::env;
use std::io::Read;
use std::process::Command;
use std::fs::File;
use tempfile::NamedTempFile;
use uuid::Uuid;
//dotenv().ok();

fn generate_waveform(audio_path: &str, json_path: &str) -> std::io::Result<()> {
    let status = Command::new("audiowaveform")
        .args(&["-i", audio_path, "-o", json_path, "-b", "8"])
        .status()?;

    if !status.success() {
        eprintln!("audiowaveform failed!")
    }
    Ok(())
}

async fn get_audio(filename: web::Path<String>, s3: web::Data<Client>) -> impl Responder {
    let result = s3
        .get_object()
        .bucket("voicelogs")
        .key(&*filename)
        .send()
        .await;

    match result {
        Ok(output) => {
            let body = output.body.collect().await.unwrap();
            let bytes = body.into_bytes();

            HttpResponse::Ok().content_type("audio/mpeg").body(bytes)
        }

        Err(_) => HttpResponse::NotFound().body("Audio file not found"),
    }
}

async fn get_waveform(filename: web::Path<String>, s3: web::Data<Client>) -> impl Responder {
    let result = s3
        .get_object()
        .bucket("voicelogs")
        .key(&format!("{}.json", filename)) // assuming json files are stored with the same name
        .send()
        .await;

    match result {
        Ok(output) => {
            let body = output.body.collect().await.unwrap();
            let bytes = body.into_bytes();
            HttpResponse::Ok()
                .content_type("application/json")
                .body(bytes)
        }
        Err(_) => HttpResponse::NotFound().body("Waveform data not found"),
    }
}

async fn upload_audio(audio: web::Bytes, s3: web::Data<Client>) -> impl Responder  {
    // save audio to a temp file
    let mut temp_audio = NamedTempFile::new().unwrap();
    if let Err(e) = std::io::Write::write_all(&mut temp_audio, &audio) {
        eprintln!("Failed to write audio: {}", e);
        return HttpResponse::InternalServerError().body("Failed to save audio");
    }
    let audio_path = temp_audio.path().to_str().unwrap();

    // generate waveform JSON
    let temp_json = NamedTempFile::new().unwrap();
    let json_path = temp_json.path().to_str().unwrap();
    if let Err(e) = generate_waveform(audio_path, json_path) {
        eprintln!("Waveform generation failed: {}", e);
        return HttpResponse::InternalServerError().body("Waveform generation failed");
    }

    let bucket = env::var("AWS_S3_BUCKET").expect("AWS_S3_BUCKET must be set");

    let uuid = Uuid::new_v4();
    let audio_key = format!("{}.wav", uuid);
    let waveform_key = format!("{}.json", uuid);

    let audio_bytes = ByteStream::from_path(audio_path).await.unwrap();
    let audio_result = s3
        .put_object()
        .bucket(&bucket)
        .key(&audio_key)
        .body(audio_bytes)
        .send()
        .await;

    if let Err(e) = audio_result {
        eprintln!("Error uploading audio to S3: {}", e);
        return HttpResponse::InternalServerError().body("Failed to upload audio");
    }

    // upload waveform JSON to S3
    let mut json_file = File::open(json_path).unwrap();
    let mut json_buf = Vec::new();
    if let Err(e) = json_file.read_to_end(&mut json_buf) {
        eprintln!("Failed to read waveform JSON: {}", e);
        return HttpResponse::InternalServerError().body("Failed to read waveform JSON");
    }
    let waveform_result = s3
        .put_object()
        .bucket(&bucket)
        .key(&waveform_key)
        .body(ByteStream::from(json_buf))
        .send()
        .await;

    if let Err(e) = waveform_result {
        eprintln!("Error uploading waveform to S3: {}", e);
        return HttpResponse::InternalServerError().body("Failed to upload waveform");
    }

    HttpResponse::Ok().json(serde_json::json!({
        "audio_url": format!("https://{}.s3.amazonaws.com/{}", bucket, audio_key),
        "waveform_url": format!("https://{}.s3.amazonaws.com/{}", bucket, waveform_key),
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    //aws config
    let access_key = env::var("AWS_ACCESS_KEY_ID").expect("AWS_ACCESS_KEY_ID is required");
    let secret_key = env::var("AWS_SECRET_ACCESS_KEY").expect("AWS_SECRET_ACCESS_KEY is required");
    let region_provider = RegionProviderChain::default_provider().or_else("eu-north-1");
    let _bucket = env::var("AWS_S3_BUCKET").expect("AWS_S3_BUCKET must be set");

    let _credentials = Credentials::new(access_key, secret_key, None, None, "custom");
    let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
    .region(region_provider)
    .load()
    .await;
    let s3_client = Client::new(&config);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(s3_client.clone()))
            .route("/upload", web::post().to(upload_audio))
            .route("/audio/{filename}", web::get().to(get_audio))
            .route("/waveform/{filename}", web::get().to(get_waveform))
//            .service(upload_audio)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
