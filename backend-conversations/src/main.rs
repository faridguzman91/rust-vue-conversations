mod models;
mod auth;
mod database;

use actix_web::{
    web, App, HttpServer, HttpResponse, HttpRequest,
    middleware::Logger, Result as ActixResult
};
use actix_web_httpauth::middleware::HttpAuthentication;
use actix_cors::Cors;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;
use sqlx::PgPool;
use std::env;
use std::io::Read;
use std::process::Command;
use std::fs::File;
use tempfile::NamedTempFile;
use uuid::Uuid;

use models::{TenantConfig, Claims, ConversationResponse};
use database::*;
use auth::jwt_validator;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("S3 error: {0}")]
    S3Error(String),
    #[error("Not found")]
    NotFound,
    #[error("Unauthorized")]
    Unauthorized,
}

impl actix_web::ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::Database(_) => HttpResponse::InternalServerError().json("Database error"),
            AppError::S3Error(_) => HttpResponse::InternalServerError().json("Storage error"),
            AppError::NotFound => HttpResponse::NotFound().json("Resource not found"),
            AppError::Unauthorized => HttpResponse::Unauthorized().json("Unauthorized"),
        }
    }
}

// Public endpoint - no authentication required
async fn get_config(
    req: HttpRequest,
    pool: web::Data<PgPool>,
) -> ActixResult<HttpResponse, AppError> {
    let hostname = req.connection_info().host().to_string();
    let tenant = get_tenant_by_hostname(&pool, &hostname).await?;

    let stun_servers: Vec<String> = tenant.stun_servers
        .split(',')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();

    let config = TenantConfig {
        display_name: tenant.display_name,
        stun_servers,
        keycloak_url: format!("{}/auth", tenant.keycloak_url),
        keycloak_realm: tenant.keycloak_realm,
        keycloak_client: tenant.keycloak_client,
        sentry_dsn: env::var("SENTRY_DSN").unwrap_or_default(),
        manager_url: tenant.manager_url,
        quality: tenant.quality,
    };

    Ok(HttpResponse::Ok().json(config))
}

// Protected endpoints
async fn get_conversations(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    query: web::Query<serde_json::Value>,
) -> ActixResult<HttpResponse, AppError> {
    let claims = req.extensions().get::<Claims>()
        .ok_or(AppError::Unauthorized)?;
    
    let hostname = req.connection_info().host().to_string();
    let tenant = get_tenant_by_hostname(&pool, &hostname).await?;
    
    // Check if user has access to this tenant
    let tenant_guid_str = tenant.guid.to_string();
    if !claims.conversations.contains_key(&tenant_guid_str) {
        return Err(AppError::Unauthorized);
    }
    
    let user_info = &claims.conversations[&tenant_guid_str];
    let limit = query.get("limit").and_then(|v| v.as_i64());
    let offset = query.get("offset").and_then(|v| v.as_i64());
    
    let conversations = get_conversations_for_tenant(
        &pool, 
        tenant.guid, 
        &user_info.user_guid,
        limit,
        offset
    ).await?;

    Ok(HttpResponse::Ok().json(conversations))
}

