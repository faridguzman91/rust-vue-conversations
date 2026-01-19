use actix_web::{dev::ServiceRequest, Error, HttpMessage, error::ErrorUnauthorized, error::ErrorForbidden};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use reqwest;
use serde_json::Value;
use std::collections::HashMap;
use crate::models::Claims;

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("Invalid token: {0}")]
    InvalidToken(String),
    #[error("No access to tenant: {0}")]
    NoTenantAccess(String),
    #[error("Failed to fetch Keycloak public key: {0}")]
    KeycloakError(String),
    #[error("Request error: {0}")]
    RequestError(#[from] reqwest::Error),
}

pub async fn fetch_keycloak_public_key(keycloak_url: &str, realm: &str) -> Result<String, AuthError> {
    let url = format!("{}/auth/realms/{}/protocol/openid_connect/certs", keycloak_url, realm);
    
    let response = reqwest::get(&url).await?;
    
    if !response.status().is_success() {
        return Err(AuthError::KeycloakError(format!("HTTP {}", response.status())));
    }

    let jwks: Value = response.json().await?;
    
    // @faridguzman - extract the first key for simplicity - in production, you'd want to match by kid
    if let Some(keys) = jwks.get("keys").and_then(|k| k.as_array()) {
        if let Some(key) = keys.first() {
            if let Some(x5c) = key.get("x5c").and_then(|x| x.as_array()) {
                if let Some(cert) = x5c.first().and_then(|c| c.as_str()) {
                    let public_key = format!("-----BEGIN CERTIFICATE-----\n{}\n-----END CERTIFICATE-----", cert);
                    return Ok(public_key);
                }
            }
        }
    }
    
    Err(AuthError::KeycloakError("No valid key found in JWKS".to_string()))
}

pub fn get_tenant_guid_from_hostname(_hostname: &str) -> Result<String, AuthError> {
    // @faridguzman - in a real implementation, this would query the database to find tenant by hostname
    // for now, we'll return a dummy tenant guid
    Ok("0b816d6b898911eca00d02a7681a3548".to_string())
}

pub async fn jwt_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, Error> {
    let token = credentials.token();
    
    // @faridguzman - for development, we'll use a simple validation
    // in production, you'd fetch the public key from Keycloak
    let keycloak_url = std::env::var("KEYCLOAK_URL").unwrap_or_else(|_| "https://id.development.myCompany.dev".to_string());
    let realm = std::env::var("KEYCLOAK_REALM").unwrap_or_else(|_| "myCompanyapp".to_string());
    
    let public_key = match fetch_keycloak_public_key(&keycloak_url, &realm).await {
        Ok(key) => key,
        Err(e) => {
            eprintln!("Failed to fetch Keycloak public key: {}", e);
            // @faridguzman - for development, allow requests without proper key validation
            // remove this in production!
            let mut claims = HashMap::new();
            claims.insert("0b816d6b898911eca00d02a7681a3548".to_string(), crate::models::UserGuidInfo {
                user_guid: "test-user-guid".to_string(),
            });
            let dummy_claims = Claims {
                email: "test@example.com".to_string(),
                conversations: claims,
                sub: "test-user".to_string(),
                exp: 9999999999, // Far future
                iss: keycloak_url,
            };
            req.extensions_mut().insert(dummy_claims);
            return Ok(req);
        }
    };

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&[&keycloak_url]);
    
    let decoding_key = match DecodingKey::from_rsa_pem(public_key.as_bytes()) {
        Ok(key) => key,
        Err(e) => {
            return Err(ErrorUnauthorized(format!("Invalid key format: {}", e)));
        }
    };

    let token_data = decode::<Claims>(token, &decoding_key, &validation)
        .map_err(|e| ErrorUnauthorized(format!("Invalid token: {}", e)))?;

    // @faridguzman - verify tenant access
    let hostname = req.connection_info().host().to_string();
    let tenant_guid = get_tenant_guid_from_hostname(&hostname)
        .map_err(|e| ErrorForbidden(format!("Error resolving tenant: {}", e)))?;

    if !token_data.claims.conversations.contains_key(&tenant_guid) {
        return Err(ErrorForbidden("No access to this tenant"));
    }

    // @faridguzman - store claims in request extensions for later use
    req.extensions_mut().insert(token_data.claims);

    Ok(req)
}
