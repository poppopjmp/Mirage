# Mirage Implementation Guide

## Technology Stack

### Core Technologies
- **Programming Languages**: Go for performance-critical services, Node.js for API-heavy services
- **API Framework**: RESTful APIs with OpenAPI/Swagger specifications
- **Event Bus**: Apache Kafka for asynchronous communication
- **Service Mesh**: Istio for service-to-service communication
- **Databases**:
  - MongoDB for document storage (modules, scans, reports)
  - PostgreSQL for relational data (users, permissions)
  - Neo4j for graph relationships (correlation engine)
  - Redis for caching and rate limiting

### Infrastructure
- **Containerization**: Docker
- **Orchestration**: Kubernetes
- **CI/CD**: GitHub Actions
- **Monitoring**: Prometheus + Grafana
- **Logging**: ELK Stack (Elasticsearch, Logstash, Kibana)
- **Secrets Management**: HashiCorp Vault

## Service Implementation

### API Gateway
- **Technology**: Kong API Gateway
- **Features**:
  - JWT validation
  - Rate limiting
  - Request transformation
  - Service routing
  - API analytics

```yaml
# Sample Kong configuration
services:
  - name: auth-service
    url: http://auth-service:8080
    routes:
      - paths: ["/api/auth"]
    plugins:
      - name: jwt
      - name: rate-limiting
        config:
          minute: 60
```

### Auth Service
- **Technology**: Node.js with Express, Passport
- **Database**: PostgreSQL
- **Responsibilities**:
  - User authentication
  - JWT issuance and verification
  - OAuth integration

```javascript
// Sample Auth Service code
const express = require('express');
const jwt = require('jsonwebtoken');
const app = express();

app.post('/api/auth/login', async (req, res) => {
  const { username, password } = req.body;
  
  // Validate credentials against database
  const user = await db.users.findOne({ username });
  if (!user || !validatePassword(password, user.passwordHash)) {
    return res.status(401).json({ error: 'Invalid credentials' });
  }
  
  // Generate JWT token
  const token = jwt.sign(
    { id: user.id, roles: user.roles }, 
    process.env.JWT_SECRET, 
    { expiresIn: '24h' }
  );
  
  res.json({ token });
});
```

### Scan Orchestration Service
- **Technology**: Go
- **Database**: MongoDB
- **Responsibilities**:
  - Managing scan configurations
  - Orchestrating the scan process
  - Tracking scan status and progress

```go
// Sample Scan Orchestration Service code
package main

import (
    "github.com/gin-gonic/gin"
    "go.mongodb.org/mongo-driver/bson"
    "go.mongodb.org/mongo-driver/mongo"
)

type ScanConfig struct {
    ID          string   `json:"id" bson:"_id"`
    Name        string   `json:"name" bson:"name"`
    Modules     []string `json:"modules" bson:"modules"`
    Target      string   `json:"target" bson:"target"`
    Schedule    string   `json:"schedule" bson:"schedule"`
    CreatedBy   string   `json:"createdBy" bson:"created_by"`
    CreatedAt   int64    `json:"createdAt" bson:"created_at"`
    Status      string   `json:"status" bson:"status"`
}

func createScan(c *gin.Context) {
    var scanConfig ScanConfig
    if err := c.BindJSON(&scanConfig); err != nil {
        c.JSON(400, gin.H{"error": err.Error()})
        return
    }
    
    // Create scan in database
    // Publish scan creation event to Kafka
    
    c.JSON(201, scanConfig)
}
```

### Data Collection Service
- **Technology**: Go for the core, with module plugins
- **Storage**: Temporary storage with Redis, permanent in Data Storage Service
- **Key Features**:
  - Plugin architecture for collection modules
  - Rate limiting for external API calls
  - Resilient collection with retry logic

```go
// Sample Data Collection Service code
package collector

import (
    "context"
    "time"
)

type CollectionModule interface {
    ID() string
    Collect(ctx context.Context, target string, options map[string]interface{}) ([]byte, error)
    RateLimit() RateLimit
}

type RateLimit struct {
    RequestsPerMinute int
    RequestsPerHour   int
    RequestsPerDay    int
}

func ExecuteCollection(ctx context.Context, module CollectionModule, target string) {
    // Check rate limits
    if !checkRateLimit(module) {
        scheduleRetry(module, target)
        return
    }
    
    // Execute collection
    results, err := module.Collect(ctx, target, nil)
    if err != nil {
        handleCollectionError(err, module, target)
        return
    }
    
    // Normalize results
    normalized := normalizeResults(results)
    
    // Send to Data Storage Service
    storeResults(ctx, normalized)
}
```

### Correlation Engine Service
- **Technology**: Python with data science libraries
- **Database**: Neo4j graph database
- **Key Features**:
  - Entity extraction and linking
  - Relationship identification
  - Confidence scoring algorithms

