# Mirage Architecture Overview

## High-Level Architecture

Mirage is built as a cloud-native application following microservices architecture principles. The system is composed of loosely coupled services that communicate primarily through asynchronous messaging, with synchronous REST APIs where appropriate.

```
                                  ┌─────────────────┐
                                  │                 │
                                  │  API Gateway    │
                                  │                 │
                                  └────────┬────────┘
                                           │
                                           ▼
┌─────────────┐  ┌─────────────┐  ┌─────────────────┐  ┌─────────────┐  ┌────────────────┐
│             │  │             │  │                 │  │             │  │                │
│ Auth        │  │ User        │  │ Scan            │  │ Module      │  │ Notification   │
│ Service     │  │ Management  │  │ Orchestration   │  │ Registry    │  │ Service        │
│             │  │ Service     │  │ Service         │  │ Service     │  │                │
└─────────────┘  └─────────────┘  └────────┬────────┘  └─────────────┘  └────────────────┘
                                           │
                                           ▼
┌─────────────┐  ┌─────────────┐  ┌─────────────────┐  ┌─────────────┐  ┌────────────────┐
│             │  │             │  │                 │  │             │  │                │
│ Data        │  │ Data        │  │ Correlation     │  │ Visualization│  │ Reporting      │
│ Collection  │  │ Storage     │  │ Engine          │  │ Service     │  │ Service        │
│ Service     │  │ Service     │  │ Service         │  │             │  │                │
└─────────────┘  └─────────────┘  └─────────────────┘  └─────────────┘  └────────────────┘
```
