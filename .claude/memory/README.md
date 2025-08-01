# CIM Project Memory System

## Overview
This memory system integrates with `/doc/progress/progress.json` which serves as our event store and system of record for all project state.

## Key Principles

1. **progress.json is the Source of Truth**
   - All task states are stored in progress.json
   - Claude must update progress.json after completing tasks
   - Context switching decisions based on progress.json state
   - Git hashes capture immutable state references

2. **Memory Structure**
   - `state.md` - Current working context derived from progress.json
   - `context-map.md` - Maps progress nodes to code contexts
   - `update-protocol.md` - Instructions for updating progress.json
   - `git-state-tracking.md` - Git hash integration for versioning

## Usage
Before starting work:
1. Read `/doc/progress/progress.json`
2. Check current node status and dependencies
3. Update working memory based on active nodes
4. Track changes for progress.json updates

After completing work:
1. Commit all changes to git
2. Capture git hash for completed nodes
3. Update progress.json with new nodes/edges and git hash
4. Update completion percentages
5. Add to recent_changes array
6. Update next_priorities based on completed work