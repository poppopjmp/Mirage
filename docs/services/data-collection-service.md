# Data Collection Service

## Overview

The Data Collection Service is responsible for executing OSINT data collection modules against specified targets. It acts as the core intelligence gathering component of the Mirage platform, interacting with hundreds of external data sources and APIs.

## Features

- Execution of data collection modules against target entities
- Intelligent rate limiting and throttling for external API calls
- Proxy management for distributed collection
- Automatic handling of authentication for various data sources
- Result normalization and initial processing
- Caching of common requests for performance optimization
- Error handling and automatic retry policies
- Parallel execution of compatible modules
- Detailed collection logs for auditing and debugging

## Technical Details

### Technology Stack

- **Language**: Python 3.11+
- **Framework**: FastAPI
- **Database**: MongoDB (module results), Redis (caching)
- **Message Queue**: RabbitMQ
- **Containerization**: Docker
- **Orchestration**: Kubernetes

### API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/collect` | POST | Execute a collection task |
| `/collect/{job_id}` | GET | Get collection job status |
| `/collect/{job_id}` | DELETE | Cancel a collection job |
| `/sources` | GET | List available data sources |
| `/sources/{id}` | GET | Get details about a specific source |
| `/health` | GET | Service health check |
| `/metrics` | GET | Prometheus metrics |

### Data Models

#### Collection Task
```json
{
  "id": "uuid",
  "scan_id": "uuid",
  "module_id": "string",
  "target": {
    "type": "enum(DOMAIN, IP, EMAIL, ...)",
    "value": "string"
  },
  "status": "enum(QUEUED, RUNNING, COMPLETED, FAILED, CANCELLED)",
  "config": {
    "timeout": "integer",
    "max_results": "integer",
    "use_proxy": "boolean",
    "api_key": "string"
  },
  "created_at": "datetime",
  "started_at": "datetime",
  "completed_at": "datetime",
  "result_count": "integer",
  "error": "string"
}
```

#### Collection Result
```json
{
  "id": "uuid",
  "task_id": "uuid",
  "data_type": "enum(DOMAIN, IP, EMAIL, URL, ...)",
  "data": "string",
  "source_data": "object",
  "confidence": "float",
  "collected_at": "datetime",
  "metadata": "object",
  "hash": "string"
}
```

### Service Interactions

- **Scan Orchestration Service**: Receives collection tasks from scan jobs
- **Module Registry Service**: Gets module implementation details and requirements
- **Data Storage Service**: Sends normalized collection results for storage
- **Notification Service**: Sends notifications about important collection events
- **Configuration Service**: Gets API keys and service configurations

## Deployment

The service is deployed as a Kubernetes Deployment with the following components:
- API Server
- Task Queue Workers (auto-scaling)
- Result Processor
- Rate Limiter

### Resource Requirements

| Component | CPU | Memory | Storage |
|-----------|-----|--------|---------|
| API Server | 0.5-1 CPU | 512MB-1GB | - |
| Workers | 0.5 CPU per worker | 512MB per worker | - |
| Result Processor | 0.5 CPU | 512MB | - |
| MongoDB | 2-4 CPU | 4-8GB | 50GB+ |
| Redis | 0.5-1 CPU | 1-2GB | 10GB |

## Development Guide

### Setup Local Environment

```bash
# Clone the repository
git clone https://github.com/mirage/mirage-data-collection.git
cd mirage-data-collection

# Set up virtual environment
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate

# Install dependencies
pip install -r requirements.txt
pip install -r requirements-dev.txt

# Start required services
docker-compose up -d mongodb redis rabbitmq

# Start the service
uvicorn app.main:app --reload
```

### Creating a New Module

```python
from app.core.module import BaseModule
from app.models.result import CollectionResult
from app.models.task import CollectionTask

class ExampleModule(BaseModule):
    """Example data collection module."""
    
    name = "example_module"
    description = "Collects data from Example API"
    version = "1.0.0"
    author = "Mirage Team"
    
    # Define data types this module can process
    input_types = ["DOMAIN", "IP"]
    
    # Define data types this module can produce
    output_types = ["URL", "EMAIL"]
    
    # Define configuration options
    default_config = {
        "api_key": None,
        "timeout": 30,
        "max_results": 100
    }
    
    async def setup(self):
        """Initialize module resources."""
        self.api_key = self.config.get("api_key") or await self.get_api_key()
        self.client = self.create_client()
    
    async def collect(self, task: CollectionTask) -> list[CollectionResult]:
        """Execute data collection."""
        target = task.target.value
        target_type = task.target.type
        
        self.logger.info(f"Collecting data for {target_type}:{target}")
        
        # Implement collection logic
        raw_results = await self.client.search(target)
        
        # Process and normalize results
        results = []
        for item in raw_results:
            result = CollectionResult(
                task_id=task.id,
                data_type="URL" if "http" in item["link"] else "EMAIL",
                data=item["link"],
                source_data=item,
                confidence=0.9,
                metadata={"found_in": item.get("context")}
            )
            results.append(result)
        
        return results
    
    async def cleanup(self):
        """Clean up resources."""
        await self.client.close()
```

### Running Tests

```bash
# Run unit tests
pytest tests/unit

# Run integration tests
pytest tests/integration

# Run with coverage
pytest --cov=app tests/
```
