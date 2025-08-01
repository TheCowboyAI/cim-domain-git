# Setting Up Claude Instructions in CIM Modules

## Quick Setup

### Download .claude to Any Module
```bash
# From within any cim-* module directory
curl -L https://github.com/thecowboyai/cim/archive/main.tar.gz | \
  tar -xz --strip=1 cim-main/.claude

# Make scripts executable
chmod +x .claude/scripts/*.sh

# Verify setup
.claude/scripts/detect-context.sh
```

### Alternative: Git Sparse Checkout
```bash
# From within any cim-* module directory
git init temp-clone
cd temp-clone
git remote add origin https://github.com/thecowboyai/cim.git
git sparse-checkout init --cone
git sparse-checkout set .claude
git pull origin main
cd ..
cp -r temp-clone/.claude .
rm -rf temp-clone
```

## What You Get

The `.claude` directory contains:

### Core Instructions
- **CLAUDE.md** - Primary CIM instructions
- **CLAUDE_MODULE_TEMPLATE.md** - Context-aware module instructions
- **INDEX.md** - Complete navigation guide

### Contextual Directories
- **instructions/** - Operational guidelines
- **patterns/** - Architectural patterns
- **standards/** - Technical standards
- **contexts/** - NATS architecture contexts
- **memory/** - Project state tracking
- **templates/** - Implementation templates
- **workflows/** - Step-by-step processes
- **scripts/** - Utility scripts including context detection

## Module-Specific Customization

After downloading `.claude`, customize for your module:

### 1. Create Module-Specific CLAUDE.md
```bash
cat > .claude/CLAUDE.md << 'EOF'
# Claude Instructions for $(basename $PWD)

This module uses the standard CIM instructions. Additionally:

## Module Purpose
[Describe this module's single responsibility]

## Key APIs
[List main interfaces/functions]

## Dependencies
[List other CIM modules this depends on]

## Module-Specific Patterns
[Any patterns unique to this module]

---
For general CIM instructions, see other files in this directory.
EOF
```

### 2. Run Context Detection
```bash
.claude/scripts/detect-context.sh
```

This will show:
- What type of module you're in
- Current registry status
- Available instructions

### 3. Add Module Metadata
Create `cim.yaml` if it doesn't exist:
```yaml
module:
  name: "cim-your-module"
  type: "core|domain|integration|utility"
  version: "0.1.0"
  description: "Module purpose"
  status: "development"
  dependencies:
    - cim-domain: "^1.0"
```

## Using Claude with Modules

### Starting a Session
When starting work in any module:
1. Claude reads `.claude/CLAUDE.md` for context
2. Detects module type from repository name
3. Applies appropriate patterns and standards

### Context Awareness
Claude will understand:
- Whether you're in the registry or a module
- If the module is production-ready
- What dependencies are available
- Appropriate patterns to follow

### Key Commands for Claude

In any module, you can ask Claude to:
- "Check our production readiness"
- "Show our dependencies"
- "Update our registry status"
- "Generate NATS subjects for this module"
- "Create event definitions following CIM patterns"

## Keeping Instructions Updated

### Manual Update
```bash
# Remove old .claude directory
rm -rf .claude

# Download latest
curl -L https://github.com/thecowboyai/cim/archive/main.tar.gz | \
  tar -xz --strip=1 cim-main/.claude
```

### Automated Update Check
```bash
# Add to CI/CD
LATEST_CLAUDE=$(curl -s https://api.github.com/repos/thecowboyai/cim/commits/main | jq -r '.sha')
LOCAL_CLAUDE=$(cat .claude/.version 2>/dev/null || echo "unknown")

if [ "$LATEST_CLAUDE" != "$LOCAL_CLAUDE" ]; then
  echo "Claude instructions are outdated. Run update script."
fi
```

## Module Development with Claude

### For New Modules
1. Clone `cim-start` template
2. `.claude` directory included
3. Customize for your domain
4. Claude understands the context

### For Existing Modules
1. Download `.claude` directory
2. Run context detection
3. Add module metadata
4. Claude adapts to module type

### Production Readiness
Claude knows only 4 modules are production-ready:
- Will guide you through production checklist
- Suggests improvements based on standards
- Helps update registry when ready

## Summary

Every CIM module should have the same `.claude` directory, providing:
- Consistent instructions across all modules
- Context awareness (registry vs module vs domain)
- Production readiness guidance
- Registry integration
- NATS patterns and event-driven architecture

This ensures Claude can effectively work on any part of the CIM ecosystem while understanding the specific context of each module.