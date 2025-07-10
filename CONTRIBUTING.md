# Contributing to Mirage OSINT Platform

## Development Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/poppopjmp/Mirage.git
   cd Mirage
   ```

2. Copy environment configuration:
   ```bash
   cp .env.example .env
   ```

3. Start development environment:
   ```bash
   docker-compose up -d
   ```

4. Build all services:
   ```bash
   cargo build
   ```

## Code Style

- Use `cargo fmt` for formatting
- Run `cargo clippy` for linting
- All tests must pass: `cargo test`

## Pull Request Process

1. Create a feature branch
2. Make your changes
3. Add tests for new functionality
4. Update documentation
5. Submit a pull request

## Service Development Guidelines

Each service must include:
- Health check endpoint at `/health`
- Proper error handling
- Comprehensive logging
- Unit and integration tests
- OpenAPI documentation