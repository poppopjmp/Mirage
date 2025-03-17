use crate::config::AppConfig;
use crate::models::{
    GraphVisualizationRequest, ChartVisualizationRequest, ReportGenerationRequest,
    GraphData, GraphNode, GraphEdge, VisualizationResult, Visualization, VisualizationResponse, VisualizationType, 
    ChartType
};
use crate::renderers::graph::GraphRenderer;
use crate::renderers::chart::ChartRenderer;
use mirage_common::Error;
use reqwest::Client;
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;
use std::path::Path;
use std::fs;
use base64::{Engine as _, engine::general_purpose};
use std::collections::HashMap;
use tokio::sync::Mutex;

/// Memory-based visualization store for demonstration purposes
/// In a real implementation, this would use a database
#[derive(Clone)]
pub struct VisualizationStore {
    visualizations: Arc<Mutex<HashMap<Uuid, Visualization>>>,
}

impl VisualizationStore {
    pub fn new() -> Self {
        Self {
            visualizations: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn add(&self, visualization: Visualization) -> Result<(), mirage_common::Error> {
        let mut visualizations = self.visualizations.lock().await;
        visualizations.insert(visualization.id, visualization);
        Ok(())
    }

    pub async fn get(&self, id: &Uuid) -> Result<Visualization, mirage_common::Error> {
        let visualizations = self.visualizations.lock().await;
        visualizations.get(id)
            .cloned()
            .ok_or_else(|| Error::NotFound(format!("Visualization with ID {} not found", id)))
    }

    pub async fn list(&self) -> Result<Vec<Visualization>, mirage_common::Error> {
        let visualizations = self.visualizations.lock().await;
        Ok(visualizations.values().cloned().collect())
    }
}

#[derive(Clone)]
pub struct VisualizationService {
    client: Arc<Client>,
    config: Arc<AppConfig>,
    store: VisualizationStore,
    correlation_engine_url: String,
    http_client: reqwest::Client,
    base_url: String,
}

impl VisualizationService {
    pub fn new(client: Client, config: AppConfig) -> Self {
        // Ensure the output directory exists
        let output_dir = Path::new(&config.visualization.output_dir);
        if !output_dir.exists() {
            fs::create_dir_all(output_dir).expect("Failed to create visualization output directory");
        }
        
        Self {
            client: Arc::new(client),
            config: Arc::new(config),
            store: VisualizationStore::new(),
            correlation_engine_url: config.correlation_service.url.clone(),
            http_client: client.clone(),
            base_url: "http://localhost:8088".to_string(),
        }
    }
    
    pub async fn create_graph_visualization(
        &self, 
        request: GraphVisualizationRequest
    ) -> Result<VisualizationResult, mirage_common::Error> {
        // Validate input
        if request.correlation_id.is_none() && request.entity_id.is_none() {
            return Err(Error::Validation("Either correlation_id or entity_id must be provided".into()));
        }
        
        // Get graph data by correlation ID or generate new correlation
        let graph_data = if let Some(correlation_id) = request.correlation_id {
            self.fetch_correlation_result(correlation_id).await?
        } else if let Some(entity_id) = request.entity_id {
            self.generate_correlation_for_entity(entity_id).await?
        } else {
            return Err(Error::Validation("No valid source for graph data".into()));
        };
        
        // Determine output format (default to SVG)
        let format = request.format.unwrap_or_else(|| "svg".to_string()).to_lowercase();
        let (content_type, renderer): (&str, Box<dyn GraphRenderer>) = match format.as_str() {
            "svg" => ("image/svg+xml", Box::new(crate::renderers::graph::SvgRenderer::new())),
            "json" => ("application/json", Box::new(crate::renderers::graph::JsonRenderer::new())),
            "png" => ("image/png", Box::new(crate::renderers::graph::PngRenderer::new())),
            _ => return Err(Error::Validation(format!("Unsupported format: {}", format))),
        };
        
        // Calculate dimensions
        let width = request.width.unwrap_or(self.config.visualization.default_graph_width);
        let height = request.height.unwrap_or(self.config.visualization.default_graph_height);
        
        // Render the visualization
        let rendered_data = renderer.render(&graph_data, width, height, request.options.as_ref())?;
        
        // For binary formats, base64 encode the data
        let data = if content_type.starts_with("image/") && content_type != "image/svg+xml" {
            general_purpose::STANDARD.encode(&rendered_data)
        } else {
            String::from_utf8(rendered_data)
                .map_err(|e| Error::Internal(format!("Invalid UTF-8 data: {}", e)))?
        };
        
        // Generate result
        let result = VisualizationResult {
            id: Uuid::new_v4(),
            format,
            content_type: content_type.to_string(),
            data,
            created_at: Utc::now(),
        };
        
        // Save result to disk
        let output_path = Path::new(&self.config.visualization.output_dir)
            .join(format!("graph_{}.{}", result.id, format));
        
        fs::write(&output_path, result.data.as_bytes())
            .map_err(|e| Error::Internal(format!("Failed to write visualization file: {}", e)))?;
        
        Ok(result)
    }
    
    pub async fn create_chart_visualization(
        &self, 
        request: ChartVisualizationRequest
    ) -> Result<VisualizationResult, mirage_common::Error> {
        // Validate input
        if request.entity_ids.is_empty() {
            return Err(Error::Validation("At least one entity_id must be provided".into()));
        }
        
        // Fetch data for each entity
        let entities_data = self.fetch_entities_data(&request.entity_ids).await?;
        
        // Determine output format (default to SVG)
        let format = request.format.unwrap_or_else(|| "svg".to_string()).to_lowercase();
        let (content_type, renderer): (&str, Box<dyn ChartRenderer>) = match format.as_str() {
            "svg" => ("image/svg+xml", Box::new(crate::renderers::chart::SvgChartRenderer::new())),
            "png" => ("image/png", Box::new(crate::renderers::chart::PngChartRenderer::new())),
            "json" => ("application/json", Box::new(crate::renderers::chart::JsonChartRenderer::new())),
            _ => return Err(Error::Validation(format!("Unsupported format: {}", format))),
        };
        
        // Calculate dimensions
        let width = request.width.unwrap_or(self.config.visualization.default_chart_width);
        let height = request.height.unwrap_or(self.config.visualization.default_chart_height);
        
        // Render the chart based on type
        let rendered_data = match request.data_type.to_lowercase().as_str() {
            "timeline" => renderer.render_timeline(&entities_data, width, height, request.options.as_ref())?,
            "bar" => renderer.render_bar(&entities_data, width, height, request.options.as_ref())?,
            "pie" => renderer.render_pie(&entities_data, width, height, request.options.as_ref())?,
            _ => return Err(Error::Validation(format!("Unsupported chart type: {}", request.data_type))),
        };
        
        // For binary formats, base64 encode the data
        let data = if content_type.starts_with("image/") && content_type != "image/svg+xml" {
            general_purpose::STANDARD.encode(&rendered_data)
        } else {
            String::from_utf8(rendered_data)
                .map_err(|e| Error::Internal(format!("Invalid UTF-8 data: {}", e)))?
        };
        
        // Generate result
        let result = VisualizationResult {
            id: Uuid::new_v4(),
            format,
            content_type: content_type.to_string(),
            data,
            created_at: Utc::now(),
        };
        
        // Save result to disk
        let output_path = Path::new(&self.config.visualization.output_dir)
            .join(format!("chart_{}.{}", result.id, format));
        
        fs::write(&output_path, result.data.as_bytes())
            .map_err(|e| Error::Internal(format!("Failed to write visualization file: {}", e)))?;
        
        Ok(result)
    }
    
    // Internal helper methods
    async fn fetch_correlation_result(&self, correlation_id: Uuid) -> Result<GraphData, mirage_common::Error> {
        let url = format!("{}/api/v1/correlation/results/{}", 
            self.config.correlation_service.url, correlation_id);
        
        let response = self.client.get(&url)
            .send()
            .await
            .map_err(|e| Error::ExternalApi(format!("Failed to fetch correlation result: {}", e)))?;
            
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            
            if status.as_u16() == 404 {
                return Err(Error::NotFound(format!("Correlation with ID {} not found", correlation_id)));
            } else {
                return Err(Error::ExternalApi(format!("Correlation API error ({}): {}", status, error_text)));
            }
        }
        
        let correlation_data = response.json::<serde_json::Value>().await
            .map_err(|e| Error::ExternalApi(format!("Failed to parse correlation data: {}", e)))?;
        
        // Transform correlation result to GraphData
        self.transform_correlation_to_graph_data(correlation_data)
    }
    
    async fn generate_correlation_for_entity(&self, entity_id: Uuid) -> Result<GraphData, mirage_common::Error> {
        let url = format!("{}/api/v1/correlation/correlate", self.config.correlation_service.url);
        
        let request_body = serde_json::json!({
            "entity_id": entity_id,
            "max_depth": 2,  // Reasonable default for visualization
            "min_confidence": 75
        });
        
        let response = self.client.post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| Error::ExternalApi(format!("Failed to generate correlation: {}", e)))?;
            
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            
            if status.as_u16() == 404 {
                return Err(Error::NotFound(format!("Entity with ID {} not found", entity_id)));
            } else {
                return Err(Error::ExternalApi(format!("Correlation API error ({}): {}", status, error_text)));
            }
        }
        
