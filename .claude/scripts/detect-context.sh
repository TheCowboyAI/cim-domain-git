#!/bin/bash
# Detect CIM module context for Claude

set -euo pipefail

# Colors
BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}=== CIM Context Detection ===${NC}"

# Get current directory name
CURRENT_DIR=$(basename "$PWD")
REPO_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || echo "$PWD")
REPO_NAME=$(basename "$REPO_ROOT")

echo -e "\n${GREEN}Repository:${NC} $REPO_NAME"
echo -e "${GREEN}Current Directory:${NC} $PWD"

# Determine module type
if [[ "$REPO_NAME" == "cim" ]]; then
    echo -e "\n${YELLOW}Context: CIM Registry (Source of Truth)${NC}"
    echo "This is the central registry for all CIM modules."
    echo "Role: Guide development, maintain registry, provide standards"
elif [[ "$REPO_NAME" == "cim-start" ]]; then
    echo -e "\n${YELLOW}Context: CIM Template${NC}"
    echo "This is the starting template for new CIM implementations."
    echo "Role: Provide boilerplate for new domains"
elif [[ "$REPO_NAME" =~ ^cim-domain- ]]; then
    echo -e "\n${YELLOW}Context: Domain Implementation${NC}"
    echo "This is a domain-specific CIM implementation."
    DOMAIN=${REPO_NAME#cim-domain-}
    echo "Domain: $DOMAIN"
    echo "Role: Assemble modules for $DOMAIN business logic"
elif [[ "$REPO_NAME" =~ ^cim- ]]; then
    echo -e "\n${YELLOW}Context: Core CIM Module${NC}"
    echo "This is a reusable CIM module."
    echo "Role: Provide specific functionality for CIM assembly"
else
    echo -e "\n${YELLOW}Context: Unknown${NC}"
    echo "This doesn't appear to be a CIM module."
fi

# Check for module metadata
echo -e "\n${GREEN}Module Metadata:${NC}"
if [ -f "cim.yaml" ]; then
    echo "cim.yaml found:"
    grep -E "^(name|type|version|status):" cim.yaml | sed 's/^/  /'
else
    echo "No cim.yaml found"
fi

if [ -f "Cargo.toml" ]; then
    echo -e "\nCargo.toml found:"
    grep -E "^name = |^version = " Cargo.toml | head -2 | sed 's/^/  /'
fi

# Check module status in registry
echo -e "\n${GREEN}Registry Status:${NC}"
if command -v curl >/dev/null 2>&1 && command -v jq >/dev/null 2>&1; then
    STATUS=$(curl -s https://raw.githubusercontent.com/thecowboyai/cim/main/registry/modules-graph.json | \
              jq -r --arg mod "$REPO_NAME" '.graph.nodes[$mod].status // "not found"')
    VERSION=$(curl -s https://raw.githubusercontent.com/thecowboyai/cim/main/registry/modules-graph.json | \
              jq -r --arg mod "$REPO_NAME" '.graph.nodes[$mod].version.current // "unknown"')
    
    echo "Module Status: $STATUS"
    echo "Registry Version: $VERSION"
    
    if [ "$STATUS" = "production" ]; then
        echo -e "${GREEN}✓ This module is production-ready${NC}"
    elif [ "$STATUS" = "development" ]; then
        echo -e "${YELLOW}⚠ This module is in development${NC}"
    fi
else
    echo "Unable to query registry (curl/jq not available)"
fi

# Check for .claude directory
echo -e "\n${GREEN}Claude Configuration:${NC}"
if [ -d ".claude" ]; then
    echo "✓ .claude directory found"
    echo "Available instructions:"
    find .claude -name "*.md" -type f | sed 's/^/  - /'
else
    echo "✗ No .claude directory found"
    echo "Run: curl -L https://github.com/thecowboyai/cim/archive/main.tar.gz | tar -xz --strip=1 cim-main/.claude"
fi

echo -e "\n${BLUE}=== End Context Detection ===${NC}"