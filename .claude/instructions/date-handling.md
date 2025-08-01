# Date Handling Instructions

## CRITICAL RULE
**NEVER generate or guess dates. ALWAYS use system commands.**

## Approved Date Sources

### 1. System Date Commands
```bash
# ISO format (preferred for JSON)
date -I                    # 2025-01-30
date +%Y-%m-%d            # 2025-01-30

# With time
date -Iseconds            # 2025-01-30T14:23:45-05:00
date +%Y-%m-%dT%H:%M:%S   # 2025-01-30T14:23:45
```

### 2. Git Commit Dates
```bash
# Last commit date
git log -1 --format=%cd --date=short         # 2025-01-30
git log -1 --format=%cd --date=iso           # 2025-01-30 14:23:45 -0500

# Specific commit date
git show --format=%cd --date=short <hash>    # 2025-01-30
```

### 3. File Timestamps
```bash
# File modification time
stat -c %y file.txt | cut -d' ' -f1          # 2025-01-30
date -r file.txt +%Y-%m-%d                   # 2025-01-30 (macOS/BSD)
```

## Usage Examples

### Updating progress.json
```bash
# ALWAYS capture date first
CURRENT_DATE=$(date -I)

# Then use in JSON updates
jq --arg date "$CURRENT_DATE" '.last_updated = $date' progress.json
```

### Creating Timestamped Entries
```bash
# For new nodes
NODE_DATE=$(date -I)
cat <<EOF
{
  "created": "$NODE_DATE",
  "status": "IN_PROGRESS"
}
EOF
```

### Git Operations with Dates
```bash
# Tag with date
TAG_DATE=$(date -I)
git tag -a "release-$TAG_DATE" -m "Release on $TAG_DATE"

# Commit with date reference
COMMIT_DATE=$(date -I)
git commit -m "chore: daily update $COMMIT_DATE"
```

## Common Mistakes to Avoid

❌ **WRONG**: Using hardcoded dates
```json
{
  "date": "2025-01-30"  // Never hardcode!
}
```

✅ **CORRECT**: Using system date
```bash
DATE=$(date -I)
echo "{\"date\": \"$DATE\"}"
```

❌ **WRONG**: Guessing future dates
```json
{
  "expected_completion": "2025-02-15"  // Never guess!
}
```

✅ **CORRECT**: Calculate from system date
```bash
FUTURE_DATE=$(date -I -d "+2 weeks")
echo "{\"expected_completion\": \"$FUTURE_DATE\"}"
```

## Template for Date Capture
```bash
#!/bin/bash
# Always start with date capture
CURRENT_DATE=$(date -I)
CURRENT_DATETIME=$(date -Iseconds)
GIT_COMMIT_DATE=$(git log -1 --format=%cd --date=short)

# Use captured dates in operations
echo "Starting work on $CURRENT_DATE"
echo "Last commit was on $GIT_COMMIT_DATE"
```