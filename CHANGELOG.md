<!-- Copyright 2025 Cowboy AI, LLC. -->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial implementation of Git domain for CIM
- Repository aggregate with cloning and analysis capabilities
- Commit and Branch aggregates with full lifecycle support
- Value objects: CommitHash, BranchName, RemoteUrl, AuthorInfo, FilePath
- Domain events following event sourcing patterns
- CQRS command and query structure
- Multi-language dependency analysis (Rust, Python, JS/TS, Java, Go, C/C++)
- Security features: path traversal and command injection protection
- Performance optimizations with caching layer
- Comprehensive test coverage

### Changed
- Removed direct dependencies on other CIM domains (graph, document, agent)
- Updated to use GitHub-hosted CIM dependencies
- Improved domain isolation for better composability

### Security
- Added input validation for all value objects
- Implemented path traversal protection
- Added command injection prevention

## [0.3.0] - 2024-01-XX

### Added
- Initial public release

[Unreleased]: https://github.com/thecowboyai/cim-domain-git/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/thecowboyai/cim-domain-git/releases/tag/v0.3.0