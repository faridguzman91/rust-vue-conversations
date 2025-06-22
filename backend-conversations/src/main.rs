use actix_web::{post, web, App, HttpResponse, HttpServer};
use aws_sdk_s3::config::Config;
use aws_sdk_s3::credentials::Credentials;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::{Client, Region};
use dotenv::dotenv;
use std::env;
use std::path::Path;
use std::fs::File;
use std::io::Read;
use tempfile::NamedTempFile;
use tokio::fs;

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

#[get("/audio/{filename}")]
async fn get_audio(filename: web::Path<String>, s3: web::Data<Client>) -> HttpResponse {
    let result = s3
        .get_object()
        .bucket("voicelogs")
        .key(&filename)
        .send()
        .await;

    match result {
        Ok(output) => {
            let body = output.body.collect().await.unwrap();
            HttpResponse::Ok().content_type("audio/mpeg").body(body)
        }

        Err(_) => HttpResponse::NotFound().body("Audio file not found"),
    }
}

#[get("/waveform/{filename}")]
async fn get_waveform(filename: web::Path<String>, s3: web::Data<Client>) -> HttpResponse {
    let result = s3
        .get_object()
        .bucket("voicelogs")
        .key(&format!("{}.json", filename)) // assuming json files are stored with the same name
        .send()
        .await;

    match result {
        Ok(output) => {
            let body = output.body.collect().await.unwrap();
            HttpResponse::Ok().json(body)
        }
        Err(_) => HttpResponse::NotFound().body("Waveform data not found"),
    }
}

#[post("/upload")]
async fn upload_audio(audio: web::Bytes, s3: web::Data<Client>) -> HttpResponse {
    // @fardguzman - save audio to a temp file
    let mut temp_audio = NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut temp_audio, &audio).unwrap();
    let audio_path = temp_audio.path().to_str().unwrap();

    let temp_json = NamedTempFile::new().unwrap();
    let json_path = temp_json.path().to_str().unwrap();
    if let Err(e) = generate_waveform(audio_path, json_path) {
        eprintln!("Waveform generation failed: {}", e);
        return HttpResponse::InternalServerError().body("Waveform generation failed");
    }

    // @faridguzman - upload audio to S3
    let audio_bytes = ByteStream::from_path(audio_path).await.unwrap();
    let audio_key = "file.wav";
    let audio_result = s3.put_object()
        .bucket("voicelogs")
        .key(audio_key)
        .body(audio_bytes)
        .send()
        .await;

    if let Err(e) = audio_result {
        eprintln!("Error uploading audio to S3: {}", e);
        return HttpResponse::InternalServerError().body("Failed to upload audio");
    }

    // @faridguzman - upload waveform JSON to S3
    let mut json_file = File::open(json_path).unwrap();
    let mut json_buf = Vec::new();
    json_file.read_to_end(&mut json_buf).unwrap();
    let waveform_key = "file.json";
    let waveform_result = s3.put_object()
        .bucket("voicelogs")
        .key(waveform_key)
        .body(ByteStream::from(json_buf))
        .send()
        .await;

    if let Err(e) = waveform_result {
        eprintln!("Error uploading waveform to S3: {}", e);
        return HttpResponse::InternalServerError().body("Failed to upload waveform");
    }

    HttpResponse::Ok().json(serde_json::json!({
        "audio_url": format!("https://your-bucket-name.s3.amazonaws.com/{}", audio_key),
        "waveform_url": format!("https://your-bucket-name.s3.amazonaws.com/{}", waveform_key),
    }))
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //aws config
    let access_key = env::var("AWS_ACCESS_KEY_ID").expect("AWS_ACCESS_KEY_ID is required");
    let secret_key = env::var("AWS_SECRET_ACCESS_KEY").expect("AWS_SECRET_ACCESS_KEY is required");
    let region = Region::new("eu-north-1"); // the region of my s3

    let credentials = Credentials::new(access_key, secret_key, None, None, "custom");

    let config = Config::builder()
        .region(region) // replace with my region
        .credentials_provider(credentials)
        .build();

    let s3_client = Client::from_conf(config);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(s3_client.clone()))
            .route("/upload", web::post().to(upload_audio))
            .route("/audio/{filename}", web::get().to(get_audio))
            .route("/waveform/{filename}", web::get().to(get_waveform))
            .service(upload_audio)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