```python
# Sample Correlation Engine code
from neo4j import GraphDatabase

class CorrelationEngine:
    def __init__(self, uri, user, password):
        self.driver = GraphDatabase.driver(uri, auth=(user, password))
        
    def create_entity(self, entity_type, properties):
        with self.driver.session() as session:
            return session.write_transaction(
                self._create_entity, entity_type, properties)
                
    def create_relationship(self, source_id, relationship_type, target_id, confidence):
        with self.driver.session() as session:
            return session.write_transaction(
                self._create_relationship, source_id, relationship_type, target_id, confidence)
    
    @staticmethod
    def _create_entity(tx, entity_type, properties):
        query = (
            f"CREATE (n:{entity_type} $properties) "
            "RETURN id(n) AS entity_id"
        )
        result = tx.run(query, properties=properties)
        return result.single()["entity_id"]
        
    @staticmethod
    def _create_relationship(tx, source_id, relationship_type, target_id, confidence):
        query = (
            "MATCH (a), (b) "
            "WHERE id(a) = $source_id AND id(b) = $target_id "
            f"CREATE (a)-[r:{relationship_type} {{confidence: $confidence}}]->(b) "
            "RETURN id(r) AS relationship_id"
        )
        result = tx.run(query, source_id=source_id, target_id=target_id, confidence=confidence)
        return result.single()["relationship_id"]
```

## Deployment Configuration

### Docker Compose (Development)

```yaml
version: '3.8'

services:
  api-gateway:
    image: kong:latest
    environment:
      - KONG_DATABASE=postgres
      - KONG_PG_HOST=kong-database
    ports:
      - "8000:8000"
      - "8443:8443"
    depends_on:
      - kong-database
      
  kong-database:
    image: postgres:13
    environment:
      - POSTGRES_USER=kong
      - POSTGRES_DB=kong
      - POSTGRES_PASSWORD=kong_pass
      
  auth-service:
    build: ./services/auth
    environment:
      - DB_HOST=auth-db
      - JWT_SECRET=dev_secret_replace_in_production
    depends_on:
      - auth-db
      
  auth-db:
    image: postgres:13
    environment:
      - POSTGRES_USER=auth_user
      - POSTGRES_DB=auth_db
      - POSTGRES_PASSWORD=auth_pass
      
  scan-orchestrator:
    build: ./services/scan-orchestrator
    environment:
      - MONGO_URI=mongodb://scan-db:27017/scans
      - KAFKA_BROKERS=kafka:9092
    depends_on:
      - scan-db
      - kafka
      
  scan-db:
    image: mongo:5
    
  kafka:
    image: confluentinc/cp-kafka:latest
    depends_on:
      - zookeeper
    environment:
      - KAFKA_ZOOKEEPER_CONNECT=zookeeper:2181
      - KAFKA_ADVERTISED_LISTENERS=PLAINTEXT://kafka:9092
      
  zookeeper:
    image: confluentinc/cp-zookeeper:latest
    environment:
      - ZOOKEEPER_CLIENT_PORT=2181
```

### Kubernetes Manifests

Create a `kubernetes` directory with service-specific deployment files:

```
kubernetes/
├── api-gateway/
│   ├── deployment.yaml
│   └── service.yaml
├── auth-service/
│   ├── deployment.yaml
│   ├── service.yaml
│   └── secret.yaml
└── ...
```

Example deployment for the Auth Service:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: auth-service
  namespace: mirage
spec:
  replicas: 3
  selector:
    matchLabels:
      app: auth-service
  template:
    metadata:
      labels:
        app: auth-service
    spec:
      containers:
      - name: auth-service
        image: mirage/auth-service:latest
        ports:
        - containerPort: 8080
        env:
        - name: DB_HOST
          value: auth-db-service
        - name: JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: auth-secrets
              key: jwt-secret
        resources:
          limits:
            cpu: "500m"
            memory: "512Mi"
          requests:
            cpu: "100m"
            memory: "256Mi"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
```

## Development Workflow

1. **Local Setup**
   - Install Docker and Docker Compose
   - Clone the repository
   - Run `docker-compose up` to start development environment

2. **Service Development**
   - Each service has its own directory in `/services`
   - Follow service-specific README for development instructions
   - Use shared libraries from `/libs` where appropriate

3. **Testing**
   - Unit tests: Run within each service directory
   - Integration tests: Use the test environment with Docker Compose
   - E2E tests: Deploy to staging Kubernetes environment

4. **CI/CD Pipeline**
   - Commit triggers unit tests
   - PR triggers integration tests
   - Merge to main deploys to staging
   - Tag release deploys to production

## Next Steps

1. Set up the development environment with Docker Compose
2. Implement the core services (Auth, User Management)
3. Establish CI/CD pipelines
4. Implement the data collection and storage services
5. Add correlation and visualization capabilities
6. Implement reporting and notifications
