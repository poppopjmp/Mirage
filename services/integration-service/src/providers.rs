use crate::error::{IntegrationError, IntegrationResult};
use crate::models::{
    AuthType, Credential, ExecutionRecord, ExecutionStatus, Integration, ProviderInfo,
};
use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;

// Provider interface for implementing different integration providers
#[async_trait]
pub trait Provider: Send + Sync {
    // Get information about the provider
    fn get_info(&self) -> ProviderInfo;

    // Check if the provider supports the given authentication type
    fn supports_auth_type(&self, auth_type: &AuthType) -> bool;

    // Validate configuration against schema
    fn validate_config(&self, config: &Value) -> IntegrationResult<()>;

    // Execute an integration
    async fn execute(
        &self,
        integration: &Integration,
        credential: Option<&Credential>,
        parameters: Option<&Value>,
        target: Option<&str>,
        client: &Client,
        decrypt_fn: &dyn Fn(&str) -> IntegrationResult<String>,
    ) -> IntegrationResult<(Option<i32>, Option<String>)>;
}

// Provider registry to manage available providers
#[derive(Clone)]
pub struct ProviderRegistry {
    providers: Arc<HashMap<String, Arc<dyn Provider>>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        // Initialize with built-in providers
        let mut providers = HashMap::new();

        // Add HTTP API provider
        providers.insert(
            "http-api".to_string(),
            Arc::new(HttpApiProvider::new()) as Arc<dyn Provider>,
        );

        // Add social media provider
        providers.insert(
            "twitter".to_string(),
            Arc::new(TwitterProvider::new()) as Arc<dyn Provider>,
        );

        Self {
            providers: Arc::new(providers),
        }
    }

    // Get provider by ID
    pub fn get_provider(&self, id: &str) -> Option<Arc<dyn Provider>> {
        self.providers.get(id).cloned()
    }

    // List available providers
    pub fn list_providers(&self) -> Vec<ProviderInfo> {
        self.providers.values().map(|p| p.get_info()).collect()
    }
}

// HTTP API Provider Implementation
pub struct HttpApiProvider;

impl HttpApiProvider {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Provider for HttpApiProvider {
    fn get_info(&self) -> ProviderInfo {
        ProviderInfo {
            id: "http-api".to_string(),
            name: "HTTP API Provider".to_string(),
            description: "Generic HTTP API integration provider".to_string(),
            version: "1.0.0".to_string(),
            auth_types: vec![
                AuthType::None,
                AuthType::ApiKey,
                AuthType::Basic,
                AuthType::Bearer,
                AuthType::OAuth2,
            ],
            supported_targets: vec!["url".to_string(), "endpoint".to_string()],
            config_schema: serde_json::json!({
                "type": "object",
                "required": ["base_url", "method"],
                "properties": {
                    "base_url": {
                        "type": "string",
                        "format": "uri"
                    },
                    "method": {
                        "type": "string",
                        "enum": ["GET", "POST", "PUT", "DELETE", "PATCH"]
                    },
                    "headers": {
                        "type": "object",
                        "additionalProperties": {
                            "type": "string"
                        }
                    },
                    "path": {
                        "type": "string"
                    },
                    "query_params": {
                        "type": "object",
                        "additionalProperties": true
                    },
                    "body_template": {
                        "type": "object",
                        "additionalProperties": true
                    },
                    "response_mapping": {
                        "type": "object",
                        "additionalProperties": true
                    }
                }
            }),
            metadata: HashMap::new(),
        }
    }

    fn supports_auth_type(&self, auth_type: &AuthType) -> bool {
        matches!(
            auth_type,
            AuthType::None
                | AuthType::ApiKey
                | AuthType::Basic
                | AuthType::Bearer
                | AuthType::OAuth2
        )
    }

    fn validate_config(&self, config: &Value) -> IntegrationResult<()> {
        // Here we would validate the config against the schema
        // For simplicity, just checking required fields
        if let Some(obj) = config.as_object() {
            if !obj.contains_key("base_url") {
                return Err(IntegrationError::Validation(
                    "Missing required field: base_url".into(),
                ));
            }

            if !obj.contains_key("method") {
                return Err(IntegrationError::Validation(
                    "Missing required field: method".into(),
                ));
            }
        } else {
            return Err(IntegrationError::Validation(
                "Configuration must be an object".into(),
            ));
        }

        Ok(())
    }

