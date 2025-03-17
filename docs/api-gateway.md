# Mirage API Gateway

The API Gateway serves as the single entry point for all client requests to the Mirage platform. It handles routing, authentication validation, rate limiting, and more.

## Features

- **Centralized Authentication**: Validates authentication tokens for all services
- **Rate Limiting**: Prevents abuse through configurable rate limits
- **Load Balancing**: Distributes traffic to service instances
- **Caching**: Improves performance for read-heavy endpoints
- **Logging & Monitoring**: Centralized request logging and service health monitoring
- **CORS Support**: Configured for cross-origin requests
- **Security Headers**: Implements best practices for security headers

## Architecture

