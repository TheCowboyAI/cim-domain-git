<!-- Copyright 2025 Cowboy AI, LLC. -->

# Contributing to CIM Domain Git

Thank you for your interest in contributing to CIM Domain Git! This document provides guidelines and instructions for contributing.

## Code of Conduct

By participating in this project, you agree to abide by our code of conduct: be respectful, constructive, and professional in all interactions.

## How to Contribute

### Reporting Issues

- Check if the issue already exists in the [issue tracker](https://github.com/thecowboyai/cim-domain-git/issues)
- Provide a clear description of the problem
- Include steps to reproduce the issue
- Specify your environment (OS, Rust version, etc.)

### Submitting Pull Requests

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes following our coding standards
4. Add tests for new functionality
5. Ensure all tests pass (`cargo test`)
6. Run formatters and linters:
   ```bash
   cargo fmt
   cargo clippy -- -D warnings
   ```
7. Commit with clear, descriptive messages
8. Push to your fork and submit a pull request

### Coding Standards

- Follow Rust naming conventions and idioms
- Write clear, self-documenting code
- Add documentation comments for public APIs
- Include unit tests for new functionality
- Maintain or improve code coverage
- All files must include copyright header: `// Copyright 2025 Cowboy AI, LLC.`

### Commit Message Guidelines

- Use present tense ("Add feature" not "Added feature")
- Use imperative mood ("Move cursor to..." not "Moves cursor to...")
- Limit first line to 72 characters
- Reference issues and pull requests when applicable

### Testing

- Write tests for all new functionality
- Ensure existing tests continue to pass
- Aim for high test coverage (we strive for 95%+)
- Use property-based testing where appropriate

### Documentation

- Update README.md if needed
- Add inline documentation for public APIs
- Include examples in doc comments
- Update CHANGELOG.md for notable changes

## Development Setup

1. Install Rust (stable channel)
2. Clone the repository
3. Run `cargo build` to verify setup
4. Run `cargo test` to execute tests
5. Use `cargo doc --open` to view documentation

## Questions?

Feel free to open an issue for any questions about contributing.