# Mirage Implementation Design

This document provides a detailed implementation design for the Mirage OSINT platform.

## 1. Service Implementation Details

### Auth Service
- **Technology Stack**: Rust + Actix-Web + JWT
- **Database**: PostgreSQL
- **Responsibilities**:
  - User authentication and authorization
  - Token issuance and validation
  - OAuth/OIDC integration
  - Rate limiting
- **Key APIs**:
  - `POST /auth/login` - Authenticate user
  - `POST /auth/refresh` - Refresh token
  - `POST /auth/logout` - Invalidate token

### User Management Service
- **Technology Stack**: Rust + Rocket
- **Database**: PostgreSQL
- **Responsibilities**:
  - User CRUD operations
  - Role and permission management
  - Team management
- **Key APIs**:
  - `GET /users/{id}` - Get user details
  - `PUT /users/{id}` - Update user
  - `POST /teams` - Create team
  - `GET /teams/{id}/members` - Get team members

### Scan Orchestration Service
- **Technology Stack**: Rust + Tokio + gRPC
- **Database**: MongoDB + Redis
- **Responsibilities**:
  - Manage scan execution and lifecycle
  - Schedule and prioritize scans
  - Track scan progress and results
- **Key APIs**:
  - `POST /scans` - Create new scan
  - `GET /scans/{id}` - Get scan details
  - `PUT /scans/{id}/pause` - Pause scan
  - `PUT /scans/{id}/resume` - Resume scan

### Data Collection Service
- **Technology Stack**: Rust + Tokio + Hyper
- **Database**: MongoDB
- **Responsibilities**:
  - Execute OSINT modules against targets
  - Manage rate limiting for external services
  - Handle proxy and VPN rotation
- **Key APIs**:
  - `POST /collection/execute` - Execute data collection module
  - `GET /collection/modules` - List available modules
  - `GET /collection/results/{id}` - Get collection results

### Correlation Engine Service
- **Technology Stack**: Rust + rayon
- **Database**: MongoDB + Redis + Neo4j
- **Responsibilities**:
  - Process collected data
  - Identify relationships between entities
  - Generate entity graphs
- **Key APIs**:
  - `POST /correlate` - Correlate data sets
  - `GET /graphs/{id}` - Get entity graph
  - `GET /entities/{id}/relationships` - Get entity relationships

### Visualization Service
- **Technology Stack**: Rust + WebAssembly + D3.js
- **Database**: Redis (cache) + MongoDB (storage)
- **Responsibilities**:
  - Generate visual representations of data
  - Render interactive graphs and charts
- **Key APIs**:
  - `GET /visualizations/{id}` - Get visualization
  - `POST /visualizations/generate` - Generate visualization

## 2. Data Models

### User Schema (PostgreSQL)
```sql
CREATE TABLE users (
  id UUID PRIMARY KEY,
  username VARCHAR(255) NOT NULL UNIQUE,
  email VARCHAR(255) NOT NULL UNIQUE,
  password_hash VARCHAR(255) NOT NULL,
  first_name VARCHAR(255),
  last_name VARCHAR(255),
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
  last_login TIMESTAMP,
  status VARCHAR(50) NOT NULL DEFAULT 'active'
);
```

### Team Schema (PostgreSQL)
```sql
CREATE TABLE teams (
  id UUID PRIMARY KEY,
  name VARCHAR(255) NOT NULL,
  description TEXT,
  created_by UUID REFERENCES users(id),
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE team_members (
  team_id UUID REFERENCES teams(id),
  user_id UUID REFERENCES users(id),
  role VARCHAR(50) NOT NULL DEFAULT 'member',
  joined_at TIMESTAMP NOT NULL DEFAULT NOW(),
  PRIMARY KEY (team_id, user_id)
);
```

### Scan Document (MongoDB)
```json
{
  "_id": "ObjectId",
  "name": "String",
  "description": "String",
  "target": "String",
  "created_by": "UUID",
  "team_id": "UUID",
  "status": "String",
  "progress": "Number",
  "start_time": "Date",
  "end_time": "Date",
  "modules": ["String"],
  "config": "Object",
  "results_summary": {
    "entities_found": "Number",
    "relationships_found": "Number",
    "data_points": "Number"
  },
  "tags": ["String"]
}
```

### Entity Document (MongoDB)
```json
{
  "_id": "ObjectId",
  "entity_type": "String",
  "value": "String",
  "source_module": "String",
  "scan_id": "ObjectId",
  "confidence": "Number",
  "first_seen": "Date",
  "last_seen": "Date",
  "metadata": "Object",
  "tags": ["String"]
}
```

## 3. Service Communication

### gRPC Interfaces

