from fastapi import FastAPI, HTTPException, Depends, Header
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel, Field
import httpx
import os
import logging
import uuid
from typing import Dict, Any, List, Optional

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
)
logger = logging.getLogger("integration-service")

# Initialize FastAPI app
app = FastAPI(
    title="Mirage Integration Service",
    description="Service that handles integration with external systems and data sources",
    version="0.1.0",
)

# Add CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # In production, specify actual origins
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Environment configuration
DATA_STORAGE_URL = os.getenv("DATA_STORAGE_URL", "http://data-storage-service:8086")

# Models
class IntegrationSource(BaseModel):
    name: str
    type: str
    config: Dict[str, Any]
    
class IntegrationConfig(BaseModel):
    id: Optional[str] = Field(None, description="Integration ID")
    name: str
    description: Optional[str] = None
    source: IntegrationSource
    enabled: bool = True
    schedule: Optional[str] = None  # Cron format

class IntegrationResult(BaseModel):
    id: str
    integration_id: str
    status: str
    entities_added: int
    entities_updated: int
    error_message: Optional[str] = None
    start_time: str
    end_time: Optional[str] = None

# In-memory storage (replace with database in production)
integrations = {}
integration_results = {}

# Authentication dependency
async def get_token_header(x_token: str = Header(...)):
    if x_token != "secret-token":  # In production, use proper authentication
        raise HTTPException(status_code=400, detail="Invalid token")
    return x_token

# Routes
@app.get("/")
async def root():
    return {"message": "Mirage Integration Service"}

@app.get("/health")
async def health_check():
    return {"status": "ok", "service": "integration"}

@app.post("/api/v1/integrations", response_model=IntegrationConfig)
async def create_integration(
    integration: IntegrationConfig,
    token: str = Depends(get_token_header)
):
    integration_id = str(uuid.uuid4())
    integration.id = integration_id
    integrations[integration_id] = integration
    logger.info(f"Created integration: {integration_id}")
    return integration

@app.get("/api/v1/integrations", response_model=List[IntegrationConfig])
async def list_integrations(token: str = Depends(get_token_header)):
    return list(integrations.values())

@app.get("/api/v1/integrations/{integration_id}", response_model=IntegrationConfig)
async def get_integration(
    integration_id: str,
    token: str = Depends(get_token_header)
):
    if integration_id not in integrations:
        raise HTTPException(status_code=404, detail="Integration not found")
    return integrations[integration_id]

@app.put("/api/v1/integrations/{integration_id}", response_model=IntegrationConfig)
async def update_integration(
    integration_id: str,
    integration: IntegrationConfig,
    token: str = Depends(get_token_header)
):
    if integration_id not in integrations:
        raise HTTPException(status_code=404, detail="Integration not found")
    
    integration.id = integration_id
    integrations[integration_id] = integration
    logger.info(f"Updated integration: {integration_id}")
    return integration

@app.delete("/api/v1/integrations/{integration_id}")
async def delete_integration(
    integration_id: str,
    token: str = Depends(get_token_header)
):
    if integration_id not in integrations:
        raise HTTPException(status_code=404, detail="Integration not found")
    
    del integrations[integration_id]
    logger.info(f"Deleted integration: {integration_id}")
    return {"status": "deleted", "id": integration_id}

@app.post("/api/v1/integrations/{integration_id}/run", response_model=IntegrationResult)
async def run_integration(
    integration_id: str,
    token: str = Depends(get_token_header)
):
    if integration_id not in integrations:
        raise HTTPException(status_code=404, detail="Integration not found")
    
    integration = integrations[integration_id]
    
    # Create a new result record
    result_id = str(uuid.uuid4())
    result = IntegrationResult(
        id=result_id,
        integration_id=integration_id,
        status="running",
        entities_added=0,
        entities_updated=0,
        start_time=str(datetime.datetime.now(datetime.timezone.utc))
    )
    integration_results[result_id] = result
    
    # In a real implementation, this would run the integration asynchronously
    # For now, we'll just simulate success
    # In the future, implement actual integration logic based on the type
    
    # Update the result
    result.status = "completed"
    result.entities_added = 10
    result.entities_updated = 5
    result.end_time = str(datetime.datetime.now(datetime.timezone.utc))
    
    logger.info(f"Ran integration {integration_id}, result: {result_id}")
    return result

@app.get("/api/v1/integration-results", response_model=List[IntegrationResult])
async def list_integration_results(
    integration_id: Optional[str] = None,
    token: str = Depends(get_token_header)
):
    results = list(integration_results.values())
    if integration_id:
        results = [r for r in results if r.integration_id == integration_id]
    return results

@app.get("/api/v1/integration-results/{result_id}", response_model=IntegrationResult)
async def get_integration_result(
    result_id: str,
    token: str = Depends(get_token_header)
):
    if result_id not in integration_results:
        raise HTTPException(status_code=404, detail="Integration result not found")
    return integration_results[result_id]

if __name__ == "__main__":
    import uvicorn
    
    host = os.getenv("HOST", "0.0.0.0")
    port = int(os.getenv("PORT", 8091))
    log_level = os.getenv("LOG_LEVEL", "info")
    
    uvicorn.run("app.main:app", host=host, port=port, log_level=log_level, reload=True)
