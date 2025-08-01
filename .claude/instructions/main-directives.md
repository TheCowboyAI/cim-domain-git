# Main Development Directives

## CRITICAL RULES - HIGHEST PRIORITY

### Filename Convention
**NEVER CREATE UPPERCASE FILENAMES**
- ALL filenames MUST be lowercase with underscores (snake_case)
- Examples: `readme.md`, `progress_report.md`, `event_store.rs`
- This applies to ALL file types without exception

### Date Handling
**NEVER generate dates - ALWAYS use system commands**
- Use `$(date -I)` for current date
- Use `$(git log -1 --format=%cd --date=short)` for git dates
- Never hardcode or guess dates

## Development Environment

### NixOS with direnv
- You are ALWAYS in a devshell
- Adjust shell commands for NixOS environment
- Use available MCP tools alongside built-in tools

### Git Requirements
- **MUST** `git add` new files for compilation
- Capture git hashes for completed work
- Commit before updating progress.json

## Core Development Principles

### 1. Assembly-First Development
- **ASSEMBLE existing cim-* modules** - don't build from scratch
- Create thin domain-specific extensions (e.g., cim-domain-mortgage)
- Each CIM targets ONE specific business domain
- Reuse infrastructure: identity, security, storage, workflow

### 2. Incremental Building
- Build in modules, one at a time
- Follow Test-Driven Development (TDD)
- Keep scope as small as possible

### 3. Single Responsibility Principle (SRP)
**EVERYTHING has ONE and ONLY ONE responsibility**
- Elements do one thing
- Strive to be the one irreducible way to be
- Use dependency injection over direct creation

### 4. Documentation Requirements
- ALWAYS document and justify actions
- Follow documentation in `/doc` organized by context
- Maintain progress graph in `/doc/progress/progress.json`

### 5. Progress Tracking
**UPDATE progress.json for:**
- Enhancements or extensions
- Completed features
- State transitions

**Definition of Done:**
1. DESIGNED
2. PLANNED
3. IMPLEMENTED
4. VERIFIED
5. TESTED
6. COMPLETE
7. DONE

### 6. Quality Standards
- Confirm operation before moving to next phase
- All features MUST work and pass tests
- Use continuous improvement
- Fix unused/incorrect APIs, don't delete

## Reference Hierarchy

1. Follow `.claude` rules
2. Apply CIM conversation model
3. Follow `/doc/design` specifications
4. Follow `/doc/plan` roadmap
5. Reference `/doc/research` for background

## Conflict Resolution

If there is a discrepancy:
1. STOP immediately
2. Ask for guidance
3. Do not proceed with assumptions

## Key Reminders

- Graph is single source of truth for progress
- Every feature must be fully tested
- Documentation is mandatory
- Small, focused changes are preferred
- Always verify before proceeding