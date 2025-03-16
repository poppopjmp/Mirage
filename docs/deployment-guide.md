# Mirage Deployment Guide

This guide outlines the deployment options and procedures for the Mirage platform.

## Deployment Options

Mirage can be deployed in various environments:

1. **Local Development**: Using Docker Compose
2. **Single-Node Production**: Using Docker Compose with production settings
3. **Kubernetes Cluster**: Recommended for production deployments
4. **Managed Kubernetes**: Using services like EKS, GKE, or AKS

## Prerequisites

- Docker Engine 20.10+
- Docker Compose 2.0+ (for local and single-node deployments)
- Kubernetes 1.22+ (for cluster deployments)
- Helm 3.7+ (for Kubernetes deployments)
- A domain name and SSL certificates for production deployments
- Adequate resources based on expected usage scale

## Local Development Deployment

For local development and testing:

```bash
# Clone the repository
git clone https://github.com/mirage/mirage.git
cd mirage

# Create and configure environment variables
cp .env.example .env
# Edit .env file with appropriate values

# Start the development environment
docker-compose -f docker-compose.dev.yml up -d

# Access the services
# Web UI: http://localhost:8080
# API: http://localhost:8081
```

## Single-Node Production Deployment

For a small-scale production environment on a single server:

```bash
# Clone the repository
git clone https://github.com/mirage/mirage.git
cd mirage

# Create and configure environment variables
cp .env.example .env.prod
# Edit .env.prod file with appropriate values

# Start the production environment
docker-compose -f docker-compose.prod.yml up -d

# Setup a reverse proxy (Nginx/Traefik) for SSL termination and routing
```

### Hardware Requirements (Single-Node)

| Component | Minimum | Recommended |
|-----------|---------|------------|
| CPU | 4 cores | 8+ cores |
| RAM | 16GB | 32GB+ |
| Storage | 100GB SSD | 500GB+ SSD |
| Network | 100Mbps | 1Gbps+ |

## Kubernetes Deployment

For scalable production deployments:

### Preparation

1. Set up a Kubernetes cluster with required add-ons:
   - Ingress controller (e.g., Nginx Ingress or Traefik)
   - Cert-Manager for SSL certificates
   - Prometheus and Grafana for monitoring
   - Elasticsearch, Fluentd, and Kibana for logging

2. Configure kubectl to connect to your cluster

3. Install Helm if not already installed

### Installation using Helm

```bash
# Add the Mirage Helm repository
helm repo add mirage https://charts.mirage.io
helm repo update

# Create a namespace for Mirage
kubectl create namespace mirage

# Create secrets for sensitive information
kubectl -n mirage create secret generic mirage-secrets \
  --from-literal=postgresql-password=YOUR_POSTGRES_PASSWORD \
  --from-literal=mongodb-password=YOUR_MONGODB_PASSWORD \
  --from-literal=rabbitmq-password=YOUR_RABBITMQ_PASSWORD \
  --from-literal=redis-password=YOUR_REDIS_PASSWORD \
  --from-literal=jwt-secret=YOUR_JWT_SECRET

# Install Mirage using Helm
helm install mirage mirage/mirage -n mirage \
  -f values.yaml
```

### Example `values.yaml` file

```yaml
global:
  domain: mirage.example.com
  tls:
    enabled: true
    issuer: letsencrypt-prod

persistence:
  storageClass: "standard"
  size: "50Gi"

postgresql:
  resources:
    requests:
      memory: "2Gi"
      cpu: "1"
    limits:
      memory: "4Gi"
      cpu: "2"

mongodb:
  resources:
    requests:
      memory: "4Gi"
      cpu: "2"
    limits:
      memory: "8Gi"
      cpu: "4"

rabbitmq:
  resources:
    requests:
      memory: "1Gi"
      cpu: "0.5"
    limits:
      memory: "2Gi"
      cpu: "1"

services:
  apiGateway:
    replicas: 2
  authService:
    replicas: 2
  userManagement:
    replicas: 2
  scanOrchestration:
    replicas: 2
  dataCollection:
    replicas: 5
    workers: 10
  dataStorage:
    replicas: 2
  correlationEngine:
    replicas: 2
```

### Horizontal Scaling

Most Mirage services can be scaled horizontally by increasing the replica count:

```bash
kubectl scale deployment -n mirage mirage-data-collection --replicas=10
```

Services that maintain state (like `scanOrchestration`) use StatefulSets to ensure proper data consistency during scaling.

## Cloud-Specific Deployment

### AWS EKS Deployment

1. Create an EKS cluster using eksctl or AWS Console
2. Set up required AWS resources (RDS, ElastiCache, MSK, etc.)
3. Install Mirage Helm chart with AWS-specific configurations

### Google GKE Deployment

1. Create a GKE cluster using gcloud or Google Cloud Console
2. Set up required GCP resources (Cloud SQL, Memorystore, Pub/Sub, etc.)
3. Install Mirage Helm chart with GCP-specific configurations

### Azure AKS Deployment

1. Create an AKS cluster using az cli or Azure Portal
2. Set up required Azure resources (Azure Database, Redis Cache, Service Bus, etc.)
3. Install Mirage Helm chart with Azure-specific configurations

## Post-Deployment Configuration

After deployment, complete these additional steps:

1. Set up an administrator account:
```bash
kubectl -n mirage exec -it deployment/mirage-user-management -- ./create-admin.sh
```

2. Configure external integrations through the admin interface

3. Set up backup procedures for databases

## Backup and Disaster Recovery

### Database Backups

1. Configure automated backups for PostgreSQL and MongoDB
2. For PostgreSQL:
```bash
kubectl -n mirage exec -it statefulset/mirage-postgresql-0 -- \
  pg_dump -U postgres mirage > mirage_backup_$(date +%Y%m%d).sql
```

3. For MongoDB:
```bash
kubectl -n mirage exec -it statefulset/mirage-mongodb-0 -- \
  mongodump --archive > mirage_mongo_$(date +%Y%m%d).archive
```

### Restore Procedures

1. For PostgreSQL:
```bash
kubectl -n mirage exec -it statefulset/mirage-postgresql-0 -- \
  psql -U postgres -d mirage < mirage_backup_20230101.sql
```

2. For MongoDB:
```bash
kubectl -n mirage exec -it statefulset/mirage-mongodb-0 -- \
  mongorestore --archive < mirage_mongo_20230101.archive
```

## Monitoring and Logging

- Access Grafana for metrics: `https://grafana.mirage.example.com`
- Access Kibana for logs: `https://kibana.mirage.example.com`
- Set up alerts for critical service thresholds

## Troubleshooting

### Common Issues

1. **Service Unavailable**: Check pod status and logs
```bash
kubectl -n mirage get pods
kubectl -n mirage logs deployment/mirage-api-gateway
```

2. **Database Connection Errors**: Verify secrets and network policies
```bash
kubectl -n mirage describe secret mirage-secrets
kubectl -n mirage exec -it deployment/mirage-data-storage -- env | grep DB_
```

3. **Performance Issues**: Check resource utilization
```bash
kubectl -n mirage top pods
kubectl -n mirage top nodes
```
