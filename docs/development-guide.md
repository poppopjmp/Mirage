# Mirage Development Guide

## Development Environment Setup

### Prerequisites

- Docker and Docker Compose
- Git
- Python 3.11+
- Go 1.21+
- Node.js 18+
- PostgreSQL client tools
- MongoDB client tools
- Kubernetes tools (kubectl, minikube, etc.)
- IDE with appropriate plugins (VSCode, IntelliJ, etc.)

### Initial Setup

1. Clone the main repository and submodules:
```bash
git clone https://github.com/mirage/mirage.git
cd mirage
git submodule update --init --recursive
```

2. Set up local environment variables:
```bash
cp .env.example .env
# Edit .env with your local settings
```

3. Start the development environment:
```bash
docker-compose -f docker-compose.dev.yml up -d
```

4. Access the development dashboard:
```
http://localhost:8080
```

## Development Workflow

### Branching Strategy

We follow a GitFlow-inspired branching strategy:

- `main`: Production-ready code
- `develop`: Integration branch for features
- `feature/*`: New features
- `bugfix/*`: Bug fixes
- `release/*`: Release preparation
- `hotfix/*`: Production hotfixes

### Pull Request Process

1. Create a feature/bugfix branch from `develop`
2. Make your changes and commit with meaningful messages
3. Push your branch and create a PR against `develop`
4. Ensure CI passes and request reviews
5. Address review comments
6. Once approved, merge into `develop`

### Coding Standards

#### General

- Write clear, concise, and meaningful comments
- Include documentation for public APIs
- Write tests for new features and bug fixes
- Follow the SOLID principles

#### Python

- Follow PEP 8 style guide
- Use type hints
- Use pytest for testing
- Use Black for code formatting
- Use isort for import sorting
- Use flake8 for linting

#### Go

- Follow Go standard formatting (gofmt)
- Use golangci-lint for linting
- Write tests using the standard testing package
- Follow idiomatic Go practices

#### JavaScript/TypeScript

- Use ESLint for linting
- Use Prettier for formatting
- Follow TypeScript best practices
- Write tests using Jest

### Testing

We maintain several test layers:

1. **Unit Tests**: Test individual components in isolation
2. **Integration Tests**: Test component interactions
3. **System Tests**: Test the system as a whole
4. **Performance Tests**: Test system performance under load
5. **Security Tests**: Test system security

All tests must pass before code can be merged.

## Microservices Development

### Adding a New Service

1. Create a new repository from the service template:
```bash
./scripts/create-service.sh my-new-service
```

2. Define the service API using OpenAPI/Swagger
3. Implement the service following the architecture guidelines
4. Add service to the main docker-compose.yml file
5. Register service in the service discovery system

### Service Communication

- Use REST for synchronous communication
- Use message queues for asynchronous communication
- Document all APIs and message formats
- Handle failures gracefully (circuit breaking, retries, etc.)

### Handling Dependencies

- Use environment variables for service discovery
- Use the Configuration Service for dynamic settings
- Design for resilience to dependent service failures
- Use feature toggles for incomplete features

## CI/CD Pipeline

Our CI/CD pipeline includes:

1. **Build**: Compile code and build containers
2. **Test**: Run unit and integration tests
3. **Analyze**: Static code analysis and security scanning
4. **Package**: Create deployable artifacts
5. **Deploy**: Deploy to staging environment
6. **Validate**: Run system and acceptance tests
7. **Release**: Deploy to production

## Troubleshooting

### Common Issues

- **Service discovery failures**: Ensure the service is registered correctly
- **Database connection issues**: Check credentials and network connectivity
- **Authentication failures**: Verify JWT configuration and token validity
- **Performance problems**: Check resource usage and scaling settings

### Debugging Tools

- Use Kibana for log analysis
- Use Prometheus and Grafana for metrics
- Use Jaeger for distributed tracing
- Use Kubernetes Dashboard for cluster monitoring

## Security Considerations

- Never commit secrets to the repository
- Use environment variables or secure vaults for sensitive information
- Follow the principle of least privilege
- Validate and sanitize all user inputs
- Use prepared statements for database queries
- Enable CORS only for trusted domains
- Implement rate limiting for APIs

