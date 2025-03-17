# Mirage Docker Service Definitions

This directory contains Docker definitions for all Mirage microservices. Each service has its own directory with:

- Dockerfile - The container definition for the service
- docker-compose.yml - Service-specific compose overrides for local development

## Service Language Overview

| Service | Language | Framework |
|---------|----------|-----------|
| api-gateway | Rust | Actix-web |
| auth-service | Rust | Actix-web |
| user-management-service | Rust | Rocket |
| scan-orchestration-service | Rust | Actix-web |
| module-registry-service | Rust | Actix-web |
| data-collection-service | Rust | Actix-web |
| data-storage-service | Rust | Actix-web |
| correlation-engine-service | Rust | Actix-web |
| visualization-service | Rust | Actix-web |
| reporting-service | Rust | Actix-web |
| notification-service | Rust | Actix-web |
| integration-service | Python | FastAPI |
| configuration-service | Go | Gin |
| discovery-service | Go | Gin |
| web-ui | TypeScript | React |

## Building and Running

To build all service containers:

```bash
cd ../..  # Back to repository root
docker-compose build
```

To build a specific service:

```bash
cd ../..  # Back to repository root
docker-compose build <service-name>
```
