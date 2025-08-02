<!-- Copyright 2025 Cowboy AI, LLC. -->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.0] - 2025-01-02

### Added
- Graph analytics capabilities for extracting metadata optimized for graph building
- CollaborationAnalyzer for detecting developer collaboration patterns
- CodeQualityAnalyzer for technical debt detection and health metrics
- 10 new event types for graph analytics (CollaborationDetected, CodeOwnershipCalculated, etc.)
- Comprehensive example demonstrating graph analysis features
- Documentation for graph analysis features
- Root directory cleanup with organized status files

### Changed
- Updated to async-nats 0.42 (latest version)
- Migrated all NATS integration to use latest async-nats APIs
- Fixed all JetStream integration tests
- Improved project structure with status files moved to doc/status/

### Fixed
- All compilation warnings resolved
- 96 tests passing (up from 93)
- JetStream stream conflicts in integration tests
- EventEnvelopeBuilder API usage corrected

## [0.3.0] - 2025-01-01

### Added
- Initial implementation of Git domain for CIM
- Repository aggregate with cloning and analysis capabilities
- Commit and Branch aggregates with full lifecycle support
- Value objects: CommitHash, BranchName, RemoteUrl, AuthorInfo, FilePath, TagName
- Domain events following event sourcing patterns
- CQRS command and query structure
- Multi-language dependency analysis (Rust, Python, JS/TS, Java, Go, C/C++)
- Security features: path traversal and command injection protection
- Performance optimizations with caching layer
- NATS integration with JetStream support
- Comprehensive test coverage (93 tests)

### Changed
- Removed direct dependencies on other CIM domains (graph, document, agent)
- Updated to use GitHub-hosted CIM dependencies
- Improved domain isolation for better composability

### Security
- Added input validation for all value objects
- Implemented path traversal protection
- Added command injection prevention

## [0.1.0] - 2024-12-15

### Added
- Initial public release

[Unreleased]: https://github.com/thecowboyai/cim-domain-git/compare/v0.5.0...HEAD
[0.5.0]: https://github.com/thecowboyai/cim-domain-git/compare/v0.3.0...v0.5.0
[0.3.0]: https://github.com/thecowboyai/cim-domain-git/compare/v0.1.0...v0.3.0
[0.1.0]: https://github.com/thecowboyai/cim-domain-git/releases/tag/v0.1.0