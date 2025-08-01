# CIM Implementation Workflow

## Workflow Overview
This workflow ensures proper sequencing and progress tracking for CIM implementation.

## Step 1: Progress Check
```bash
# Always start by reading current state
cat /git/thecowboyai/cim/doc/progress/progress.json

# Check for:
- Active (IN_PROGRESS) nodes
- Blocked nodes needing attention
- Dependencies for next work
```

## Step 2: Context Loading
Based on active nodes, load appropriate context:
- Infrastructure → `.claude/contexts/cluster.md` or `leaf.md`
- Client work → `.claude/contexts/client.md`
- Service work → `.claude/contexts/leaf.md`

## Step 3: Implementation
Follow the appropriate template:
- Client: `.claude/templates/client-implementation.md`
- Service: `.claude/templates/service-creation.md`
- Infrastructure: `.claude/templates/infrastructure-setup.md`

## Step 4: Testing
- Unit tests for new code
- Integration tests for NATS communication
- Update test coverage metrics

## Step 5: Progress Update
```bash
# First commit your changes
git add -A
git commit -m "feat: implement <feature description>"

# Capture the git hash and date
GIT_HASH=$(git rev-parse HEAD)
CURRENT_DATE=$(date -I)
```

```json
// Add to progress.json
{
  "nodes": [{
    "id": "your-new-node",
    "type": "implementation",
    "status": "COMPLETE",
    "git_hash": "$GIT_HASH",
    "completed": "$CURRENT_DATE",
    "artifacts": ["files/created.go"]
  }],
  "recent_changes": [{
    "date": "$CURRENT_DATE",
    "description": "What was accomplished",
    "git_hash": "$GIT_HASH"
  }]
}
```

## Step 6: Next Actions
- Update `next_priorities` in progress.json
- Clear completed todos
- Plan next implementation phase

## Critical Reminders
1. **ALWAYS** update progress.json after significant work
2. Track all created files in artifacts
3. Update completion percentages
4. Maintain graph relationships with proper edges