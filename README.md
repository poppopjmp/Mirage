# Mirage

Mirage is a next-generation OSINT platform based on SpiderFoot, rebuilt as a cloud-native microservices architecture.

## Overview

Mirage provides advanced open-source intelligence gathering, correlation, and analysis capabilities through a modern, scalable infrastructure. It extends SpiderFoot's capabilities with real-time monitoring, enhanced collaboration features, and advanced analytics.

## Key Features

- OSINT data collection from 200+ sources
- Extensible module system for intelligence gathering
- Advanced scan management and orchestration
- Automated data correlation and analysis
- Interactive visualization of relationships
- Team collaboration and shared investigations
- Real-time monitoring and alerts
- ML-based pattern recognition
- Custom automated workflows
- Multi-tenant architecture

## Architecture

Mirage follows a microservices architecture with the following key components:

- Auth Service
- User Management Service
- Scan Orchestration Service
- Module Registry Service
- Data Collection Service
- Data Storage Service
- Correlation Engine Service
- Visualization Service
- Reporting Service
- Notification Service
- API Gateway
- Integration Service
- Configuration Service
- Discovery Service

See [architecture.md](architecture.md) for detailed architecture information.

## Core Functionalities

### Event Handling

The core module provides detailed implementations for event handling. Events are represented by the `Event` struct, which includes fields for event type, data, source, and timestamp. The `EventHandler` struct manages events, allowing you to add, retrieve, and list events.

Example usage:

```rust
use core::event::{Event, EventHandler};

let mut handler = EventHandler::new();
let event = Event::new("test_type", "test_data", Some("test_source"), 1234567890);
handler.add_event(event);

let events = handler.get_events("test_type").unwrap();
println!("{:?}", events);
```

### Target Management

The core module also includes target management functionalities. Targets are represented by the `Target` struct, which includes fields for id, name, and description. The `TargetManager` struct manages targets, allowing you to add, retrieve, remove, and list targets.

Example usage:

```rust
use core::target::{Target, TargetManager};

let mut manager = TargetManager::new();
let target = Target::new("1", "Test Target", "This is a test target.");
manager.add_target(target);

let retrieved_target = manager.get_target("1").unwrap();
println!("{:?}", retrieved_target);
```

### Helper Functions

The core module provides helper functions for validating email addresses, URLs, and IP addresses.

Example usage:

```rust
use core::helpers::{is_valid_email, is_valid_url, is_valid_ip};

assert!(is_valid_email("test@example.com"));
assert!(is_valid_url("http://example.com"));
assert!(is_valid_ip("192.168.0.1"));
```

## Correlation Functionalities

The correlations module provides detailed implementations for correlation rules. Each rule is defined in a separate file and includes logic for identifying relationships and patterns in the collected data.

### Cloud Bucket Open Rule

The `cloud_bucket_open.rs` file implements the correlation rule for detecting open cloud buckets. It includes logic for identifying publicly accessible cloud storage buckets and generating alerts.

### DNS Zone Transfer Rule

The `dns_zone_transfer.rs` file implements the correlation rule for detecting DNS zone transfers. It includes logic for identifying unauthorized DNS zone transfers and generating alerts.

### Email Breach Rule

The `email_breach.rs` file implements the correlation rule for detecting email breaches. It includes logic for identifying compromised email addresses and generating alerts.

## Getting Started

Documentation for setup, configuration, and development is available in the [docs](docs) directory.

### 1. **Define the Scope and Requirements**
   - **Identify Features**: List all the features of SpiderFoot that you want to include in Mirage. Determine if there are any new features you want to add.
   - **Microservices Architecture**: Decide how you want to break down the application into microservices. Each service should have a single responsibility.

### 2. **Design the Architecture**
   - **Service Communication**: 
     - Primary: gRPC for high-performance internal service communication
     - Event-driven: Kafka for asynchronous workflows and event propagation
     - WebSockets for real-time client updates and notifications
   - **Data Storage**: 
     - PostgreSQL: Primary relational data store for structured data and user management
     - MongoDB: Document store for flexible schema data and scan results
     - Elasticsearch: For efficient full-text search and analytics across OSINT data
     - Redis: For caching, rate limiting, and ephemeral data storage
   - **API Gateway**: Implement an API gateway using Envoy with custom Rust extensions for:
     - Request routing and load balancing
     - Authentication and authorization
     - Rate limiting and circuit breaking
     - Request/response transformation
     - Monitoring and observability integration