## New Microservices

### Auth Service

- **Technology**: Rust + Actix-Web
- **Features**:
  - JWT token generation and validation
  - OAuth/OIDC integration
  - Rate limiting middleware
- **Testing**: Unit tests for authentication endpoints

### User Management Service

- **Technology**: Rust + Rocket
- **Features**:
  - User CRUD operations
  - Role and permission management
  - Team management functionality
- **Testing**: Unit tests for user management endpoints

### Scan Orchestration Service

- **Technology**: Rust + Tokio
- **Features**:
  - Scan lifecycle management logic
  - Job scheduling and distribution logic
  - Scan progress tracking
- **Testing**: Unit tests for scan orchestration endpoints

### Module Registry Service

- **Technology**: Rust + Actix-Web
- **Features**:
  - Module registration and discovery logic
  - Module configuration management
  - Module dependency resolution
- **Testing**: Unit tests for module registry endpoints

### Data Collection Service

- **Technology**: Rust + Hyper
- **Features**:
  - Data collection module execution logic
  - Rate limiting for external API calls
  - Result normalization logic
- **Testing**: Unit tests for data collection endpoints

### Data Storage Service

- **Technology**: Rust + Actix-Web
- **Features**:
  - Persistent storage logic
  - Data versioning logic
  - Query interface for other services
- **Testing**: Unit tests for data storage endpoints

### Correlation Engine Service

- **Technology**: Rust + rayon
- **Features**:
  - Entity relationship analysis logic
  - Pattern identification logic
  - Data enrichment logic
- **Testing**: Unit tests for correlation engine endpoints

### Visualization Service

- **Technology**: Rust + WebAssembly
- **Features**:
  - Network graph generation logic
  - Interactive visualization components
  - Visual filtering and search logic
- **Testing**: Unit tests for visualization endpoints

### Reporting Service

- **Technology**: Rust + Actix-Web
- **Features**:
  - Report template management logic
  - Custom report generation logic
  - Scheduled reporting logic
- **Testing**: Unit tests for reporting endpoints

### Notification Service

- **Technology**: Rust + Actix-Web
- **Features**:
  - Alert generation logic
  - Notification delivery logic
  - Notification preferences management
- **Testing**: Unit tests for notification endpoints

### Integration Service

- **Technology**: Rust + Actix-Web
- **Features**:
  - Third-party tool integration logic
  - API connectors for external systems
  - Data import/export capabilities
- **Testing**: Unit tests for integration endpoints

### Configuration Service

- **Technology**: Rust + Actix-Web
- **Features**:
  - Centralized configuration management logic
  - Dynamic configuration updates logic
  - Environment-specific settings logic
- **Testing**: Unit tests for configuration endpoints

### Discovery Service

- **Technology**: Rust + Actix-Web
- **Features**:
  - Service registration logic
  - Service health monitoring logic
  - Load balancing logic
- **Testing**: Unit tests for discovery endpoints

## Running Unit Tests and Integration Tests

To run unit tests and integration tests for each microservice, follow these steps:

1. Navigate to the service directory:
   ```bash
   cd services/<service-name>
   ```

2. Run the tests using Cargo:
   ```bash
   cargo test
   ```

## Building and Deploying Docker Containers

To build and deploy Docker containers for each microservice, follow these steps:

1. Navigate to the service directory:
   ```bash
   cd services/<service-name>
   ```

2. Build the Docker image:
   ```bash
   docker build -t mirage/<service-name>:latest .
   ```

3. Run the Docker container:
   ```bash
   docker run -d -p <host-port>:<container-port> mirage/<service-name>:latest
   ```

## CI/CD Pipeline

Our CI/CD pipeline includes:

1. **Build**: Compile code and build containers
2. **Test**: Run unit and integration tests
3. **Analyze**: Static code analysis and security scanning
4. **Package**: Create deployable artifacts
5. **Deploy**: Deploy to staging environment
6. **Validate**: Run system and acceptance tests
7. **Release**: Deploy to production
