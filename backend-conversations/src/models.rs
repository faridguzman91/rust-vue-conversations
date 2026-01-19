use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Tenant {
    pub id: i32,
    pub guid: Uuid,
    pub hostname: String,
    pub display_name: String,
    pub keycloak_url: String,
    pub keycloak_realm: String,
    pub keycloak_client: String,
    pub manager_url: String,
    pub quality: String,
    pub stun_servers: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Conversation {
    pub id: i32,
    pub guid: Uuid,
    pub tenant_guid: Uuid,
    pub destination: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Participant {
    pub id: i32,
    pub guid: Uuid,
    pub conversation_guid: Uuid,
    pub display_name: Option<String>,
    pub role: String, // "agent" or "customer"
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ParticipantDetail {
    pub id: i32,
    pub participant_guid: Uuid,
    pub key: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Fact {
    pub id: i32,
    pub guid: Uuid,
    pub conversation_guid: Uuid,
    pub fact_type: String, // "conversation_start", "switch", "hold", etc.
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct FactDetail {
    pub id: i32,
    pub fact_guid: Uuid,
    pub key: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Metadata {
    pub id: i32,
    pub conversation_guid: Uuid,
    pub key: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Group {
    pub id: i32,
    pub guid: Uuid,
    pub tenant_guid: Uuid,
    pub set: String, // "campaigns", "queues", etc.
    pub display_name: String,
    pub parent_guid: Option<Uuid>,
}

// JWT Claims structure based on README
#[derive(Debug, Deserialize)]
pub struct Claims {
    pub email: String,
    pub conversations: HashMap<String, UserGuidInfo>,
    pub sub: String,
    pub exp: usize,
    pub iss: String,
}

#[derive(Debug, Deserialize)]
pub struct UserGuidInfo {
    pub user_guid: String,
}

// Configuration response structure
#[derive(Serialize)]
pub struct TenantConfig {
    pub display_name: String,
    pub stun_servers: Vec<String>,
    pub keycloak_url: String,
    pub keycloak_realm: String,
    pub keycloak_client: String,
    pub sentry_dsn: String,
    pub manager_url: String,
    pub quality: String,
}

// Response DTOs for API endpoints
#[derive(Debug, Serialize)]
pub struct ConversationResponse {
    pub guid: Uuid,
    pub destination: String,
    pub tenant_guid: Uuid,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub participants: Vec<ParticipantResponse>,
    pub facts: Vec<FactResponse>,
    pub metadata: Vec<MetadataResponse>,
    pub group: Option<GroupResponse>,
}

#[derive(Debug, Serialize)]
pub struct ParticipantResponse {
    pub display_name: Option<String>,
    pub role: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub details: Vec<DetailResponse>,
}

#[derive(Debug, Serialize)]
pub struct FactResponse {
    #[serde(rename = "type")]
    pub fact_type: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub details: Vec<DetailResponse>,
}

#[derive(Debug, Serialize)]
pub struct DetailResponse {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Serialize)]
pub struct MetadataResponse {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Serialize)]
pub struct GroupResponse {
    pub set: String,
    pub display_name: String,
    pub child: Option<Box<GroupResponse>>,
}