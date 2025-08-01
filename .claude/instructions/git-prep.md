<!-- Copyright 2025 Cowboy AI, LLC. -->

# Git Repository Preparation Guide

This guide documents the standard process for preparing CIM domain repositories for GitHub publication.

## Overview

When preparing a CIM domain repository for GitHub, follow these steps to ensure consistency with the CIM ecosystem standards.

## Checklist

### 1. Documentation

- [ ] **README.md**: Create a professional README with:
  - Project badges (CI, Crates.io, Documentation, Coverage, License)
  - Clear project description
  - Architecture overview
  - Usage examples
  - Development instructions
  - Status indicators
  - Copyright notice at the top

- [ ] **CONTRIBUTING.md**: Include:
  - Code of conduct
  - How to report issues
  - Pull request process
  - Coding standards
  - Testing requirements
  - Documentation requirements

- [ ] **CHANGELOG.md**: Following Keep a Changelog format:
  - Unreleased section
  - Semantic versioning
  - Links to releases
  - Copyright notice at the top

- [ ] **Update CLAUDE.md**: Ensure it accurately reflects:
  - Current architecture
  - Available commands
  - Domain patterns
  - Integration points

### 2. Licensing

- [ ] **LICENSE**: Main license file pointing to dual licensing
- [ ] **LICENSE-MIT**: MIT license text
- [ ] **LICENSE-APACHE**: Apache 2.0 license text
- [ ] Ensure all files reference "MIT OR Apache-2.0"

### 3. Git Configuration

- [ ] **.gitignore**: Include patterns for:
  ```
  # Rust
  target/
  Cargo.lock
  **/*.rs.bk
  *.pdb
  
  # IDE
  .idea/
  .vscode/
  *.swp
  *.swo
  *~
  
  # OS
  .DS_Store
  Thumbs.db
  
  # Nix
  result
  result-*
  
  # Testing
  tarpaulin-report.html
  cobertura.xml
  coverage/
  test-results/
  
  # Temporary files
  *.tmp
  *.temp
  .cache/
  
  # Documentation
  /target/doc
  /target/criterion
  
  # Examples output
  examples/output/
  
  # Local environment
  .env
  .env.local
  
  # Backup files
  *.bak
  *.backup
  
  # Log files
  *.log
  ```

### 4. GitHub Actions

Create `.github/workflows/` directory with:

- [ ] **ci.yml**: Continuous Integration
  - Format checking
  - Clippy linting
  - Test execution
  - Doc tests
  - Security audit

- [ ] **release.yml**: Release automation
  - Version verification
  - Crates.io publication
  - GitHub release creation

### 5. Cargo.toml Metadata

Update with proper metadata:
```toml
[package]
name = "cim-domain-{name}"
version = "0.3.0"
edition = "2021"
authors = ["Cowboy AI, LLC <dev@thecowboy.ai>"]
description = "Clear, concise description"
documentation = "https://docs.rs/cim-domain-{name}"
homepage = "https://github.com/thecowboyai/cim-domain-{name}"
repository = "https://github.com/thecowboyai/cim-domain-{name}"
license = "MIT OR Apache-2.0"
readme = "README.md"
keywords = ["domain", "cim", "event-sourcing", "ddd", "specific-keyword"]
categories = ["development-tools", "data-structures"]
exclude = [
    ".github/",
    "tests/",
    "examples/",
    "benches/",
    ".gitignore",
    ".git/",
    "*.log",
    "target/",
]
```

### 6. Copyright Headers

Add copyright headers to all source files:

- [ ] **Rust files (.rs)**: 
  ```rust
  // Copyright 2025 Cowboy AI, LLC.
  
  ```

- [ ] **Markdown files (.md)**:
  ```markdown
  <!-- Copyright 2025 Cowboy AI, LLC. -->
  
  ```

### 7. Dependencies

- [ ] Update dependencies to use GitHub sources where appropriate
- [ ] Remove local path dependencies
- [ ] Ensure domain isolation (no cross-domain dependencies)
- [ ] Use proper version specifications

### 8. Code Quality

- [ ] Run `cargo fmt` to format all code
- [ ] Run `cargo clippy -- -D warnings` and fix all issues
- [ ] Ensure all tests pass with `cargo test`
- [ ] Add or update tests for new functionality
- [ ] Document all public APIs

### 9. Git Commit

Create initial commit:
```bash
git add -A
git commit -m "Initial commit: CIM Domain {Name}

- Complete domain implementation with {key features}
- {Additional key feature}
- Comprehensive test coverage
- Full documentation and GitHub repository setup
- Copyright headers on all source files
- CI/CD workflows for GitHub Actions
- Dual MIT/Apache-2.0 licensing"
```

### 10. GitHub Repository Setup

After pushing to GitHub:
- [ ] Enable branch protection for `main`
- [ ] Set up required status checks
- [ ] Configure Dependabot for dependency updates
- [ ] Add repository description and topics
- [ ] Set up GitHub Pages if documentation site needed
- [ ] Configure secrets for CI/CD (CRATES_IO_TOKEN)

## Domain Independence

Remember that CIM domains should be:
- Independent and composable
- Not directly dependent on other domains
- Integrated at the composition layer
- Following event-driven patterns

## Standard Badge URLs

Use these badge patterns in README.md:
```markdown
[![CI](https://github.com/thecowboyai/cim-domain-{name}/actions/workflows/ci.yml/badge.svg)](https://github.com/thecowboyai/cim-domain-{name}/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/cim-domain-{name}.svg)](https://crates.io/crates/cim-domain-{name})
[![Documentation](https://docs.rs/cim-domain-{name}/badge.svg)](https://docs.rs/cim-domain-{name})
[![Test Coverage](https://img.shields.io/codecov/c/github/thecowboyai/cim-domain-{name})](https://codecov.io/gh/thecowboyai/cim-domain-{name})
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
```

## Notes

- Always include `.claude/` directory in commits for consistency
- Ensure all paths use the GitHub organization: `thecowboyai`
- Follow semantic versioning for releases
- Keep documentation up to date with code changes