#### Scan Service Proto
```protobuf
syntax = "proto3";

package scan;

service ScanService {
  rpc CreateScan(CreateScanRequest) returns (Scan);
  rpc GetScan(GetScanRequest) returns (Scan);
  rpc ListScans(ListScansRequest) returns (ListScansResponse);
  rpc UpdateScan(UpdateScanRequest) returns (Scan);
  rpc ControlScan(ControlScanRequest) returns (ControlScanResponse);
}

message CreateScanRequest {
  string name = 1;
  string description = 2;
  string target = 3;
  string user_id = 4;
  optional string team_id = 5;
  repeated string modules = 6;
  map<string, string> config = 7;
}

message Scan {
  string id = 1;
  string name = 2;
  string description = 3;
  string target = 4;
  string status = 5;
  float progress = 6;
  string created_by = 7;
  optional string team_id = 8;
  string start_time = 9;
  optional string end_time = 10;
  repeated string modules = 11;
}
```

### Kafka Topics

- `scan.events` - Scan lifecycle events
- `entity.discovered` - New entities discovered
- `entity.updated` - Updates to existing entities
- `relationship.discovered` - New relationships discovered
- `alerts.generated` - New alerts generated

## 4. API Gateway Configuration

### Envoy Configuration
```yaml
static_resources:
  listeners:
  - address:
      socket_address:
        address: 0.0.0.0
        port_value: 8080
    filter_chains:
    - filters:
      - name: envoy.filters.network.http_connection_manager
        typed_config:
          "@type": type.googleapis.com/envoy.extensions.filters.network.http_connection_manager.v3.HttpConnectionManager
          stat_prefix: ingress_http
          route_config:
            name: local_route
            virtual_hosts:
            - name: mirage_services
              domains: ["*"]
              routes:
              - match: { prefix: "/api/auth/" }
                route: { cluster: auth_service }
              - match: { prefix: "/api/users/" }
                route: { cluster: user_service }
              - match: { prefix: "/api/scans/" }
                route: { cluster: scan_service }
              - match: { prefix: "/api/collection/" }
                route: { cluster: collection_service }
          http_filters:
          - name: envoy.filters.http.jwt_authn
          - name: envoy.filters.http.router
  
  clusters:
  - name: auth_service
    connect_timeout: 0.25s
    type: STRICT_DNS
    lb_policy: ROUND_ROBIN
    load_assignment:
      cluster_name: auth_service
      endpoints:
      - lb_endpoints:
        - endpoint:
            address:
              socket_address:
                address: auth-service
                port_value: 8000

  - name: user_service
    connect_timeout: 0.25s
    type: STRICT_DNS
    lb_policy: ROUND_ROBIN
    load_assignment:
      cluster_name: user_service
      endpoints:
      - lb_endpoints:
        - endpoint:
            address:
              socket_address:
                address: user-service
                port_value: 8001
```

## 5. Implementation Phases

### Phase 1: Core Infrastructure (Weeks 1-4)
- Set up CI/CD pipeline with GitHub Actions
- Implement Kubernetes manifests and Helm charts
- Set up development environment with Docker Compose
- Implement base libraries and shared code

### Phase 2: Authentication & User Management (Weeks 5-8)
- Implement Auth Service
- Implement User Management Service
- Set up API Gateway with authentication
- Implement admin dashboard for user management

### Phase 3: Scanning & Data Collection (Weeks 9-16)
- Implement Scan Orchestration Service
- Implement Data Collection Service
- Implement Module Registry Service
- Port and adapt SpiderFoot modules to new architecture

### Phase 4: Analysis & Visualization (Weeks 17-24)
- Implement Correlation Engine Service
- Implement Visualization Service
- Implement Reporting Service
- Develop interactive user interface

### Phase 5: Integration & Advanced Features (Weeks 25-32)
- Implement Notification Service
- Implement Integration Service with external systems
- Add ML-based analysis features
- Implement automated workflows

### Phase 6: Testing & Optimization (Weeks 33-36)
- Comprehensive system testing
- Performance optimization
- Security auditing
- Documentation finalization

## 6. Development Practices

### Code Organization
- Monorepo structure with workspace-based organization
- Shared libraries in `/libs` directory
- Service-specific code in `/services/{service-name}`
- Infrastructure code in `/infra`
- Common schemas in `/schemas`

### CI/CD Pipeline
- Lint and format check on PR
- Unit tests per service on PR
- Integration tests on main branch
- Automated deployment to staging
- Manual promotion to production

### Monitoring Setup
- Prometheus for metrics collection
- Grafana for visualization
- Jaeger for distributed tracing
- ELK stack for log aggregation