### 3. **Choose Rust Frameworks and Libraries**
   - **Web Framework**: Consider using frameworks Rocket for building your web services.
   - **Database Interaction**: Use libraries like  SQLx for database interactions.
   - **Asynchronous Programming**: Leverage Rust's async capabilities with libraries like Tokio or async-std for handling concurrent requests.

### 4. **Set Up Your Development Environment**
   - **Version Control**: Use Git for version control and consider hosting your repository on platforms like GitHub or GitLab.
   - **Build System**: Use Cargo, Rust's package manager and build system, to manage dependencies and build your project.

### 5. **Implement Microservices**
   - **Service Development**: Start developing each microservice based on the defined architecture. Ensure that each service is independently deployable.
   - **Testing**: Write unit tests and integration tests for each service to ensure reliability.

### 6. **Deployment Strategy**
   - **Containerization**: Consider using Docker to containerize your microservices for easier deployment and scaling.
   - **Orchestration**: Use Kubernetes or Docker Compose for managing your containers and services.

### 7. **Monitoring and Logging**
   - Implement logging and monitoring for your microservices to track performance and errors. Consider using tools like Prometheus and Grafana for monitoring.

### 8. **Documentation**
   - Document your code and architecture thoroughly. Consider using tools like Swagger/OpenAPI for API documentation.

### 9. **Community and Contributions**
   - If you plan to open-source Mirage, create a CONTRIBUTING.md file to guide potential contributors on how to get involved.

### 10. **Iterate and Improve**
   - After the initial release, gather feedback and continuously improve the application based on user needs and performance metrics.

### Example Microservices Breakdown
- **Data Collection Service**: Responsible for gathering data from various sources.
- **Analysis Service**: Analyzes the collected data and generates insights.
- **User Management Service**: Handles user authentication and management.
- **Reporting Service**: Generates reports based on the analyzed data.

## Deployment and Testing

### Deploying Microservices

Each microservice in Mirage is containerized using Docker. To deploy a microservice, follow these steps:

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

### Testing Microservices

Each microservice includes unit tests and integration tests to ensure reliability. To run the tests, follow these steps:

1. Navigate to the service directory:
   ```bash
   cd services/<service-name>
   ```

2. Run the tests using Cargo:
   ```bash
   cargo test
   ```

## Detailed Documentation

For detailed documentation on each microservice, including API specifications, configuration options, and deployment instructions, refer to the following links:

- [Auth Service](docs/services/auth-service.md)
- [User Management Service](docs/services/user-management-service.md)
- [Scan Orchestration Service](docs/services/scan-orchestration-service.md)
- [Module Registry Service](docs/services/module-registry-service.md)
- [Data Collection Service](docs/services/data-collection-service.md)
- [Data Storage Service](docs/services/data-storage-service.md)
- [Correlation Engine Service](docs/services/correlation-engine-service.md)
- [Visualization Service](docs/services/visualization-service.md)
- [Reporting Service](docs/services/reporting-service.md)
- [Notification Service](docs/services/notification-service.md)
- [API Gateway](docs/services/api-gateway.md)
- [Integration Service](docs/services/integration-service.md)
- [Configuration Service](docs/services/configuration-service.md)
- [Discovery Service](docs/services/discovery-service.md)

## Unit Tests and Integration Tests

Each microservice includes unit tests and integration tests to ensure reliability. To run the tests, follow these steps:

1. Navigate to the service directory:
   ```bash
   cd services/<service-name>
   ```

2. Run the tests using Cargo:
   ```bash
   cargo test
   ```

## Dockerfiles and Containerization

Each microservice in Mirage is containerized using Docker. Dockerfiles for all services are present, ensuring each service is containerized and deployable. To build and run a Docker container for a service, follow these steps:

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

For more information on Dockerfiles and containerization, refer to the [Docker documentation](https://docs.docker.com/).

