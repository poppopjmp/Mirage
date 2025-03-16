# Scan Orchestration Service

## Overview

The Scan Orchestration Service is responsible for managing the lifecycle of OSINT scans in the Mirage platform. It handles scan creation, configuration, execution planning, job scheduling, and progress tracking.

## Features

- Scan definition and configuration management
- Execution planning based on available modules and targets
- Job scheduling and distribution to Data Collection Service
- Scan lifecycle management (start, stop, pause, resume)
- Scan progress monitoring and reporting
- Scan history and audit logs
- Resource utilization control and throttling
- Scan templates for common investigation patterns
- Scan chaining and dependency management

## Technical Details

### Technology Stack

- **Language**: Python 3.11+
- **Framework**: FastAPI
- **Database**: PostgreSQL
- **Message Queue**: RabbitMQ
- **Cache**: Redis
- **Containerization**: Docker
- **Orchestration**: Kubernetes

### API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/scans` | GET | List all scans |
| `/scans` | POST | Create a new scan |
| `/scans/{id}` | GET | Get scan details |
| `/scans/{id}` | PUT | Update scan configuration |
| `/scans/{id}` | DELETE | Delete a scan |
| `/scans/{id}/start` | POST | Start a scan |
| `/scans/{id}/stop` | POST | Stop a scan |
| `/scans/{id}/pause` | POST | Pause a scan |
| `/scans/{id}/resume` | POST | Resume a scan |
| `/scans/{id}/status` | GET | Get scan status |
| `/scans/{id}/results` | GET | Get scan results |
| `/templates` | GET | List scan templates |
| `/templates` | POST | Create a scan template |
| `/templates/{id}` | GET | Get template details |

### Data Model

#### Scan
```json
{
  "id": "uuid",
  "name": "string",
  "description": "string",
  "user_id": "uuid",
  "organization_id": "uuid",
  "status": "enum(CREATED, RUNNING, PAUSED, COMPLETED, FAILED, CANCELLED)",
  "progress": "float",
  "target": {
    "type": "enum(DOMAIN, IP, EMAIL, ...)",
    "value": "string"
  },
  "modules": [
    {
      "id": "string",
      "enabled": "boolean",
      "config": "object"
    }
  ],
  "created_at": "datetime",
  "updated_at": "datetime",
  "started_at": "datetime",
  "completed_at": "datetime",
  "configuration": {
    "max_duration": "integer",
    "max_results": "integer",
    "priority": "enum(LOW, MEDIUM, HIGH)",
    "recursion_depth": "integer"
  }
}
```

### Service Interactions

- **Module Registry Service**: Gets available modules and their requirements
- **Data Collection Service**: Delegates actual data collection tasks
- **Data Storage Service**: Stores scan results
- **User Management Service**: Validates user permissions
- **Notification Service**: Sends scan status updates

## Deployment

The service is deployed as a Kubernetes StatefulSet with the following components:
- Scan Orchestration API
- Scheduler
- Progress Monitor
- Worker Pool Manager

### Resource Requirements

| Component | CPU | Memory | Storage |
|-----------|-----|--------|---------|
| API | 0.5-1 CPU | 512MB | - |
| Scheduler | 0.5-1 CPU | 512MB | - |
| Monitor | 0.25 CPU | 256MB | - |
| Database | 1-2 CPU | 2-4GB | 20GB+ |

## Development Guide

### Setup Local Environment

```bash
# Clone the repository
git clone https://github.com/mirage/mirage-scan-orchestration.git
cd mirage-scan-orchestration

# Set up virtual environment
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate

# Install dependencies
pip install -r requirements.txt
pip install -r requirements-dev.txt

# Run database migrations
alembic upgrade head

# Start the service
uvicorn app.main:app --reload
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
