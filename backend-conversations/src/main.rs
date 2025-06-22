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
        .await
        .unwrap();

    HttpResponse::Ok().body("Uploaded")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let s3_client = aws_sdk_s3::Client::new( /* some config */);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(s3_client.clone()))
            .service(upload_audio)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