        let correlation_data = response.json::<serde_json::Value>().await
            .map_err(|e| Error::ExternalApi(format!("Failed to parse correlation data: {}", e)))?;
        
        // Transform correlation result to GraphData
        self.transform_correlation_to_graph_data(correlation_data)
    }
    
    fn transform_correlation_to_graph_data(&self, correlation_data: serde_json::Value) -> Result<GraphData, mirage_common::Error> {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        
        // Process nodes
        if let Some(nodes_array) = correlation_data["nodes"].as_array() {
            for node in nodes_array {
                if nodes.len() >= self.config.visualization.max_nodes {
                    break; // Limit the number of nodes for visualization
                }
                
                let id = node["id"].as_str().unwrap_or_default().to_string();
                let entity_type = node["entity_type"].as_str().unwrap_or_default().to_string();
                let value = node["value"].as_str().unwrap_or_default().to_string();
                
                let mut properties = std::collections::HashMap::new();
                if let Some(props) = node["properties"].as_object() {
                    for (k, v) in props {
                        properties.insert(k.clone(), v.clone());
                    }
                }
                
                // Create meaningful label based on entity type
                let label = match entity_type.as_str() {
                    "domain" => format!("Domain: {}", value),
                    "ip" => format!("IP: {}", value),
                    "email" => format!("Email: {}", value),
                    _ => value.clone(),
                };
                
                nodes.push(GraphNode {
                    id,
                    label,
                    entity_type,
                    value,
                    properties,
                });
            }
        }
        
        // Process edges
        if let Some(rels_array) = correlation_data["relationships"].as_array() {
            for rel in rels_array {
                let id = rel["id"].as_str().unwrap_or_default().to_string();
                let source = rel["source_id"].as_str().unwrap_or_default().to_string();
                let target = rel["target_id"].as_str().unwrap_or_default().to_string();
                let rel_type = rel["relationship_type"].as_str().unwrap_or_default().to_string();
                
                let mut properties = std::collections::HashMap::new();
                if let Some(props) = rel["properties"].as_object() {
                    for (k, v) in props {
                        properties.insert(k.clone(), v.clone());
                    }
                }
                
                edges.push(GraphEdge {
                    id,
                    source,
                    target,
                    label: rel_type.clone(),
                    properties,
                });
            }
        }
        
        Ok(GraphData { nodes, edges })
    }
    
    async fn fetch_entities_data(&self, entity_ids: &[Uuid]) -> Result<Vec<serde_json::Value>, mirage_common::Error> {
        let mut entities_data = Vec::new();
        
        for id in entity_ids {
            let url = format!("{}/api/v1/data/{}", self.config.data_storage.url, id);
            
            let response = self.client.get(&url)
                .send()
                .await
                .map_err(|e| Error::ExternalApi(format!("Failed to fetch entity data: {}", e)))?;
                
            if !response.status().is_success() {
                continue; // Skip this entity if not found
            }
            
            let entity_data = response.json::<serde_json::Value>().await
                .map_err(|e| Error::ExternalApi(format!("Failed to parse entity data: {}", e)))?;
                
            entities_data.push(entity_data);
        }
        
        Ok(entities_data)
    }

    /// Create a graph-based visualization
    pub async fn create_graph_visualization(&self, request: GraphVisualizationRequest) -> Result<VisualizationResponse, mirage_common::Error> {
        // Validate data source existence with the data storage service
        self.validate_data_source(&request.data_source_id).await?;
        
        // Create visualization object
        let id = Uuid::new_v4();
        let now = Utc::now();
        
        // Convert request to config JSON
        let config = serde_json::json!({
            "query": request.query,
            "entity_types": request.entity_types,
            "relationship_types": request.relationship_types,
            "max_nodes": request.max_nodes.unwrap_or(100),
            "max_depth": request.max_depth.unwrap_or(3),
            "layout": request.layout.unwrap_or_else(|| "force".to_string()),
            "style_options": request.style_options,
        });
        
        let visualization = Visualization {
            id,
            title: request.title,
            description: request.description,
            visualization_type: VisualizationType::Graph,
            data_source_id: request.data_source_id,
            created_at: now,
            updated_at: now,
            created_by: None, // This would come from auth context in real impl
            config,
            metadata: request.metadata.unwrap_or_default(),
            thumbnail_url: None,
        };
        
        // Save visualization
        self.store.add(visualization.clone()).await?;
        
        // Get render URL
        let render_url = format!("{}/api/v1/visualizations/render/{}", self.base_url, id);
        
        // Return response
        let response = VisualizationResponse {
            id: visualization.id,
            title: visualization.title,
            description: visualization.description,
            visualization_type: visualization.visualization_type,
            data_source_id: visualization.data_source_id,
            created_at: visualization.created_at,
            updated_at: visualization.updated_at,
            created_by: visualization.created_by,
            config: visualization.config,
            thumbnail_url: visualization.thumbnail_url,
            render_url,
        };
        
        Ok(response)
    }

    /// Create a chart-based visualization
    pub async fn create_chart_visualization(&self, request: ChartVisualizationRequest) -> Result<VisualizationResponse, mirage_common::Error> {
        // Validate data source existence with the data storage service
        self.validate_data_source(&request.data_source_id).await?;
        
        // Create visualization object
        let id = Uuid::new_v4();
        let now = Utc::now();
        
        // Convert request to config JSON
        let config = serde_json::json!({
            "query": request.query,
            "chart_type": request.chart_type,
            "x_axis": request.x_axis,
            "y_axis": request.y_axis,
            "filters": request.filters,
            "style_options": request.style_options,
        });
        
        let visualization = Visualization {
            id,
            title: request.title,
            description: request.description,
            visualization_type: VisualizationType::Chart,
            data_source_id: request.data_source_id,
            created_at: now,
            updated_at: now,
            created_by: None, // This would come from auth context in real impl
            config,
            metadata: request.metadata.unwrap_or_default(),
            thumbnail_url: None,
        };
        
        // Save visualization
        self.store.add(visualization.clone()).await?;
        
        // Get render URL
        let render_url = format!("{}/api/v1/visualizations/render/{}", self.base_url, id);
        
        // Return response
        let response = VisualizationResponse {
            id: visualization.id,
            title: visualization.title,
            description: visualization.description,
            visualization_type: visualization.visualization_type,
            data_source_id: visualization.data_source_id,
            created_at: visualization.created_at,
            updated_at: visualization.updated_at,
            created_by: visualization.created_by,
            config: visualization.config,
            thumbnail_url: visualization.thumbnail_url,
            render_url,
        };
        
        Ok(response)
    }

    /// Get a visualization by ID
    pub async fn get_visualization(&self, id: &Uuid) -> Result<VisualizationResponse, mirage_common::Error> {
        let visualization = self.store.get(id).await?;
        
        // Get render URL
        let render_url = format!("{}/api/v1/visualizations/render/{}", self.base_url, id);
        
        // Return response
        let response = VisualizationResponse {
            id: visualization.id,
            title: visualization.title,
            description: visualization.description,
            visualization_type: visualization.visualization_type,
            data_source_id: visualization.data_source_id,
            created_at: visualization.created_at,
            updated_at: visualization.updated_at,
            created_by: visualization.created_by,
            config: visualization.config,
            thumbnail_url: visualization.thumbnail_url,
            render_url,
        };
        
        Ok(response)
    }

    /// Helper method to validate data source exists
    async fn validate_data_source(&self, data_source_id: &Uuid) -> Result<(), mirage_common::Error> {
        // In a real implementation, this would call the data storage service
        // For now, just simulate the validation
        if data_source_id.as_u128() % 10 == 0 {
            return Err(Error::NotFound(format!("Data source with ID {} not found", data_source_id)));
        }
        Ok(())
    }
}
