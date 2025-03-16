# Mirage Architecture

This document outlines the high-level architecture of the Mirage OSINT platform.

## System Architecture Overview

Mirage is built on a cloud-native microservices architecture designed to provide scalability, resilience, and flexibility. The system is composed of multiple loosely-coupled services that communicate primarily through gRPC and event-driven messaging.

![Mirage Architecture Diagram](assets/architecture_diagram.png)

## Service Descriptions

### Auth Service
Handles user authentication, authorization, and manages access control across the platform.

### User Management Service
Manages user profiles, teams, roles, and permissions.

### Scan Orchestration Service
Coordinates OSINT scans, manages their lifecycle, and delegates tasks to the Data Collection Service.

### Module Registry Service
Maintains a registry of available OSINT modules, their capabilities, configurations, and dependencies.

### Data Collection Service
Executes OSINT modules to gather intelligence from various sources.

### Data Storage Service
Handles the persistence of collected data, scan results, and system metadata.

### Correlation Engine Service
Analyzes collected data to identify relationships, patterns, and insights.

### Visualization Service
Generates interactive visualizations of OSINT data and relationships.

### Reporting Service
Creates customized reports based on scan results and analysis.

### Notification Service
Manages alerts, notifications, and real-time updates to users.

### API Gateway
Serves as the entry point for all client requests, handling routing, authentication, and rate limiting.

### Integration Service
Enables integration with external systems and platforms.

### Configuration Service
Manages service configurations and feature flags.

### Discovery Service
Provides service discovery and load balancing capabilities.

## Communication Patterns

### Service-to-Service Communication
- **gRPC**: Used for synchronous communication between services
- **Kafka**: Used for asynchronous, event-driven communication
- **WebSockets**: Used for real-time updates to clients

### Data Flow

1. Client requests are received by the API Gateway
2. API Gateway authenticates requests through the Auth Service
3. Requests are routed to the appropriate service
4. Services process requests, potentially communicating with other services
5. Results are returned to the client through the API Gateway
6. Asynchronous events are published to Kafka for consumption by interested services

## Data Storage

Different services use specialized databases optimized for their specific needs:

- **PostgreSQL**: Used for structured data like user profiles, teams, and permissions
- **MongoDB**: Used for flexible schema data like scan results and entity information
- **Elasticsearch**: Used for fast text search and analytics
- **Redis**: Used for caching, session storage, and ephemeral data
- **Neo4j**: Used for graph-based relationship data between entities

## Deployment Architecture

Mirage is designed to be deployed on Kubernetes, with each service packaged as a container. The system supports:

- Horizontal scaling of individual services based on demand
- Blue/green deployments for zero-downtime updates
- Auto-recovery from failures
- Geographic distribution for high availability

## Security Architecture

Security is implemented at multiple levels:

- **Network**: Service isolation, encryption, and firewall rules
- **Authentication**: JWT-based authentication with OAuth/OIDC support
- **Authorization**: Role-based access control (RBAC)
- **Data**: Encryption at rest and in transit
- **API**: Rate limiting, input validation, and output encoding

## Monitoring and Observability

The system includes comprehensive monitoring:

- **Metrics**: Prometheus for collecting service metrics
- **Logging**: ELK stack for centralized log management
- **Tracing**: Jaeger for distributed tracing
- **Dashboards**: Grafana for visualization and alerting
