use crate::analysis::{self, CorrelationAnalyzer};
use crate::config::AppConfig;
use crate::models::{
    AnalysisJob, AnalysisJobType, BatchCorrelationRequest, CorrelationInsight,
    CorrelationParameters, CorrelationRequest, CorrelationResult, EntityImportance, EntityNode,
    EntityPath, GraphNode, GraphRelationship, JobStatus, PathFindingRequest, PathFindingResult,
    PatternMatch, PatternMatchRequest, PatternMatchResult, Relationship,
};
use crate::repositories::{DataStorageRepository, GraphDatabase, GraphRepository};
use chrono::Utc;
use mirage_common::{Error, Result};
use neo4rs::Graph;
use reqwest::Client as HttpClient;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{self, Duration};
use uuid::Uuid;

#[derive(Clone)]
pub struct CorrelationService {
    graph_repo: Arc<GraphRepository>,
    data_storage_repo: Arc<DataStorageRepository>,
    config: Arc<AppConfig>,
    graph_db: Arc<GraphDatabase>,
    http_client: Arc<HttpClient>,
    analyzer: Arc<CorrelationAnalyzer>,
    active_jobs: Arc<Mutex<HashMap<Uuid, JobStatus>>>,
}

impl CorrelationService {
    pub fn new(graph: Graph, http_client: HttpClient, config: AppConfig) -> Self {
        Self {
            graph_repo: Arc::new(GraphRepository::new(graph)),
            data_storage_repo: Arc::new(DataStorageRepository::new(
                http_client.clone(),
                config.data_storage.url.clone(),
            )),
            config: Arc::new(config),
            graph_db: Arc::new(GraphDatabase::new(graph)),
            http_client: Arc::new(http_client),
            analyzer: Arc::new(CorrelationAnalyzer {}),
            active_jobs: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    // Import entity and its relationships from data storage into graph database
    pub async fn import_entity(&self, entity_id: &Uuid) -> Result<EntityNode> {
        // Check if entity already exists in graph
        if let Some(entity) = self.graph_repo.get_entity_by_id(entity_id).await? {
            return Ok(entity);
        }

        // Fetch entity from data storage
        let entity_data = self.data_storage_repo.get_entity(entity_id).await?;

        // Convert to EntityNode
        let entity_type = entity_data["entity_type"]
            .as_str()
            .ok_or_else(|| Error::ExternalApi("Missing entity_type in data".to_string()))?
            .to_string();

        let value = entity_data["value"]
            .as_str()
            .ok_or_else(|| Error::ExternalApi("Missing value in data".to_string()))?
            .to_string();

        // Extract properties
        let mut properties = HashMap::new();
        if let Some(data) = entity_data["data"].as_object() {
            for (k, v) in data {
                properties.insert(k.clone(), v.clone());
            }
        }

        // Extract metadata into properties
        if let Some(metadata) = entity_data["metadata"].as_object() {
            for (k, v) in metadata {
                if let Some(v_str) = v.as_str() {
                    properties.insert(k.clone(), serde_json::Value::String(v_str.to_string()));
                }
            }
        }

        let confidence = entity_data["confidence"].as_u64().unwrap_or(70) as u8;

        let created_at = if let Some(created_at_str) = entity_data["created_at"].as_str() {
            chrono::DateTime::parse_from_rfc3339(created_at_str)
                .map_err(|_| Error::ExternalApi("Invalid created_at format".to_string()))?
                .with_timezone(&Utc)
        } else {
            Utc::now()
        };

        // Create entity node
        let entity = EntityNode {
            id: *entity_id,
            entity_type,
            value,
            properties,
            confidence,
            created_at,
        };

        // Store in graph database
        self.graph_repo.create_entity_node(&entity).await?;

        // Also import relationships
        let relationships = self.data_storage_repo.get_relationships(entity_id).await?;

        for rel_data in relationships {
            let source_id = Uuid::parse_str(rel_data["source_id"].as_str().unwrap_or_default())
                .map_err(|_| Error::ExternalApi("Invalid source_id format".to_string()))?;

            let target_id = Uuid::parse_str(rel_data["target_id"].as_str().unwrap_or_default())
                .map_err(|_| Error::ExternalApi("Invalid target_id format".to_string()))?;

            // Make sure both source and target entities are imported
            if source_id != *entity_id {
                self.import_entity(&source_id).await?;
            }

            if target_id != *entity_id {
                self.import_entity(&target_id).await?;
            }

            let rel_type = rel_data["relationship_type"]
                .as_str()
                .ok_or_else(|| Error::ExternalApi("Missing relationship_type in data".to_string()))?
                .to_string();

            // Extract properties
            let mut properties = HashMap::new();
            if let Some(data) = rel_data["data"].as_object() {
                for (k, v) in data {
                    properties.insert(k.clone(), v.clone());
                }
            }

            let rel_id = if let Some(id_str) = rel_data["id"].as_str() {
                Uuid::parse_str(id_str)
                    .map_err(|_| Error::ExternalApi("Invalid relationship id format".to_string()))?
            } else {
                Uuid::new_v4()
            };

            let confidence = rel_data["confidence"].as_u64().unwrap_or(70) as u8;

            let created_at = if let Some(created_at_str) = rel_data["created_at"].as_str() {
                chrono::DateTime::parse_from_rfc3339(created_at_str)
                    .map_err(|_| Error::ExternalApi("Invalid created_at format".to_string()))?
                    .with_timezone(&Utc)
            } else {
                Utc::now()
            };

            // Create relationship
            let relationship = Relationship {
                id: rel_id,
                source_id,
                target_id,
                relationship_type: rel_type,
                properties,
                confidence,
                created_at,
            };

            self.graph_repo.create_relationship(&relationship).await?;
        }

        Ok(entity)
    }

    // Run correlation on an entity
    pub async fn correlate(&self, req: CorrelationRequest) -> Result<CorrelationResult> {
        // Import the entity if not already in graph
        self.import_entity(&req.entity_id).await?;

        // Set correlation parameters
        let max_depth = req
            .max_depth
            .unwrap_or(self.config.analysis.max_correlation_depth);
        let min_confidence = req
            .min_confidence
            .unwrap_or(self.config.analysis.min_confidence_score);

        // Get correlated entities and relationships
        let (nodes, relationships) = self
            .graph_repo
            .get_neighbors(&req.entity_id, max_depth, min_confidence)
            .await?;

        // Add the starting node if not included
        let mut result_nodes = nodes;
        let start_node_included = result_nodes.iter().any(|n| n.id == req.entity_id);

        if !start_node_included {
            if let Some(start_node) = self.graph_repo.get_entity_by_id(&req.entity_id).await? {
                result_nodes.push(start_node);
            }
        }

        // Create correlation result
        let correlation_id = Uuid::new_v4();
        let result = CorrelationResult {
            nodes: result_nodes,
            relationships,
            correlation_id,
            created_at: Utc::now(),
            parameters: CorrelationParameters {
                start_entity_id: req.entity_id,
                max_depth,
                min_confidence,
            },
        };

        Ok(result)
    }

    // Run pattern matching
    pub async fn match_pattern(&self, req: PatternMatchRequest) -> Result<PatternMatchResult> {
        // Lookup appropriate pattern matching algorithm
        let pattern_matches = match req.pattern_type.as_str() {
            "ip_domain_relationship" => {
                analysis::patterns::find_ip_domain_relationships(
                    self.graph_repo.clone(),
                    req.min_confidence
                        .unwrap_or(self.config.analysis.min_confidence_score),
                    req.parameters.clone(),
                )
                .await?
            }
            "email_clusters" => {
                analysis::patterns::find_email_clusters(
                    self.graph_repo.clone(),
                    req.min_confidence
                        .unwrap_or(self.config.analysis.min_confidence_score),
                    req.parameters.clone(),
                )
                .await?
            }
            "infrastructure_groups" => {
                analysis::patterns::find_infrastructure_groups(
                    self.graph_repo.clone(),
                    req.min_confidence
                        .unwrap_or(self.config.analysis.min_confidence_score),
                    req.parameters.clone(),
                )
                .await?
            }
            _ => {
                return Err(Error::Validation(format!(
                    "Unknown pattern type: {}",
                    req.pattern_type
                )))
            }
        };

        // Create result
        let result = PatternMatchResult {
            pattern_type: req.pattern_type,
            matches: pattern_matches,
            match_id: Uuid::new_v4(),
            created_at: Utc::now(),
        };

        Ok(result)
    }

    // Enrich an entity with additional information
    pub async fn enrich_entity(&self, req: &EntityEnrichmentRequest) -> Result<EntityNode> {
        // Get the entity node
        let entity = self
            .graph_repo
            .get_entity_by_id(&req.entity_id)
            .await?
            .ok_or_else(|| {
                Error::NotFound(format!("Entity with ID {} not found", req.entity_id))
            })?;

        // Apply enrichment types
        let mut enriched_entity = entity.clone();
        let mut updated = false;

        for enrichment_type in &req.enrichment_types {
            match enrichment_type.as_str() {
                "domain_whois" => {
                    if entity.entity_type == "domain" {
                        updated =
                            analysis::enrichment::enrich_domain_whois(&mut enriched_entity).await?;
                    }
                }
                "ip_geolocation" => {
                    if entity.entity_type == "ip" {
                        updated = analysis::enrichment::enrich_ip_geolocation(&mut enriched_entity)
                            .await?;
                    }
                }
                "email_breach_check" => {
                    if entity.entity_type == "email" {
                        updated =
                            analysis::enrichment::enrich_email_breach_check(&mut enriched_entity)
                                .await?;
                    }
                }
                _ => {
                    return Err(Error::Validation(format!(
                        "Unknown enrichment type: {}",
                        enrichment_type
                    )))
                }
            }
        }

        // Update the entity in the graph if it was enriched
        if updated {
            self.graph_repo.update_entity(&enriched_entity).await?;
        }

        Ok(enriched_entity)
    }

    // Generate correlation for a single entity
    pub async fn generate_correlation(
        &self,
        request: CorrelationRequest,
    ) -> Result<CorrelationResult> {
        // Validate that entity exists
        let entity = self
            .graph_db
            .get_node(&request.entity_id)
            .await?
            .ok_or_else(|| {
                Error::NotFound(format!("Entity with ID {} not found", request.entity_id))
            })?;

        // Set up defaults
        let max_depth = request
            .max_depth
            .unwrap_or_else(|| self.config.engine.max_correlation_depth);
        let min_confidence = request
            .min_confidence
            .unwrap_or_else(|| self.config.engine.min_correlation_confidence);
        let max_entities = request
            .max_entities
            .unwrap_or_else(|| self.config.engine.max_entities_per_correlation);

        // Fetch connected nodes up to depth
        let nodes = self
            .graph_db
            .get_connected_nodes(&request.entity_id, max_depth, max_entities)
            .await?;

        // Add the entity itself if not already in the nodes list
        let mut all_nodes = nodes;
        if !all_nodes.iter().any(|n| n.id == request.entity_id) {
            all_nodes.push(entity);
        }

        // Apply entity type filtering
        all_nodes = self.filter_nodes_by_type(
            all_nodes,
            request.include_entities,
            request.exclude_entities,
        )?;

        // Fetch relationships between these nodes
        let mut relationships = Vec::new();
        let node_ids: HashSet<Uuid> = all_nodes.iter().map(|n| n.id).collect();

        for node in &all_nodes {
            let node_relationships = self
                .graph_db
                .get_relationships(&node.id, max_depth, max_entities)
                .await?;

            // Only include relationships where both ends are in our node set
            for rel in node_relationships {
                if rel.confidence >= min_confidence
                    && node_ids.contains(&rel.source_id)
                    && node_ids.contains(&rel.target_id)
                {
                    // Apply relationship filtering
                    let rel_type = rel.relationship_type.clone();

                    let include = if let Some(include_types) = &request.include_relationships {
                        include_types.contains(&rel_type)
                    } else {
                        true
                    };

                    let exclude = if let Some(exclude_types) = &request.exclude_relationships {
                        exclude_types.contains(&rel_type)
                    } else {
                        false
                    };

                    if include
                        && !exclude
                        && !relationships
                            .iter()
                            .any(|r: &GraphRelationship| r.id == rel.id)
                    {
                        relationships.push(rel);
                    }
                }
            }
        }

        // Calculate importance metrics for entities
        for node in &mut all_nodes {
            if let Ok(importance) = self.graph_db.calculate_entity_importance(&node.id).await {
                node.importance = importance.total_score;
            }
        }

        // Generate insights
        let insights = self
            .analyzer
            .analyze_correlation(&all_nodes, &relationships);

        // Create result
        let result = CorrelationResult {
            id: Uuid::new_v4(),
            created_at: Utc::now(),
            query_entity_id: request.entity_id,
            depth: max_depth,
            node_count: all_nodes.len() as i32,
            relationship_count: relationships.len() as i32,
            nodes: all_nodes,
            relationships,
            insights,
            metadata: HashMap::new(),
        };

        // Save result to database
        self.graph_db.save_correlation_result(&result).await?;

        Ok(result)
    }

    // Generate batch correlations (async job)
    pub async fn generate_batch_correlation(
        &self,
        request: BatchCorrelationRequest,
    ) -> Result<AnalysisJob> {
        // Create job
        let job_id = Uuid::new_v4();
        let job = AnalysisJob {
            id: job_id,
            job_type: AnalysisJobType::Correlation,
            entity_ids: request.entity_ids.clone(),
            parameters: HashMap::from_iter([
                (
                    "max_depth".to_string(),
                    serde_json::to_value(request.max_depth)?,
                ),
                (
                    "min_confidence".to_string(),
                    serde_json::to_value(request.min_confidence)?,
                ),
                (
                    "include_entities".to_string(),
                    serde_json::to_value(request.include_entities)?,
                ),
                (
                    "exclude_entities".to_string(),
                    serde_json::to_value(request.exclude_entities)?,
                ),
                (
                    "include_relationships".to_string(),
                    serde_json::to_value(request.include_relationships)?,
                ),
                (
                    "exclude_relationships".to_string(),
                    serde_json::to_value(request.exclude_relationships)?,
                ),
                (
                    "max_entities".to_string(),
                    serde_json::to_value(request.max_entities)?,
                ),
            ]),
            status: JobStatus::Pending,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            result_id: None,
            error: None,
        };

        // Add to active jobs
        {
            let mut active_jobs = self.active_jobs.lock().await;
            active_jobs.insert(job_id, JobStatus::Pending);
        }

        // Clone required data for background job
        let service_clone = self.clone();
        let request_clone = request.clone();

        // Start background job
        tokio::spawn(async move {
            // Wait a moment to allow response to be sent
            time::sleep(Duration::from_millis(100)).await;

            // Mark job as running
            {
                let mut active_jobs = service_clone.active_jobs.lock().await;
                active_jobs.insert(job_id, JobStatus::Running);
            }

            // Process each entity
            let mut result_ids = Vec::new();
            let mut failed = false;
            let mut error_message = None;

            for entity_id in request_clone.entity_ids {
                // Create single correlation request
                let single_request = CorrelationRequest {
                    entity_id,
                    max_depth: request_clone.max_depth,
                    min_confidence: request_clone.min_confidence,
                    include_entities: request_clone.include_entities.clone(),
                    exclude_entities: request_clone.exclude_entities.clone(),
                    include_relationships: request_clone.include_relationships.clone(),
                    exclude_relationships: request_clone.exclude_relationships.clone(),
                    max_entities: request_clone.max_entities,
                };

                // Generate correlation
                match service_clone.generate_correlation(single_request).await {
                    Ok(result) => result_ids.push(result.id),
                    Err(e) => {
                        tracing::error!(
                            "Failed to generate correlation for entity {}: {}",
                            entity_id,
                            e
                        );
                        failed = true;
                        error_message =
                            Some(format!("Error processing entity {}: {}", entity_id, e));
                    }
                }
            }

            // Mark job as complete or failed
            let final_status = if failed {
                JobStatus::Failed
            } else {
                JobStatus::Completed
            };

            {
                let mut active_jobs = service_clone.active_jobs.lock().await;
                active_jobs.insert(job_id, final_status);
            }

            // In a real implementation, would store job results to database
        });

        Ok(job)
    }

    // Path finding between two entities
    pub async fn find_path(&self, request: PathFindingRequest) -> Result<PathFindingResult> {
        // Validate that both entities exist
        let source_entity = self
            .graph_db
            .get_node(&request.source_entity_id)
            .await?
            .ok_or_else(|| {
                Error::NotFound(format!(
                    "Source entity with ID {} not found",
                    request.source_entity_id
                ))
            })?;

        let target_entity = self
            .graph_db
            .get_node(&request.target_entity_id)
            .await?
            .ok_or_else(|| {
                Error::NotFound(format!(
                    "Target entity with ID {} not found",
                    request.target_entity_id
                ))
            })?;

        // Set up defaults
        let max_depth = request.max_depth.unwrap_or(4);
        let min_confidence = request.min_confidence.unwrap_or(70);

        // Find paths between entities
        let raw_paths = self
            .graph_db
            .find_paths(
                &request.source_entity_id,
                &request.target_entity_id,
                max_depth,
            )
            .await?;

        // Convert to entity paths
        let mut paths = Vec::new();

        for raw_path in raw_paths {
            let mut nodes = Vec::new();
            let mut relationships = Vec::new();
            let mut path_confidence = 0.0;
            let mut path_strength = 0.0;

            for (node, opt_relationship) in raw_path {
                nodes.push(node);

                if let Some(rel) = opt_relationship {
                    relationships.push(rel.clone());
                    path_confidence += rel.confidence as f32;
                    path_strength += rel.strength;
                }
            }

            // Calculate average confidence and strength
            if !relationships.is_empty() {
                path_confidence /= relationships.len() as f32;
                path_strength /= relationships.len() as f32;
            }

            paths.push(EntityPath {
                path_length: nodes.len() as i32 - 1,
                nodes,
                relationships,
                total_confidence: path_confidence,
                path_strength,
            });
        }

        // Filter paths based on entity and relationship type filters
        paths = self.filter_paths(
            paths,
            request.include_entity_types,
            request.exclude_entity_types,
            request.include_relationship_types,
            request.exclude_relationship_types,
        )?;

        // Sort paths by length (shortest first)
        paths.sort_by_key(|p| p.path_length);

        // Generate insights
        let insights = self.analyzer.analyze_paths(&paths);

        // Create result
        let result = PathFindingResult {
            id: Uuid::new_v4(),
            created_at: Utc::now(),
            source_entity_id: request.source_entity_id,
            target_entity_id: request.target_entity_id,
            paths,
            insights,
        };

        // Save result to database
        self.graph_db.save_pathfinding_result(&result).await?;

        Ok(result)
    }

    // Helper methods

    fn filter_nodes_by_type(
        &self,
        nodes: Vec<GraphNode>,
        include_types: Option<Vec<String>>,
        exclude_types: Option<Vec<String>>,
    ) -> Result<Vec<GraphNode>> {
        if include_types.is_none() && exclude_types.is_none() {
            return Ok(nodes);
        }

        let filtered = nodes
            .into_iter()
            .filter(|node| {
                let include = if let Some(include) = &include_types {
                    include.contains(&node.entity_type)
                } else {
                    true
                };

                let exclude = if let Some(exclude) = &exclude_types {
                    exclude.contains(&node.entity_type)
                } else {
                    false
                };

                include && !exclude
            })
            .collect();

        Ok(filtered)
    }

    fn filter_paths(
        &self,
        paths: Vec<EntityPath>,
        include_entity_types: Option<Vec<String>>,
        exclude_entity_types: Option<Vec<String>>,
        include_relationship_types: Option<Vec<String>>,
        exclude_relationship_types: Option<Vec<String>>,
    ) -> Result<Vec<EntityPath>> {
        if include_entity_types.is_none()
            && exclude_entity_types.is.none()
            && include_relationship_types.is.none()
            && exclude_relationship_types.is.none()
        {
            return Ok(paths);
        }

        let filtered = paths
            .into_iter()
            .filter(|path| {
                // Check entities
                let entities_valid = path.nodes.iter().all(|node| {
                    let include = if let Some(include) = &include_entity_types {
                        include.contains(&node.entity_type)
                    } else {
                        true
                    };

                    let exclude = if let Some(exclude) = &exclude_entity_types {
                        exclude.contains(&node.entity_type)
                    } else {
                        false
                    };

                    include && !exclude
                });

                // Check relationships
                let relationships_valid = path.relationships.iter().all(|rel| {
                    let include = if let Some(include) = &include_relationship_types {
                        include.contains(&rel.relationship_type)
                    } else {
                        true
                    };

                    let exclude = if let Some(exclude) = &exclude_relationship_types {
                        exclude.contains(&rel.relationship_type)
                    } else {
                        false
                    };

                    include && !exclude
                });

                entities_valid && relationships_valid
            })
            .collect();

        Ok(filtered)
    }

    // Get job status
    pub async fn get_job_status(&self, job_id: &Uuid) -> Result<Option<JobStatus>> {
        let active_jobs = self.active_jobs.lock().await;

        if let Some(status) = active_jobs.get(job_id) {
            Ok(Some(status.clone()))
        } else {
            // In a real implementation, we'd check the database for completed/failed jobs
            Ok(None)
        }
    }
}

// Start background correlation of newly discovered entities
pub async fn start_background_correlation(service: web::Data<CorrelationService>) {
    tracing::info!("Starting background correlation worker");

    // In a real implementation, this would subscribe to events from the data service
    // and automatically trigger correlations for new entities of interest

    loop {
        // Sleep between correlation attempts
        let interval = service.config.engine.background_job_interval_seconds;
        time::sleep(Duration::from_secs(interval)).await;

        // In a real implementation, we would:
        // 1. Check for new entities that need correlation
        // 2. Generate correlations in the background
        // 3. Save results to the database
    }
}
