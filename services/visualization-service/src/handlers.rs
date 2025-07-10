use actix_web::{get, post, web, Error, HttpResponse, Responder};
use mirage_common::Error as CommonError;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::AppConfig;
use crate::models::{
    ChartVisualizationRequest, GraphVisualizationRequest, ReportGenerationRequest,
};
use crate::services::VisualizationService;

pub fn visualization_routes() -> actix_web::Scope {
    web::scope("/visualizations")
        .service(create_graph)
        .service(create_chart)
        .service(get_visualization)
        .service(render_visualization)
        .service(list_visualizations)
        .service(create_report)
}

#[post("/graph")]
async fn create_graph(
    request: web::Json<GraphVisualizationRequest>,
    viz_service: web::Data<VisualizationService>,
) -> Result<HttpResponse, Error> {
    let result = viz_service
        .create_graph_visualization(request.into_inner())
        .await
        .map_err(|e| {
            tracing::error!("Failed to create graph visualization: {}", e);
            match e {
                CommonError::NotFound(_) => actix_web::error::ErrorNotFound(e),
                CommonError::Validation(_) => actix_web::error::ErrorBadRequest(e),
                _ => actix_web::error::ErrorInternalServerError(e),
            }
        })?;

    Ok(HttpResponse::Created().json(result))
}

#[post("/chart")]
async fn create_chart(
    request: web::Json<ChartVisualizationRequest>,
    viz_service: web::Data<VisualizationService>,
) -> Result<HttpResponse, Error> {
    let result = viz_service
        .create_chart_visualization(request.into_inner())
        .await
        .map_err(|e| {
            tracing::error!("Failed to create chart visualization: {}", e);
            match e {
                CommonError::NotFound(_) => actix_web::error::ErrorNotFound(e),
                CommonError::Validation(_) => actix_web::error::ErrorBadRequest(e),
                _ => actix_web::error::ErrorInternalServerError(e),
            }
        })?;

    Ok(HttpResponse::Created().json(result))
}

#[get("/{id}")]
async fn get_visualization(
    id: web::Path<String>,
    viz_service: web::Data<VisualizationService>,
) -> Result<HttpResponse, Error> {
    let visualization_id = Uuid::parse_str(&id)
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid visualization ID format"))?;

    let result = viz_service
        .get_visualization(&visualization_id)
        .await
        .map_err(|e| match e {
            CommonError::NotFound(_) => actix_web::error::ErrorNotFound(e),
            _ => {
                tracing::error!("Failed to get visualization: {}", e);
                actix_web::error::ErrorInternalServerError(e)
            }
        })?;

    Ok(HttpResponse::Ok().json(result))
}

#[get("/render/{id}")]
async fn render_visualization(
    id: web::Path<String>,
    query: web::Query<RenderOptions>,
    viz_service: web::Data<VisualizationService>,
    config: web::Data<AppConfig>,
) -> Result<HttpResponse, Error> {
    let visualization_id = Uuid::parse_str(&id)
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid visualization ID format"))?;

    let format = query.format.clone().unwrap_or_else(|| "svg".to_string());
    let width = query
        .width
        .unwrap_or(config.visualization.default_graph_width);
    let height = query
        .height
        .unwrap_or(config.visualization.default_graph_height);

    let result = viz_service
        .render_visualization(&visualization_id, &format, width, height)
        .await
        .map_err(|e| match e {
            CommonError::NotFound(_) => actix_web::error::ErrorNotFound(e),
            CommonError::Validation(_) => actix_web::error::ErrorBadRequest(e),
            _ => {
                tracing::error!("Failed to render visualization: {}", e);
                actix_web::error::ErrorInternalServerError(e)
            }
        })?;

    let content_type = match format.as_str() {
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "json" => "application/json",
        _ => "text/plain",
    };

    Ok(HttpResponse::Ok().content_type(content_type).body(result))
}

#[derive(Debug, Deserialize)]
struct RenderOptions {
    format: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
}

#[get("")]
async fn list_visualizations(
    viz_service: web::Data<VisualizationService>,
) -> Result<HttpResponse, Error> {
    let result = viz_service.list_visualizations().await.map_err(|e| {
        tracing::error!("Failed to list visualizations: {}", e);
        actix_web::error::ErrorInternalServerError(e)
    })?;

    Ok(HttpResponse::Ok().json(result))
}

#[post("/reports")]
async fn create_report(
    request: web::Json<ReportGenerationRequest>,
    viz_service: web::Data<VisualizationService>,
) -> Result<HttpResponse, Error> {
    let result = viz_service
        .generate_report(request.into_inner())
        .await
        .map_err(|e| {
            tracing::error!("Failed to generate report: {}", e);
            match e {
                CommonError::NotFound(_) => actix_web::error::ErrorNotFound(e),
                CommonError::Validation(_) => actix_web::error::ErrorBadRequest(e),
                _ => actix_web::error::ErrorInternalServerError(e),
            }
        })?;

    Ok(HttpResponse::Created().json(result))
}
