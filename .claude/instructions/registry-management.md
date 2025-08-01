# CIM Registry Management

## Overview

The `thecowboyai/cim` repository serves as the **source of truth** for all CIM modules and implementations. It maintains a registry that tracks all official modules and domain implementations across the ecosystem.

## Repository Conventions

### Naming Convention
All CIM repositories in `thecowboyai` organization follow strict naming:
- **Registry**: `cim` (this repository only)
- **Modules**: `cim-*` (e.g., `cim-domain`, `cim-security`)
- **Domains**: `cim-domain-*` (e.g., `cim-domain-mortgage`)

### GitHub Topics
Every CIM repository MUST have:
- `cim` - Required for all CIM-related repos
- Additional topics based on type:
  - `cim-module` - For reusable modules
  - `cim-domain` - For domain implementations
  - `cim-private` - For private implementations
  - Category topics: `core`, `security`, `storage`, etc.

## Registry Structure

```
cim/
├── registry/
│   ├── modules.yaml          # All official modules
│   ├── private-domains.yaml  # Private implementations
│   ├── status.json          # Current health status
│   └── changelog.md         # Auto-generated changes
├── scripts/
│   └── query-modules.sh     # Query utilities
└── .github/workflows/
    └── update-registry.yml  # Automated updates
```

## Event-Driven Updates

### Module Repository Setup
Each `cim-*` repository must include:

1. **GitHub Action** (`.github/workflows/notify-registry.yml`):
```yaml
name: Notify CIM Registry
on:
  push:
    branches: [main]
jobs:
  notify:
    # Sends event to CIM registry
```

2. **Metadata File** (`cim.yaml`):
```yaml
module:
  name: "cim-domain-mortgage"
  type: "domain"
  version: "0.1.0"
  dependencies:
    - cim-domain: "^0.2"
    - cim-security: "^0.1"
```

### Update Flow
1. Module repository receives commit
2. GitHub Action publishes NATS event
3. CIM registry receives notification
4. Registry workflow queries GitHub
5. Updates registry files automatically
6. Commits changes to registry

## Querying the Registry

### Using the Query Script
```bash
# List all modules
./scripts/query-modules.sh --list

# Filter by type
./scripts/query-modules.sh --type domain

# Search modules
./scripts/query-modules.sh --search workflow

# Show private domains
./scripts/query-modules.sh --private
```

### Direct API Access
```bash
# Get modules via curl
curl https://raw.githubusercontent.com/thecowboyai/cim/main/registry/modules.yaml

# Use GitHub API
gh api repos/thecowboyai/cim/contents/registry/modules.yaml
```

## Module Categories

- **core**: Essential infrastructure
- **domain**: Business domain modules
- **security**: Security-related modules
- **storage**: Data persistence modules
- **graph**: Graph-related modules
- **ai**: AI and knowledge modules
- **edge**: Edge computing modules
- **technical**: Technical integrations

## Private Domain Registry

Private implementations are tracked separately:
- Not published to public registry
- Tracked in `private-domains.yaml`
- Includes owner and deployment status
- Helps coordinate between teams

## Adding a New Module

1. **Create Repository**
   - Name: `cim-<module-name>`
   - Add topics: `cim`, `cim-module`, category
   
2. **Add Metadata**
   - Create `cim.yaml` with module info
   - Include in README.md
   
3. **Setup Notifications**
   - Add GitHub Action for updates
   - Configure NATS publishing
   
4. **Registry Updates**
   - Automatic within 6 hours
   - Or trigger manually via workflow

## Best Practices

1. **Consistent Naming**: Always follow conventions
2. **Complete Metadata**: Fill all fields in cim.yaml
3. **Proper Topics**: Use correct GitHub topics
4. **Version Tags**: Tag releases properly
5. **Documentation**: Keep module docs updated

## Registry Maintenance

The registry self-maintains through:
- Scheduled updates every 6 hours
- Event-driven updates on changes
- Health checks and status monitoring
- Automatic changelog generation

## Integration with cim-start

When cloning `cim-start` for new domains:
1. The template includes registry notification
2. Automatically registers as private domain
3. Can be promoted to public module later
4. Maintains connection to registry