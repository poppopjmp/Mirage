# Mirage OSINT Platform Development Guide

## Project Overview

Mirage is an open-source OSINT (Open Source Intelligence) platform designed for information gathering, analysis, and visualization. The platform is built using a microservices architecture with the following key components:

- **Auth Service** - Handles authentication and authorization
- **User Management** - Manages user accounts and permissions
- **Module Registry** - Manages OSINT modules and plugins
- **Scan Orchestration** - Coordinates scanning tasks across modules
- **Data Collection** - Collects data from various sources
- **Data Storage** - Stores and indexes collected data
- **Correlation Engine** - Analyzes relationships between data points
- **Visualization** - Provides visualizations of data and relationships
- **Reporting** - Generates reports from collected data
- **Notification** - Handles alerts and notifications

## Development Environment Setup

### Prerequisites

- Rust 1.67+ (https://rustup.rs/)
- Docker and Docker Compose
- Node.js 18+ (for frontend development)
- PostgreSQL 14+
- MongoDB 6.0+
- Neo4j 5.0+
- Redis 7.0+
- Elasticsearch 8.0+

### Quick Start

1. Clone the repository:
   ```bash
   git clone https://github.com/poppopjmp/Mirage.git
   cd mirage
   ```

2. Start the development environment:
   ```bash
   docker-compose up -d
   ```

3. Build all services:
   ```bash
   cargo build
   ```

4. Run tests:
   ```bash
   cargo test
   ```

## Service Development

### Creating a New Service

1. Create a new directory under `services/`:
   ```bash
   mkdir -p services/my-new-service/src
   ```

2. Initialize a new Cargo project:
   ```bash
   cd services/my-new-service
   cargo init
   ```

3. Add the service to the workspace in the root `Cargo.toml`.

4. Implement the standard service components:
   - `main.rs` - Application entry point
   - `config.rs` - Configuration handling
   - `models.rs` - Data models
   - `handlers.rs` - API endpoints
   - `services.rs` - Business logic
   - `repositories.rs` - Data access

### Adding a New Module

Modules provide the core functionality for data collection and analysis. To create a new module:

1. Create a directory under `modules/`:
   ```bash
   mkdir -p modules/my-module
   ```

2. Create the module configuration file (`module.json`):
   ```json
   {
     "name": "my-module",
     "version": "0.1.0",
     "description": "Module description",
     "author": "Your Name",
     "capabilities": ["domain_info", "ip_info"],
     "parameters": {
       "api_key": {
         "type": "string",
         "required": true,
         "description": "API key for the service"
       }
     }
   }
   ```

3. Implement the module handler logic in `main.rs` or other source files.

## API Documentation

Each service exposes its API on a standard HTTP port with a `/api/v1` prefix. You can access the API documentation at:

- Auth Service: http://localhost:8000/api/docs
- User Management: http://localhost:8001/api/docs
- Module Registry: http://localhost:8002/api/docs
- And so on...

## Architectural Guidelines

### Communication Between Services

- Services should communicate via HTTP REST APIs
- Use a common error format for all responses
- For high-throughput events, use a message queue

### Database Access

- Each service owns its database schema
- No cross-service database access allowed
- Use migrations for schema changes

### Configuration

- Use environment variables for service configuration
- Store default configurations in files under `config/`
- Sensitive values should not be committed to the repository

## Release Process

1. Update version numbers in `Cargo.toml` files
2. Run the test suite: `cargo test`
3. Create a new git tag: `git tag v0.1.0`
4. Push the tag: `git push origin v0.1.0`
5. The CI pipeline will build and publish docker images

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for details on how to contribute to the project.
