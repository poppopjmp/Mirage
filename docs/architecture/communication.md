# Communication Patterns

## Service Communication

- **Synchronous Communication**: REST APIs for direct request/response patterns
- **Asynchronous Communication**: Message queues for event-driven processes
- **Service Mesh**: For managing service-to-service communication with:
  - Circuit breaking
  - Retries
  - Timeouts
  - Load balancing
  - Observability

## Data Management

Each service maintains its own database when needed, following the database-per-service pattern. The Data Storage Service acts as the central repository for all collected intelligence data, while other services maintain databases specific to their domain.
