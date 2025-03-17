# Mirage Microservices

This directory contains documentation for each microservice in the Mirage platform.

## Service Overview

| Service Name | Description | Language/Framework | Repository |
|-------------|-------------|-------------------|------------|
| API Gateway | Entry point for all client requests | Rust/Actix-web | [mirage-api-gateway](https://github.com/mirage/mirage-api-gateway) |
| Auth Service | Authentication and authorization | Rust/Actix-web | [mirage-auth-service](https://github.com/mirage/mirage-auth-service) |
| User Management | User and organization management | Rust/Rocket | [mirage-user-service](https://github.com/mirage/mirage-user-service) |
| Scan Orchestration | Manages scan lifecycle | Rust/Actix-web | [mirage-scan-orchestration](https://github.com/mirage/mirage-scan-orchestration) |
| Module Registry | Module management | Rust/Actix-web | [mirage-module-registry](https://github.com/mirage/mirage-module-registry) |
| Data Collection | Collects data from various sources | Rust/Actix-web | [mirage-data-collection](https://github.com/mirage/mirage-data-collection) |
| Data Storage | Stores collected data | Rust/Actix-web | [mirage-data-storage](https://github.com/mirage/mirage-data-storage) |
| Correlation Engine | Correlates and analyzes data | Rust/Actix-web | [mirage-correlation-engine](https://github.com/mirage/mirage-correlation-engine) |
| Visualization | Visualizes data | Rust/Actix-web | [mirage-visualization](https://github.com/mirage/mirage-visualization) |
| Reporting | Generates reports | Rust/Actix-web | [mirage-reporting](https://github.com/mirage/mirage-reporting) |
| Notification | Sends notifications | Rust/Actix-web | [mirage-notification](https://github.com/mirage/mirage-notification) |
| Integration | Integrates with third-party tools | Rust/Actix-web | [mirage-integration](https://github.com/mirage/mirage-integration) |
| Configuration | Manages configuration | Go/Gin | [mirage-configuration](https://github.com/mirage/mirage-configuration) |
| Discovery | Service discovery | Rust/Actix-web | [mirage-discovery](https://github.com/mirage/mirage-discovery) |
| Web UI | User interface | TypeScript/React | [mirage-web-ui](https://github.com/mirage/mirage-web-ui) |

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
