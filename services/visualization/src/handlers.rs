use actix_web::{web, HttpResponse, Responder, Error, post, get};
use mirage_common::Error as CommonError;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::services::VisualizationService;
use crate::models::{GraphVisualizationRequest, ChartVisualizationRequest, ReportGenerationRequest};

pub fn visualization_routes() -> actix_web::Scope {
    web::scope("/visualizations")
        .service(create_graph)
        .service(create_chart)
        .service(get_visualization)
}

#[post("/graph")]
async fn create_graph(
    request: web::Json<GraphVisualizationRequest>,
    viz_service: web::Data<VisualizationService>,
) -> Result<HttpResponse, Error> {
    let result = viz_service.create_graph_visualization(request.into_inner()).await
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
    let result = viz_service.create_chart_visualization(request.into_inner()).await
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
    // Parse UUID from path
    let viz_id = Uuid::parse_str(&id).map_err(|_| {
        actix_web::error::ErrorBadRequest("Invalid visualization ID format")
    })?;
    
    // For a real implementation, this would fetch the visualization from storage
    // For now, return a simple error
    Err(actix_web::error::ErrorNotFound("Visualization not found"))
}
