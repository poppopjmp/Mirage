# Mirage Docker Configuration

This directory contains Docker configurations for the Mirage OSINT platform, organized in a consistent manner to facilitate deployment and development.

## Directory Structure

- `services/` - Contains Dockerfiles for all microservices
- `compose/` - Contains docker-compose files for different deployment scenarios

## Docker Services

Each service in the Mirage platform has its own Dockerfile located in `docker/services/{service-name}/Dockerfile`. The Dockerfiles are structured to:

1. Build the service from source in a builder stage
2. Copy only the necessary artifacts to a minimal runtime container
3. Configure appropriate environment variables and expose needed ports

## Service Binary Naming Convention

All Rust services follow the naming convention `mirage-{service-name}` for the compiled binary.

## Building and Running

To build and run the entire Mirage platform:

```bash
docker-compose up --build
```

To build and run a specific service:

```bash
docker-compose up --build {service-name}
```

## Service Dependencies

The docker-compose file defines service dependencies to ensure services start in the correct order. The general dependency hierarchy is:

1. Infrastructure services (databases, message brokers)
2. Core services (config, discovery, auth)
3. Domain services (user management, module registry, etc.)
4. Frontend services (web-ui)

## Configuration

Services are configured through environment variables defined in the docker-compose file. For more complex configurations, configuration files are mounted from the `config/` directory to the appropriate location in each container.
