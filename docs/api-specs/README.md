# Mirage API Specifications

This directory contains the API specifications for all Mirage microservices. We use the OpenAPI (Swagger) specification for REST APIs and Protocol Buffers for gRPC services.

## API Design Principles

1. **Consistency**: All APIs follow consistent naming, error handling, and versioning
2. **Simplicity**: APIs should be intuitive and easy to understand
3. **Documentation**: All endpoints, parameters, and responses are documented
4. **Versioning**: APIs are versioned to allow for evolution
5. **Security**: Authentication and authorization are built into the API design
6. **Pagination**: Collections support pagination for performance
7. **Filtering**: Collections support filtering for better data access
8. **HATEOAS**: APIs include links to related resources where appropriate

## REST API Standards

### URL Structure

- Use nouns, not verbs (e.g., `/users`, not `/getUsers`)
- Use plural nouns for collections (e.g., `/users`, not `/user`)
- Nest resources to show relationships (e.g., `/users/{id}/scans`)
- Use query parameters for filtering, sorting, and pagination

### HTTP Methods

- `GET`: Retrieve resources
- `POST`: Create resources
- `PUT`: Replace resources
- `PATCH`: Update resources partially
- `DELETE`: Remove resources

### Status Codes

- `200 OK`: Request succeeded
- `201 Created`: Resource created
- `204 No Content`: Success with no response body
- `400 Bad Request`: Invalid request format
- `401 Unauthorized`: Authentication required
- `403 Forbidden`: Not authorized for the resource
- `404 Not Found`: Resource not found
- `409 Conflict`: Resource conflict
- `422 Unprocessable Entity`: Validation error
- `429 Too Many Requests`: Rate limit exceeded
- `500 Internal Server Error`: Server error

### Common Query Parameters

- `page`: Page number (1-based)
- `per_page`: Items per page
- `sort`: Sort field
- `order`: Sort order (`asc` or `desc`)
- `filter`: Field-specific filters

### Response Format

```json
{
  "data": {},
  "meta": {
    "pagination": {
      "total": 100,
      "per_page": 10,
      "current_page": 1,
      "last_page": 10
    }
  },
  "links": {
    "self": "https://api.mirage.io/v1/resource?page=1",
    "next": "https://api.mirage.io/v1/resource?page=2",
    "prev": null
  }
}
```

### Error Format

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "The request contains invalid parameters",
    "details": [
      {
        "field": "email",
        "message": "Invalid email format"
      }
    ]
  }
}
```

## Available API Specifications

| Service | Version | Specification |
|---------|---------|---------------|
| API Gateway | v1 | [api-gateway.yaml](./api-gateway.yaml) |
| Auth Service | v1 | [auth-service.yaml](./auth-service.yaml) |
| User Management | v1 | [user-service.yaml](./user-service.yaml) |
| Scan Orchestration | v1 | [scan-orchestration.yaml](./scan-orchestration.yaml) |
| Module Registry | v1 | [module-registry.yaml](./module-registry.yaml) |
| Data Collection | v1 | [data-collection.yaml](./data-collection.yaml) |
| Data Storage | v1 | [data-storage.yaml](./data-storage.yaml) |
| Correlation Engine | v1 | [correlation-engine.yaml](./correlation-engine.yaml) |
| Visualization | v1 | [visualization.yaml](./visualization.yaml) |
| Reporting | v1 | [reporting.yaml](./reporting.yaml) |
| Notification | v1 | [notification.yaml](./notification.yaml) |
| Integration | v1 | [integration.yaml](./integration.yaml) |

## Using the API Specifications

### Generating Client Libraries

You can generate client libraries for various languages using the OpenAPI Generator:

```bash
# Generate a Python client
openapi-generator generate -i ./api-specs/scan-orchestration.yaml -g python -o ./clients/python/scan-orchestration

# Generate a TypeScript client
openapi-generator generate -i ./api-specs/scan-orchestration.yaml -g typescript-fetch -o ./clients/typescript/scan-orchestration
```

### Testing APIs

You can use the included Swagger UI to test APIs:

```
http://localhost:8080/swagger-ui
```

### Updating Specifications

When updating an API specification:

1. Make changes to the YAML file
2. Update the version if changes are not backward compatible
3. Generate updated client libraries
4. Update the relevant service implementation
5. Document the changes in the changelog
