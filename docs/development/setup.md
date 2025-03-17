# Development Environment Setup

This document describes how to set up your development environment for working with Mirage.

## Prerequisites

- Docker Desktop 20.10.0 or higher
- Node.js 16 or higher
- Go 1.18 or higher
- Kubernetes (local) such as minikube or kind
- Helm 3.x

## Clone the Repository

```bash
git clone https://github.com/your-org/Mirage.git
cd Mirage
```

## Local Development

### Running with Docker Compose

For local development, you can use Docker Compose to start all required services:

```bash
docker-compose up -d
```

This will start:
- All microservices
- Required databases
- Message broker
- Local development tools

### Starting Individual Services

Each service can be started independently:

```bash
# Example for starting the Auth Service
cd services/auth
npm install
npm run dev
```

## Kubernetes Development

For a more production-like environment, you can deploy to a local Kubernetes cluster:

```bash
# Start minikube
minikube start

# Deploy using Helm
helm install mirage ./helm/mirage

# Port-forward the API Gateway
kubectl port-forward svc/mirage-api-gateway 8080:80
```

## Environment Configuration

Copy the example environment files:

```bash
cp .env.example .env
```

Edit the `.env` file to match your local setup.

## Running Tests

```bash
# Run all tests
npm test

# Run tests for a specific service
cd services/auth
npm test
```

## Debugging

### Debugging with VS Code

A launch configuration for VS Code is provided in the `.vscode` directory.

### Debugging in Kubernetes

For debugging services running in Kubernetes, use:

```bash
kubectl debug <pod-name> -it --image=busybox
```

## Next Steps

After setting up your development environment, see [Contributing Guidelines](contributing.md) for information on contributing to the project.