async fn get_conversation(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> ActixResult<HttpResponse, AppError> {
    let claims = req.extensions().get::<Claims>()
        .ok_or(AppError::Unauthorized)?;
    
    let hostname = req.connection_info().host().to_string();
    let tenant = get_tenant_by_hostname(&pool, &hostname).await?;
    
    let tenant_guid_str = tenant.guid.to_string();
    if !claims.conversations.contains_key(&tenant_guid_str) {
        return Err(AppError::Unauthorized);
    }
    
    let conversation_guid = path.into_inner();
    let conversation = get_conversation_by_guid(&pool, conversation_guid, tenant.guid).await?;
    
    match conversation {
        Some(conv) => Ok(HttpResponse::Ok().json(conv)),
        None => Err(AppError::NotFound),
    }
}

async fn get_groups(
    req: HttpRequest,
    pool: web::Data<PgPool>,
) -> ActixResult<HttpResponse, AppError> {
    let claims = req.extensions().get::<Claims>()
        .ok_or(AppError::Unauthorized)?;
    
    let hostname = req.connection_info().host().to_string();
    let tenant = get_tenant_by_hostname(&pool, &hostname).await?;
    
    let tenant_guid_str = tenant.guid.to_string();
    if !claims.conversations.contains_key(&tenant_guid_str) {
        return Err(AppError::Unauthorized);
    }
    
    let groups = get_groups_for_tenant(&pool, tenant.guid).await?;
    Ok(HttpResponse::Ok().json(groups))
}

async fn start_stream(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<serde_json::Value>,
) -> ActixResult<HttpResponse, AppError> {
    let claims = req.extensions().get::<Claims>()
        .ok_or(AppError::Unauthorized)?;
    
    let hostname = req.connection_info().host().to_string();
    let tenant = get_tenant_by_hostname(&pool, &hostname).await?;
    
    let tenant_guid_str = tenant.guid.to_string();
    if !claims.conversations.contains_key(&tenant_guid_str) {
        return Err(AppError::Unauthorized);
    }
    
    let conversation_guid = path.into_inner();
    let sdp_offer = body.into_inner();
    
    // This would integrate with the streaming microservice
    // For now, return a mock response
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "stream_id": conversation_guid,
        "sdp_answer": "mock_sdp_answer",
        "status": "started"
    })))
}

fn generate_waveform(audio_path: &str, json_path: &str) -> std::io::Result<()> {
    let status = Command::new("audiowaveform")
        .args(&["-i", audio_path, "-o", json_path, "-b", "8"])
        .status()?;

    if !status.success() {
        eprintln!("audiowaveform failed!")
    }
    Ok(())
}

// Keep existing S3/audio functionality
async fn get_audio(filename: web::Path<String>, s3: web::Data<Client>) -> HttpResponse {
    let result = s3
        .get_object()
        .bucket(&env::var("AWS_S3_BUCKET").unwrap_or("voicelogs".to_string()))
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

async fn get_waveform(filename: web::Path<String>, s3: web::Data<Client>) -> HttpResponse {
    let result = s3
        .get_object()
        .bucket(&env::var("AWS_S3_BUCKET").unwrap_or("voicelogs".to_string()))
        .key(&format!("{}.json", filename))
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

async fn upload_audio(audio: web::Bytes, s3: web::Data<Client>) -> HttpResponse {
    let mut temp_audio = NamedTempFile::new().unwrap();
    if let Err(e) = std::io::Write::write_all(&mut temp_audio, &audio) {
        eprintln!("Failed to write audio: {}", e);
        return HttpResponse::InternalServerError().body("Failed to save audio");
    }
    let audio_path = temp_audio.path().to_str().unwrap();

    let temp_json = NamedTempFile::new().unwrap();
    let json_path = temp_json.path().to_str().unwrap();
    if let Err(e) = generate_waveform(audio_path, json_path) {
        eprintln!("Waveform generation failed: {}", e);
        return HttpResponse::InternalServerError().body("Waveform generation failed");
    }

    let bucket = env::var("AWS_S3_BUCKET").unwrap_or("voicelogs".to_string());
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
    env_logger::init();

    // Database setup
    let pool = create_connection_pool().await
        .expect("Failed to create database connection pool");

    // AWS S3 setup
    let region_provider = RegionProviderChain::default_provider().or_else("eu-north-1");
    let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .region(region_provider)
        .load()
        .await;
    let s3_client = Client::new(&config);

    let bearer_middleware = HttpAuthentication::bearer(jwt_validator);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(s3_client.clone()))
            .wrap(cors)
            .wrap(Logger::default())
            // Public endpoints
            .route("/config", web::get().to(get_config))
            // Protected endpoints
            .service(
                web::scope("")
                    .wrap(bearer_middleware.clone())
                    .route("/conversations", web::get().to(get_conversations))
                    .route("/conversations/{guid}", web::get().to(get_conversation))
                    .route("/stream/{guid}/start", web::post().to(start_stream))
                    .route("/groups", web::get().to(get_groups))
                    .route("/upload", web::post().to(upload_audio))
                    .route("/audio/{filename}", web::get().to(get_audio))
                    .route("/waveform/{filename}", web::get().to(get_waveform))
            )
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
