# Mirage Microservices

This directory contains documentation for each microservice in the Mirage platform.

## Service Overview

| Service Name | Description | Language/Framework | Repository |
|-------------|-------------|-------------------|------------|
| API Gateway | Entry point for all client requests | Go/Chi | [mirage-api-gateway](https://github.com/mirage/mirage-api-gateway) |
| Auth Service | Authentication and authorization | Go/Echo | [mirage-auth-service](https://github.com/mirage/mirage-auth-service) |
| User Management | User and organization management | Go/Fiber | [mirage-user-service](https://github.com/mirage/mirage-user-service) |
| Scan Orchestration | Manages scan lifecycle | Python/FastAPI | [mirage-scan-orchestration](https://github.com/mirage/mirage-scan-orchestration) |
| Module Registry | Module management | Python/Flask | [mirage-module-registry](https://github.com/mirage/mirage-module-registry) |
| Data Collection | Intelligence gathering | Python/FastAPI | [mirage-data-collection](https://github.com/mirage/mirage-data-collection) |
| Data Storage | Central data repository | Go/Gin | [mirage-data-storage](https://github.com/mirage/mirage-data-storage) |
| Correlation Engine | Data relationship analysis | Python/FastAPI | [mirage-correlation-engine](https://github.com/mirage/mirage-correlation-engine) |
| Visualization | Data visualization | Node.js/Express | [mirage-visualization](https://github.com/mirage/mirage-visualization) |
| Reporting | Report generation | Node.js/Express | [mirage-reporting](https://github.com/mirage/mirage-reporting) |
| Notification | Alert management | Go/Echo | [mirage-notification](https://github.com/mirage/mirage-notification) |
| Integration | Third-party integrations | Python/FastAPI | [mirage-integration](https://github.com/mirage/mirage-integration) |
| Configuration | Configuration management | Go/Gin | [mirage-configuration](https://github.com/mirage/mirage-configuration) |
| Discovery | Service discovery | Go/Gin | [mirage-discovery](https://github.com/mirage/mirage-discovery) |

## Service Architecture Principles

1. **Single Responsibility**: Each service focuses on a specific business capability
2. **Independent Deployment**: Services can be deployed independently
3. **Decentralized Data Management**: Each service manages its own data
4. **Resilience**: Services are designed to handle failures gracefully
5. **Observability**: Comprehensive logging, metrics, and tracing
6. **Scalability**: Services can scale independently based on demand

## Common Components

Each service repository follows a similar structure:
- `cmd/`: Main application entry points
- `internal/`: Private application code
- `pkg/`: Public packages that can be imported by other services
- `api/`: API definitions (OpenAPI/Swagger, gRPC)
- `config/`: Configuration files
- `scripts/`: Utility scripts
- `Dockerfile`: Container definition
- `docker-compose.yml`: Local development setup

See each service's documentation for details specific to that service.