    async fn execute(
        &self,
        integration: &Integration,
        credential: Option<&Credential>,
        parameters: Option<&Value>,
        target: Option<&str>,
        client: &Client,
        decrypt_fn: &dyn Fn(&str) -> IntegrationResult<String>,
    ) -> IntegrationResult<(Option<i32>, Option<String>)> {
        // Extract config
        let config = integration
            .config
            .as_object()
            .ok_or_else(|| IntegrationError::Validation("Invalid configuration format".into()))?;

        let base_url = config
            .get("base_url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| IntegrationError::Validation("Invalid base_url".into()))?;

        let method = config
            .get("method")
            .and_then(|v| v.as_str())
            .ok_or_else(|| IntegrationError::Validation("Invalid method".into()))?;

        // Build URL with optional path and target
        let mut url = base_url.to_string();

        if let Some(path) = config.get("path").and_then(|v| v.as_str()) {
            url.push_str(path);
        }

        // If target is provided, append it to the URL
        if let Some(t) = target {
            if !url.ends_with('/') {
                url.push('/');
            }
            url.push_str(t);
        }

        // Build request
        let mut req_builder = match method {
            "GET" => client.get(&url),
            "POST" => client.post(&url),
            "PUT" => client.put(&url),
            "DELETE" => client.delete(&url),
            "PATCH" => client.patch(&url),
            _ => {
                return Err(IntegrationError::Validation(format!(
                    "Unsupported method: {}",
                    method
                )))
            }
        };

        // Add headers
        if let Some(headers) = config.get("headers").and_then(|v| v.as_object()) {
            for (key, value) in headers {
                if let Some(val_str) = value.as_str() {
                    req_builder = req_builder.header(key, val_str);
                }
            }
        }

        // Add authentication
        if let Some(cred) = credential {
            let auth_data = decrypt_fn(&cred.encrypted_data)?;

            match cred.auth_type {
                AuthType::ApiKey => {
                    // Parse ApiKey auth data
                    let api_key_auth: Result<crate::models::ApiKeyAuth, _> =
                        serde_json::from_str(&auth_data);
                    if let Ok(api_key) = api_key_auth {
                        req_builder = req_builder.header(api_key.header_name, api_key.api_key);

                        // Add any additional headers
                        if let Some(headers) = api_key.additional_headers {
                            for (key, value) in headers {
                                req_builder = req_builder.header(key, value);
                            }
                        }
                    } else {
                        return Err(IntegrationError::Authentication(
                            "Invalid API key format".into(),
                        ));
                    }
                }
                AuthType::Basic => {
                    // Parse Basic auth data
                    let basic_auth: Result<crate::models::BasicAuth, _> =
                        serde_json::from_str(&auth_data);
                    if let Ok(basic) = basic_auth {
                        req_builder = req_builder.basic_auth(basic.username, Some(basic.password));
                    } else {
                        return Err(IntegrationError::Authentication(
                            "Invalid Basic auth format".into(),
                        ));
                    }
                }
                AuthType::Bearer => {
                    // Parse Bearer auth data
                    let bearer_auth: Result<crate::models::BearerAuth, _> =
                        serde_json::from_str(&auth_data);
                    if let Ok(bearer) = bearer_auth {
                        req_builder =
                            req_builder.header("Authorization", format!("Bearer {}", bearer.token));
                    } else {
                        return Err(IntegrationError::Authentication(
                            "Invalid Bearer auth format".into(),
                        ));
                    }
                }
                AuthType::OAuth2 => {
                    // Parse OAuth2 auth data
                    let oauth2_auth: Result<crate::models::OAuth2Auth, _> =
                        serde_json::from_str(&auth_data);
                    if let Ok(oauth2) = oauth2_auth {
                        let token_type = oauth2.token_type.unwrap_or_else(|| "Bearer".to_string());
                        req_builder = req_builder.header(
                            "Authorization",
                            format!("{} {}", token_type, oauth2.access_token),
                        );
                    } else {
                        return Err(IntegrationError::Authentication(
                            "Invalid OAuth2 auth format".into(),
                        ));
                    }
                }
                _ => {
                    return Err(IntegrationError::Authentication(format!(
                        "Unsupported auth type: {:?}",
                        cred.auth_type
                    )))
                }
            }
        }

        // Add query parameters
        if let Some(query_params) = config.get("query_params").and_then(|v| v.as_object()) {
            for (key, value) in query_params {
                req_builder = req_builder.query(&[(key, value)]);
            }
        }

        // Add body if POST, PUT, or PATCH
        if matches!(method, "POST" | "PUT" | "PATCH") {
            if let Some(body_template) = config.get("body_template") {
                // Replace placeholders with parameters
                let mut body = body_template.clone();
                if let Some(params) = parameters {
                    // Simplistic approach - in real implementation, use proper template engine
                    if let Some(body_obj) = body.as_object_mut() {
                        if let Some(params_obj) = params.as_object() {
                            for (k, v) in params_obj {
                                body_obj.insert(k.clone(), v.clone());
                            }
                        }
                    }
                }

                req_builder = req_builder.json(&body);
            }
        }

        // Execute the request
        let response = req_builder.send().await?;

        // Check for errors
        if !response.status().is_success() {
            return Err(IntegrationError::ExternalApi(format!(
                "API request failed with status: {}",
                response.status()
            )));
        }

        // Parse response based on mapping
        let response_body = response.text().await?;
        let response_data: Value = serde_json::from_str(&response_body).map_err(|e| {
            IntegrationError::ExternalApi(format!("Failed to parse response as JSON: {}", e))
        })?;

        // Count results if array
        let result_count = if response_data.is_array() {
            Some(response_data.as_array().unwrap().len() as i32)
        } else {
            None
        };

        // Return result count and raw response
        Ok((result_count, Some(response_body)))
    }
}

// Twitter API Provider
pub struct TwitterProvider;

impl TwitterProvider {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Provider for TwitterProvider {
    fn get_info(&self) -> ProviderInfo {
        ProviderInfo {
            id: "twitter".to_string(),
            name: "Twitter API Provider".to_string(),
            description: "Integration with Twitter API v2".to_string(),
            version: "1.0.0".to_string(),
            auth_types: vec![AuthType::OAuth1, AuthType::OAuth2, AuthType::Bearer],
            supported_targets: vec![
                "user".to_string(),
                "hashtag".to_string(),
                "tweet".to_string(),
                "followers".to_string(),
                "following".to_string(),
            ],
            config_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "api_version": {
                        "type": "string",
                        "enum": ["v1", "v2"],
                        "default": "v2"
                    },
                    "tweet_fields": {
                        "type": "array",
                        "items": {
                            "type": "string",
                            "enum": ["id", "text", "created_at", "author_id", "public_metrics"]
                        }
                    },
                    "max_results": {
                        "type": "integer",
                        "minimum": 5,
                        "maximum": 100,
                        "default": 10
                    }
                }
            }),
            metadata: HashMap::new(),
        }
    }

