# Mirage OSINT Platform

Mirage is an open-source intelligence (OSINT) platform built as a microservice architecture. It provides a scalable and extensible framework for collecting, analyzing, and visualizing intelligence data from various sources.

## Architecture

Mirage is built using a microservice architecture with the following components:

* **API Gateway**: Central entry point for all client requests
* **Authentication Service**: Handles user authentication and authorization
* **User Management Service**: Manages user accounts and permissions
* **Scan Orchestration Service**: Coordinates data collection tasks
* **Module Registry Service**: Manages OSINT modules
* **Data Collection Service**: Executes OSINT collection modules
* **Data Storage Service**: Stores collected intelligence data
* **Correlation Engine Service**: Analyzes relationships between collected data
* **Visualization Service**: Creates visual representations of data
* **Reporting Service**: Generates reports based on collected data
* **Notification Service**: Handles alerts and notifications
* **Integration Service**: Connects to external systems
* **Configuration Service**: Manages system configuration
* **Discovery Service**: Service discovery and registration

## Technology Stack

* **Backend**: Primarily Rust (Actix-web, Rocket), with some Go and Python services
* **Frontend**: React with TypeScript
* **Databases**: PostgreSQL, MongoDB, Neo4j
* **Cache**: Redis
* **Search**: Elasticsearch
* **Messaging**: RabbitMQ
* **Containerization**: Docker
* **Orchestration**: Kubernetes

## Getting Started

### Prerequisites

* Docker and Docker Compose
* Rust 1.70 or later
* Node.js 18 or later
* Python 3.11 or later
* Go 1.21 or later

### Development Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/van1sh/Mirage.git
   cd Mirage
   ```

2. Create a `.env` file with necessary configuration:
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

3. Start the development environment:
   ```bash
   docker-compose -f docker-compose.dev.yml up
   ```

4. Access the web UI at `http://localhost:3000`

### Production Deployment

For production deployment, we recommend using Kubernetes:

```bash
# Setup Kubernetes namespace and secrets
kubectl apply -f kubernetes/mirage-namespace.yaml

# Deploy infrastructure components
kubectl apply -f kubernetes/infrastructure/

# Deploy services
kubectl apply -f kubernetes/deployments/
```

## Documentation

* [Architecture Overview](docs/architecture.md)
* [API Documentation](docs/api.md)
* [Module Development Guide](docs/modules.md)
* [Deployment Guide](docs/deployment.md)

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

