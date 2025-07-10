# Contributing to Mirage OSINT Platform

We welcome contributions to the Mirage OSINT platform! This document outlines the process for contributing to the project.

## Getting Started

1. Fork the repository on GitHub
2. Clone your fork locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/Mirage.git
   cd Mirage
   ```
3. Set up the development environment as described in the [README.md](README.md)

## Development Workflow

1. Create a new branch for your feature or bugfix:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. Make your changes following our coding standards
3. Test your changes thoroughly
4. Commit your changes with a clear commit message
5. Push to your fork and submit a pull request

## Coding Standards

### Rust Code
- Follow standard Rust formatting (`cargo fmt`)
- Ensure code passes linting (`cargo clippy`)
- Write comprehensive tests for new functionality
- Use descriptive variable and function names
- Add documentation comments for public APIs

### TypeScript/JavaScript Code
- Use TypeScript for all new frontend code
- Follow ESLint configuration
- Write unit tests for components and utilities
- Use semantic component and variable names

### Docker and Infrastructure
- Keep Dockerfiles minimal and efficient
- Use multi-stage builds where appropriate
- Document any new environment variables
- Ensure services are properly health-checked

## Testing

- Run the full test suite before submitting a PR:
  ```bash
  cargo test --workspace
  ```
- Add tests for any new functionality
- Ensure all existing tests continue to pass
- Test your changes in the development environment

## Documentation

- Update documentation for any API changes
- Add comments for complex logic
- Update README.md if you add new features or change setup instructions

## Pull Request Process

1. Ensure your PR description clearly describes the problem and solution
2. Include the relevant issue number if applicable
3. Make sure all tests pass and code follows our standards
4. Request review from maintainers
5. Address any feedback promptly

## Code of Conduct

- Be respectful and inclusive in all interactions
- Focus on constructive feedback
- Help newcomers learn and contribute
- Follow the project's technical decisions and architecture

## Issue Reporting

When reporting issues:
- Use the issue templates provided
- Include steps to reproduce the problem
- Provide relevant logs and error messages
- Specify your environment (OS, Docker version, etc.)

## Security Issues

For security-related issues, please email the maintainers directly rather than opening a public issue.

## Questions?

If you have questions about contributing, feel free to:
- Open a discussion on GitHub
- Ask in the pull request comments
- Contact the maintainers

Thank you for contributing to Mirage!