    fn supports_auth_type(&self, auth_type: &AuthType) -> bool {
        matches!(
            auth_type,
            AuthType::OAuth1 | AuthType::OAuth2 | AuthType::Bearer
        )
    }

    fn validate_config(&self, config: &Value) -> IntegrationResult<()> {
        // Basic validation
        if let Some(obj) = config.as_object() {
            if let Some(api_version) = obj.get("api_version").and_then(|v| v.as_str()) {
                if api_version != "v1" && api_version != "v2" {
                    return Err(IntegrationError::Validation(
                        "api_version must be either 'v1' or 'v2'".into(),
                    ));
                }
            }

            if let Some(max_results) = obj.get("max_results").and_then(|v| v.as_i64()) {
                if max_results < 5 || max_results > 100 {
                    return Err(IntegrationError::Validation(
                        "max_results must be between 5 and 100".into(),
                    ));
                }
            }
        }

        Ok(())
    }

    async fn execute(
        &self,
        integration: &Integration,
        credential: Option<&Credential>,
        parameters: Option<&Value>,
        target: Option<&str>,
        client: &Client,
        decrypt_fn: &dyn Fn(&str) -> IntegrationResult<String>,
    ) -> IntegrationResult<(Option<i32>, Option<String>)> {
        // We need credentials for Twitter API
        let cred = credential.ok_or_else(|| {
            IntegrationError::Authentication("Twitter API integration requires credentials".into())
        })?;

        // Get config
        let config = integration
            .config
            .as_object()
            .ok_or_else(|| IntegrationError::Validation("Invalid configuration format".into()))?;

        // Set defaults
        let api_version = config
            .get("api_version")
            .and_then(|v| v.as_str())
            .unwrap_or("v2");

        let max_results = config
            .get("max_results")
            .and_then(|v| v.as_i64())
            .unwrap_or(10);

        // Construct base URL based on API version
        let base_url = match api_version {
            "v1" => "https://api.twitter.com/1.1",
            "v2" => "https://api.twitter.com/2",
            _ => return Err(IntegrationError::Validation("Invalid API version".into())),
        };

        // Determine endpoint based on target type
        let endpoint = match target {
            Some("user") => "/users/by",
            Some("hashtag") => "/tweets/search/recent",
            Some("tweet") => "/tweets",
            Some("followers") => "/users/{id}/followers",
            Some("following") => "/users/{id}/following",
            _ => {
                return Err(IntegrationError::Validation(
                    "Invalid or missing target type".into(),
                ))
            }
        };

        // Construct URL
        let url = format!("{}{}", base_url, endpoint);

        // Get auth token
        let auth_data = decrypt_fn(&cred.encrypted_data)?;
        let auth_header = match cred.auth_type {
            AuthType::Bearer => {
                let bearer_auth: Result<crate::models::BearerAuth, _> =
                    serde_json::from_str(&auth_data);
                match bearer_auth {
                    Ok(bearer) => format!("Bearer {}", bearer.token),
                    Err(_) => {
                        return Err(IntegrationError::Authentication(
                            "Invalid Bearer auth format".into(),
                        ))
                    }
                }
            }
            AuthType::OAuth2 => {
                let oauth2_auth: Result<crate::models::OAuth2Auth, _> =
                    serde_json::from_str(&auth_data);
                match oauth2_auth {
                    Ok(oauth2) => {
                        let token_type = oauth2.token_type.unwrap_or_else(|| "Bearer".to_string());
                        format!("{} {}", token_type, oauth2.access_token)
                    }
                    Err(_) => {
                        return Err(IntegrationError::Authentication(
                            "Invalid OAuth2 auth format".into(),
                        ))
                    }
                }
            }
            _ => {
                return Err(IntegrationError::Authentication(
                    "Unsupported auth type for Twitter provider".into(),
                ))
            }
        };

        // Create request
        let mut req_builder = client.get(&url);

        // Add auth header
        req_builder = req_builder.header("Authorization", auth_header);

        // Add query parameters
        let mut query_params = Vec::new();

        // Add max_results parameter
        query_params.push(("max_results", max_results.to_string()));

        // Add tweet fields if specified
        if let Some(tweet_fields) = config.get("tweet_fields").and_then(|v| v.as_array()) {
            let fields: Vec<String> = tweet_fields
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect();

            if !fields.is_empty() {
                query_params.push(("tweet.fields", fields.join(",")));
            }
        }

        // Add target-specific parameters
        match target {
            Some("user") => {
                // Username parameter
                if let Some(params) = parameters {
                    if let Some(username) = params.get("username").and_then(|v| v.as_str()) {
                        query_params.push(("usernames", username.to_string()));
                    }
                }
            }
            Some("hashtag") => {
                // Query parameter for hashtag
                if let Some(hashtag) =
                    parameters.and_then(|p| p.get("hashtag").and_then(|v| v.as_str()))
                {
                    let query = format!("#{}", hashtag.trim_start_matches('#'));
                    query_params.push(("query", query));
                }
            }
            Some("tweet") => {
                // Tweet ID parameter
                if let Some(tweet_id) =
                    parameters.and_then(|p| p.get("id").and_then(|v| v.as_str()))
                {
                    req_builder = client.get(&format!("{}/{}", url, tweet_id));
                }
            }
            Some(target_type) => {
                // Replace {id} in URL with actual ID for followers/following endpoints
                if target_type == "followers" || target_type == "following" {
                    if let Some(user_id) =
                        parameters.and_then(|p| p.get("user_id").and_then(|v| v.as_str()))
                    {
                        let url = url.replace("{id}", user_id);
                        req_builder = client.get(&url);
                    } else {
                        return Err(IntegrationError::Validation(
                            "user_id parameter required".into(),
                        ));
                    }
                }
            }
            _ => {}
        }

        // Add query parameters to the request
        for (key, value) in query_params {
            req_builder = req_builder.query(&[(key, value)]);
        }

        // Send request
        let response = req_builder.send().await?;

        // Check for errors
        if !response.status().is_success() {
            let error_body = response.text().await?;
            return Err(IntegrationError::ExternalApi(format!(
                "Twitter API request failed with status: {}. Details: {}",
                response.status(),
                error_body
            )));
        }

        // Parse response
        let response_body = response.text().await?;
        let response_data: Value = serde_json::from_str(&response_body).map_err(|e| {
            IntegrationError::ExternalApi(format!("Failed to parse response as JSON: {}", e))
        })?;

        // Count results from Twitter's data structure
        let result_count = if let Some(data) = response_data.get("data") {
            if data.is_array() {
                Some(data.as_array().unwrap().len() as i32)
            } else if data.is_object() {
                Some(1) // Single object result
            } else {
                None
            }
        } else if let Some(meta) = response_data.get("meta") {
            meta.get("result_count")
                .and_then(|v| v.as_i64())
                .map(|v| v as i32)
        } else {
            None
        };

        Ok((result_count, Some(response_body)))
    }
}
