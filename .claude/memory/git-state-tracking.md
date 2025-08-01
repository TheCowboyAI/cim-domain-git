# Git State Tracking Protocol

## Purpose
Track git commit hashes in progress.json to enable:
- Reproducible builds with Nix
- Version pinning for releases
- State recovery and rollback
- Audit trail of changes

## Git Hash Integration

### When to Capture Git Hash
1. **On Node Completion**: When marking a node as COMPLETE
2. **At Milestones**: When reaching significant project milestones
3. **Before Breaking Changes**: Capture last stable state
4. **For Releases**: Tag specific versions

### Capture Commands
```bash
# Get current commit hash
git rev-parse HEAD

# Get short hash (first 8 chars)
git rev-parse --short HEAD

# Ensure clean working tree before capturing
git status --porcelain
```

### Progress.json Schema Extension
```json
{
  "nodes": [{
    "id": "node-id",
    "git_hash": "abc123def456",
    "git_tag": "v0.1.0",  // Optional: if tagged
    "git_branch": "main",  // Branch where completed
    "working_tree_clean": true  // Was working tree clean
  }],
  "versions": {
    "v0.1.0": {
      "git_hash": "abc123def456",
      "date": "$(date -I)",  // ALWAYS use system date
      "description": "Initial NATS infrastructure",
      "nix_compatible": true
    }
  }
}
```

## Nix Integration Pattern

### Using Git Hashes in Nix Inputs
```nix
{
  inputs = {
    cim-stable = {
      url = "git+file:///git/thecowboyai/cim?rev=abc123def456";
      flake = false;
    };
  };
}
```

### Version Management
1. Complete implementation milestone
2. Ensure all tests pass
3. Commit all changes
4. Capture git hash in progress.json
5. Tag if appropriate
6. Update Nix inputs with hash

## State Recovery Workflow

### Finding Working States
```bash
# List all completed nodes with git hashes
jq '.graph.nodes[] | select(.status == "COMPLETE" and .git_hash != null) | {id, git_hash, completed}' progress.json

# Checkout specific state
git checkout <git_hash>
```

### Creating Checkpoints
```bash
# Before major changes
git add -A
git commit -m "Checkpoint: Before <description>"
CHECKPOINT_HASH=$(git rev-parse HEAD)

# Add to progress.json
{
  "checkpoints": [{
    "hash": "$CHECKPOINT_HASH",
    "date": "$(date -I)",
    "reason": "Before major refactoring"
  }]
}
```

## Best Practices

1. **Clean Commits**: Only capture hashes of clean, tested states
2. **Descriptive Messages**: Use clear commit messages for traceability
3. **Regular Tagging**: Tag significant milestones for easy reference
4. **Test Before Capture**: Ensure all tests pass before marking complete
5. **Document Dependencies**: Note any external dependencies at that hash

## Example Workflow

```bash
# 1. Complete implementation
vim client/nats_client.go

# 2. Run tests
go test ./client/...

# 3. Commit changes
git add -A
git commit -m "feat: implement NATS client with reconnection logic"

# 4. Capture hash
CURRENT_HASH=$(git rev-parse HEAD)

# 5. Update progress.json
# Add git_hash: "$CURRENT_HASH" to completed node

# 6. Tag if milestone
git tag -a v0.1.0 -m "First working NATS client"
```

## Integration with CI/CD

### Automated Hash Capture
```yaml
# .github/workflows/progress-update.yml
- name: Update progress.json with git hash
  run: |
    HASH=$(git rev-parse HEAD)
    jq --arg hash "$HASH" '.last_stable_hash = $hash' progress.json > tmp.json
    mv tmp.json progress.json
    git add progress.json
    git commit -m "chore: update progress.json with hash $HASH"
```