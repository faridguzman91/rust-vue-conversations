use sqlx::{PgPool, Row};
use uuid::Uuid;
use crate::models::*;
use std::collections::HashMap;

pub async fn create_connection_pool() -> Result<PgPool, sqlx::Error> {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    
    PgPool::connect(&database_url).await
}

pub async fn get_tenant_by_hostname(pool: &PgPool, hostname: &str) -> Result<Tenant, sqlx::Error> {
    sqlx::query_as::<_, Tenant>(
        r#"
        SELECT id, guid, hostname, display_name, keycloak_url, keycloak_realm,
               keycloak_client, manager_url, quality, stun_servers
        FROM tenants 
        WHERE hostname = $1
        "#,
    )
    .bind(hostname)
    .fetch_one(pool)
    .await
}

pub async fn get_conversations_for_tenant(
    pool: &PgPool, 
    tenant_guid: Uuid,
    user_guid: &str,
    limit: Option<i64>,
    offset: Option<i64>
) -> Result<Vec<ConversationResponse>, sqlx::Error> {
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);
    
    // Get basic conversations
    let conversations = sqlx::query_as::<_, Conversation>(
        r#"
        SELECT id, guid, tenant_guid, destination, started_at, completed_at
        FROM conversations 
        WHERE tenant_guid = $1
        ORDER BY started_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(tenant_guid)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let mut result = Vec::new();
    
    for conv in conversations {
        // Get participants
        let participants = get_participants_for_conversation(pool, conv.guid).await?;
        
        // Get facts
        let facts = get_facts_for_conversation(pool, conv.guid).await?;
        
        // Get metadata
        let metadata = get_metadata_for_conversation(pool, conv.guid).await?;
        
        // For now, group is optional
        let group = None;
        
        result.push(ConversationResponse {
            guid: conv.guid,
            destination: conv.destination,
            tenant_guid: conv.tenant_guid,
            started_at: conv.started_at,
            completed_at: conv.completed_at,
            participants,
            facts,
            metadata,
            group,
        });
    }
    
    Ok(result)
}

pub async fn get_conversation_by_guid(
    pool: &PgPool, 
    conversation_guid: Uuid,
    tenant_guid: Uuid
) -> Result<Option<ConversationResponse>, sqlx::Error> {
    let conversation = sqlx::query_as::<_, Conversation>(
        r#"
        SELECT id, guid, tenant_guid, destination, started_at, completed_at
        FROM conversations 
        WHERE guid = $1 AND tenant_guid = $2
        "#,
    )
    .bind(conversation_guid)
    .bind(tenant_guid)
    .fetch_optional(pool)
    .await?;

    if let Some(conv) = conversation {
        let participants = get_participants_for_conversation(pool, conv.guid).await?;
        let facts = get_facts_for_conversation(pool, conv.guid).await?;
        let metadata = get_metadata_for_conversation(pool, conv.guid).await?;
        let group = None;
        
        Ok(Some(ConversationResponse {
            guid: conv.guid,
            destination: conv.destination,
            tenant_guid: conv.tenant_guid,
            started_at: conv.started_at,
            completed_at: conv.completed_at,
            participants,
            facts,
            metadata,
            group,
        }))
    } else {
        Ok(None)
    }
}

async fn get_participants_for_conversation(
    pool: &PgPool, 
    conversation_guid: Uuid
) -> Result<Vec<ParticipantResponse>, sqlx::Error> {
    let participants = sqlx::query_as::<_, Participant>(
        r#"
        SELECT id, guid, conversation_guid, display_name, role, started_at, completed_at
        FROM participants 
        WHERE conversation_guid = $1
        "#,
    )
    .bind(conversation_guid)
    .fetch_all(pool)
    .await?;

    let mut result = Vec::new();
    
    for participant in participants {
        let details = sqlx::query_as::<_, ParticipantDetail>(
            "SELECT id, participant_guid, key, value FROM participant_details WHERE participant_guid = $1"
        )
        .bind(participant.guid)
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|d| DetailResponse { key: d.key, value: d.value })
        .collect();

        result.push(ParticipantResponse {
            display_name: participant.display_name,
            role: participant.role,
            started_at: participant.started_at,
            completed_at: participant.completed_at,
            details,
        });
    }
    
    Ok(result)
}

async fn get_facts_for_conversation(
    pool: &PgPool, 
    conversation_guid: Uuid
) -> Result<Vec<FactResponse>, sqlx::Error> {
    let facts = sqlx::query_as::<_, Fact>(
        r#"
        SELECT id, guid, conversation_guid, fact_type, started_at, completed_at
        FROM facts 
        WHERE conversation_guid = $1
        "#,
    )
    .bind(conversation_guid)
    .fetch_all(pool)
    .await?;

    let mut result = Vec::new();
    
    for fact in facts {
        let details = sqlx::query_as::<_, FactDetail>(
            "SELECT id, fact_guid, key, value FROM fact_details WHERE fact_guid = $1"
        )
        .bind(fact.guid)
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|d| DetailResponse { key: d.key, value: d.value })
        .collect();

        result.push(FactResponse {
            fact_type: fact.fact_type,
            started_at: fact.started_at,
            completed_at: fact.completed_at,
            details,
        });
    }
    
    Ok(result)
}

async fn get_metadata_for_conversation(
    pool: &PgPool, 
    conversation_guid: Uuid
) -> Result<Vec<MetadataResponse>, sqlx::Error> {
    let metadata = sqlx::query_as::<_, Metadata>(
        "SELECT id, conversation_guid, key, value FROM metadata WHERE conversation_guid = $1"
    )
    .bind(conversation_guid)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|m| MetadataResponse { key: m.key, value: m.value })
    .collect();

    Ok(metadata)
}

pub async fn get_groups_for_tenant(
    pool: &PgPool, 
    tenant_guid: Uuid
) -> Result<Vec<GroupResponse>, sqlx::Error> {
    let groups = sqlx::query_as::<_, Group>(
        r#"
        SELECT id, guid, tenant_guid, set, display_name, parent_guid
        FROM groups 
        WHERE tenant_guid = $1 AND parent_guid IS NULL
        ORDER BY display_name
        "#,
    )
    .bind(tenant_guid)
    .fetch_all(pool)
    .await?;

    let mut result = Vec::new();
    
    for group in groups {
        let child = get_child_groups(pool, group.guid).await?;
        
        result.push(GroupResponse {
            set: group.set,
            display_name: group.display_name,
            child,
        });
    }
    
    Ok(result)
}

async fn get_child_groups(
    pool: &PgPool, 
    parent_guid: Uuid
) -> Result<Option<Box<GroupResponse>>, sqlx::Error> {
    let child = sqlx::query_as::<_, Group>(
        "SELECT id, guid, tenant_guid, set, display_name, parent_guid FROM groups WHERE parent_guid = $1"
    )
    .bind(parent_guid)
    .fetch_optional(pool)
    .await?;

    if let Some(child_group) = child {
        let nested_child = get_child_groups(pool, child_group.guid).await?;
        
        Ok(Some(Box::new(GroupResponse {
            set: child_group.set,
            display_name: child_group.display_name,
            child: nested_child,
        })))
    } else {
        Ok(None)
    }
}