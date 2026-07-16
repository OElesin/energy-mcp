use aws_sdk_dynamodb::Client as DynamoClient;
use aws_sdk_dynamodb::types::AttributeValue;
use chrono::Utc;
use serde::Serialize;
use std::collections::HashMap;

const KEYS_TABLE: &str = "energy-mcp-keys";
const RATELIMIT_TABLE: &str = "energy-mcp-ratelimit";
const FREE_TIER_LIMIT: i64 = 50;
const PRO_TIER_LIMIT: i64 = 10_000;
const BUSINESS_TIER_LIMIT: i64 = 100_000;

#[derive(Debug, Clone, Serialize)]
pub struct AuthResult {
    pub authenticated: bool,
    pub tier: String,
    pub remaining: i64,
}

#[derive(Debug)]
pub enum AuthError {
    RateLimited { limit: i64, tier: String },
    InvalidKey,
    InternalError(String),
}

/// Validate request: check API key if present, otherwise rate-limit by IP
pub async fn validate_request(
    dynamo: &DynamoClient,
    api_key: Option<&str>,
    source_ip: &str,
) -> Result<AuthResult, AuthError> {
    match api_key {
        Some(key) => validate_api_key(dynamo, key).await,
        None => validate_free_tier(dynamo, source_ip).await,
    }
}

/// Validate an API key against DynamoDB
async fn validate_api_key(dynamo: &DynamoClient, key: &str) -> Result<AuthResult, AuthError> {
    let result = dynamo
        .get_item()
        .table_name(KEYS_TABLE)
        .key("api_key", AttributeValue::S(key.to_string()))
        .send()
        .await
        .map_err(|e| AuthError::InternalError(format!("DynamoDB error: {}", e)))?;

    let item = match result.item {
        Some(item) => item,
        None => return Err(AuthError::InvalidKey),
    };

    let tier = item
        .get("tier")
        .and_then(|v| v.as_s().ok())
        .unwrap_or(&"pro".to_string())
        .clone();

    let limit = match tier.as_str() {
        "pro" => PRO_TIER_LIMIT,
        "business" => BUSINESS_TIER_LIMIT,
        _ => PRO_TIER_LIMIT,
    };

    // Increment usage counter for the day
    let today = Utc::now().format("%Y-%m-%d").to_string();
    let usage_key = format!("key:{}", key);

    let remaining = increment_and_check(dynamo, &usage_key, &today, limit).await?;

    if remaining < 0 {
        return Err(AuthError::RateLimited { limit, tier });
    }

    Ok(AuthResult {
        authenticated: true,
        tier,
        remaining,
    })
}

/// Rate-limit free tier by IP address
async fn validate_free_tier(dynamo: &DynamoClient, ip: &str) -> Result<AuthResult, AuthError> {
    let today = Utc::now().format("%Y-%m-%d").to_string();
    let usage_key = format!("ip:{}", ip);

    let remaining = increment_and_check(dynamo, &usage_key, &today, FREE_TIER_LIMIT).await?;

    if remaining < 0 {
        return Err(AuthError::RateLimited {
            limit: FREE_TIER_LIMIT,
            tier: "free".into(),
        });
    }

    Ok(AuthResult {
        authenticated: false,
        tier: "free".into(),
        remaining,
    })
}

/// Increment the counter and return remaining requests
async fn increment_and_check(
    dynamo: &DynamoClient,
    usage_key: &str,
    date: &str,
    limit: i64,
) -> Result<i64, AuthError> {
    let composite_key = format!("{}:{}", usage_key, date);

    // Calculate TTL: end of today + 1 hour buffer
    let tomorrow = Utc::now().date_naive().succ_opt().unwrap();
    let ttl = tomorrow.and_hms_opt(1, 0, 0).unwrap().and_utc().timestamp();

    let result = dynamo
        .update_item()
        .table_name(RATELIMIT_TABLE)
        .key("ip_key", AttributeValue::S(composite_key))
        .update_expression("SET #count = if_not_exists(#count, :zero) + :inc, #ttl = :ttl")
        .expression_attribute_names("#count", "request_count")
        .expression_attribute_names("#ttl", "ttl")
        .expression_attribute_values(":inc", AttributeValue::N("1".into()))
        .expression_attribute_values(":zero", AttributeValue::N("0".into()))
        .expression_attribute_values(":ttl", AttributeValue::N(ttl.to_string()))
        .return_values(aws_sdk_dynamodb::types::ReturnValue::UpdatedNew)
        .send()
        .await
        .map_err(|e| AuthError::InternalError(format!("Rate limit check failed: {}", e)))?;

    let count = result
        .attributes()
        .and_then(|attrs| attrs.get("request_count"))
        .and_then(|v| v.as_n().ok())
        .and_then(|n| n.parse::<i64>().ok())
        .unwrap_or(1);

    Ok(limit - count)
}
