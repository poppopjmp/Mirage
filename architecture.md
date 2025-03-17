# Mirage Architecture

## High-Level Architecture

Mirage is built as a cloud-native application following microservices architecture principles. The system is composed of loosely coupled services that communicate primarily through asynchronous messaging, with synchronous REST APIs where appropriate.

```
                                  ┌─────────────────┐
                                  │                 │
                                  │  API Gateway    │
                                  │                 │
                                  └────────┬────────┘
                                           │
                                           ▼
┌─────────────┐  ┌─────────────┐  ┌─────────────────┐  ┌─────────────┐  ┌────────────────┐
│             │  │             │  │                 │  │             │  │                │
│ Auth        │  │ User        │  │ Scan            │  │ Module      │  │ Notification   │
│ Service     │  │ Management  │  │ Orchestration   │  │ Registry    │  │ Service        │
│             │  │ Service     │  │ Service         │  │ Service     │  │                │
└─────────────┘  └─────────────┘  └────────┬────────┘  └─────────────┘  └────────────────┘
                                           │
                                           ▼
┌─────────────┐  ┌─────────────┐  ┌─────────────────┐  ┌─────────────┐  ┌────────────────┐
│             │  │             │  │                 │  │             │  │                │
│ Data        │  │ Data        │  │ Correlation     │  │ Visualization│  │ Reporting      │
│ Collection  │  │ Storage     │  │ Engine          │  │ Service     │  │ Service        │
│ Service     │  │ Service     │  │ Service         │  │             │  │                │
└─────────────┘  └─────────────┘  └─────────────────┘  └─────────────┘  └────────────────┘
```

## Service Responsibilities

### API Gateway
- Single entry point for all client requests (Rust/Actix-web)
- Request routing to appropriate microservices
- Authentication verification
- Rate limiting
- Request/response transformation

### Auth Service
- User authentication (username/password, OAuth, SSO) (Rust/Actix-web)
- JWT token generation and validation
- Authorization policies
- Session management

### User Management Service
- User profile creation and management (Rust/Rocket)
- Role-based access control
- Team and organization management
- User preferences

### Scan Orchestration Service
- Scan definition and configuration (Rust/Actix-web)
- Scan execution planning
- Job scheduling and distribution
- Scan lifecycle management (start, stop, pause, resume)
- Progress tracking

### Module Registry Service
- Module registration and discovery (Rust/Actix-web)
- Module configuration management
- Module dependency resolution
- Module version control
- Module analytics

### Data Collection Service
- Execution of data collection modules (Rust/Actix-web)
- Rate limiting for external API calls
- Retry logic for failed requests
- Result normalization
- Collection progress tracking

### Data Storage Service
- Persistent storage of all collected intelligence (Rust/Actix-web)
- Data versioning
- Query interface for other services
- Data retention policies
- Backup and recovery

### Correlation Engine Service
- Entity relationship analysis (Rust/Actix-web)
- Pattern identification
- Data enrichment
- Confidence scoring
- Knowledge graph management

### Visualization Service
- Network graph generation (Rust/Actix-web)
- Interactive visualization components
- Visual filtering and search
- Timeline generation
- Export capabilities

### Reporting Service
- Report template management (Rust/Actix-web)
- Custom report generation
- Scheduled reporting
- Export in multiple formats (PDF, CSV, JSON)
- Report sharing

### Notification Service
- Alert generation based on configurable triggers (Rust/Actix-web)
- Notification delivery (email, SMS, webhooks)
- Notification preferences
- Alert aggregation and deduplication

### Integration Service
- Third-party tool integration (Rust/Actix-web)
- API connectors for external systems
- Data import/export capabilities
- Webhook management

### Configuration Service
- Centralized configuration management (Go/Gin)
- Dynamic configuration updates
- Environment-specific settings
- Feature flags

### Discovery Service
- Service registration (Rust/Actix-web)
- Service health monitoring
- Load balancing
- Service-to-service discovery

## Communication Patterns

- **Synchronous Communication**: REST APIs for direct request/response patterns
- **Asynchronous Communication**: Message queues for event-driven processes
- **Service Mesh**: For managing service-to-service communication with:
  - Circuit breaking
  - Retries
  - Timeouts
  - Load balancing
  - Observability

## Data Management

Each service maintains its own database when needed, following the database-per-service pattern. The Data Storage Service acts as the central repository for all collected intelligence data, while other services maintain databases specific to their domain.

## Deployment Model

Mirage is designed to be deployed in containerized environments using:
- Docker for containerization
- Kubernetes for orchestration
- Helm charts for deployment management
- CI/CD pipelines for automated testing and deployment

## Security Considerations

- All inter-service communication is encrypted
- Least privilege access for all services
- Regular security scanning for vulnerabilities
- Secrets management through secure vaults
- Comprehensive audit logging
