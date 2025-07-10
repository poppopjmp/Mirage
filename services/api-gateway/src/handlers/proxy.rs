use crate::AppState;
use actix_web::{web, HttpRequest, HttpResponse};

pub async fn proxy_request(
    req: HttpRequest,
    body: web::Bytes,
    state: web::Data<AppState>,
) -> HttpResponse {
    // Extract service from path
    let path = req.path();
    let path_segments: Vec<&str> = path.split('/').collect();

    // Path should be /api/v1/service/...
    if path_segments.len() < 4 {
        return HttpResponse::BadRequest().body("Invalid path");
    }

    let service_name = match path_segments[3] {
        "users" => "user-management",
        "scans" => "scan-orchestration",
        "modules" => "module-registry",
        "data" => "data-storage",
        "analyze" => "correlation-engine",
        "visualize" => "visualization",
        "reports" => "reporting",
        "notifications" => "notification",
        "config" => "configuration",
        "auth" => "auth",
        "integrations" => "integration",
        "discovery" => "discovery",
        _ => return HttpResponse::NotFound().body("Service not found"),
    };

    // Get service endpoint
    let service_url = match state.service_endpoints.get(service_name) {
        Some(url) => url,
        None => return HttpResponse::ServiceUnavailable().body("Service endpoint not configured"),
    };

    // Create the target URL
    let target_path = path_segments[3..].join("/");
    let target_url = format!("{}/api/v1/{}", service_url, target_path);

    // Forward the request
    let client = reqwest::Client::new();
    let mut request_builder = match req.method().as_str() {
        "GET" => client.get(&target_url),
        "POST" => client.post(&target_url).body(body),
        "PUT" => client.put(&target_url).body(body),
        "DELETE" => client.delete(&target_url),
        "PATCH" => client.patch(&target_url).body(body),
        _ => return HttpResponse::MethodNotAllowed().finish(),
    };

    // Copy headers
    for (header_name, header_value) in req.headers() {
        // Skip connection-specific headers
        if header_name == "connection" || header_name == "host" {
            continue;
        }
        request_builder = request_builder.header(header_name, header_value);
    }

    // Execute the request
    match request_builder.send().await {
        Ok(response) => {
            // Create response builder
            let mut builder = HttpResponse::build(response.status());

            // Copy headers
            for (name, value) in response.headers() {
                builder.insert_header((name.clone(), value.clone()));
            }

            // Set body and return
            match response.bytes().await {
                Ok(bytes) => builder.body(bytes),
                Err(_) => HttpResponse::InternalServerError().body("Failed to read response body"),
            }
        }
        Err(e) => {
            log::error!("Proxy request error: {}", e);
            HttpResponse::InternalServerError().body(format!("Proxy request failed: {}", e))
        }
    }
}
