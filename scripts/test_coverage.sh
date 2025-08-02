#!/bin/bash
# Copyright 2025 Cowboy AI, LLC.

# Script to analyze test coverage gaps

echo "=== Analyzing test coverage gaps ==="
echo

# Find all public functions and methods that might need tests
echo "=== Public functions in src/nats ==="
rg "^pub fn|^pub async fn" src/nats --type rust -A1 | grep -v "^--$"

echo
echo "=== Public structs without tests ==="
rg "^pub struct" src --type rust | while read -r line; do
    file=$(echo "$line" | cut -d':' -f1)
    struct_name=$(echo "$line" | sed 's/.*pub struct \([A-Za-z0-9_]*\).*/\1/')
    
    # Check if there's a test for this struct
    if ! rg "$struct_name" src --type rust | grep -q "test.*$struct_name\|$struct_name.*test"; then
        echo "$file: $struct_name"
    fi
done | head -20

echo
echo "=== Event types without tests ==="
rg "^pub struct.*Event\b" src/events --type rust | while read -r line; do
    event=$(echo "$line" | sed 's/.*pub struct \([A-Za-z0-9_]*\).*/\1/')
    if ! rg "$event" src/events --type rust | grep -q "#\[test\]" -A10; then
        echo "$event"
    fi
done

echo
echo "=== Value objects without validation tests ==="
rg "impl.*TryFrom.*for" src/value_objects --type rust | while read -r line; do
    vo=$(echo "$line" | sed 's/.*for \([A-Za-z0-9_]*\).*/\1/')
    if ! rg "test.*$vo.*invalid\|invalid.*$vo" src/value_objects --type rust | grep -q "#\[test\]" -B5; then
        echo "$vo needs invalid input tests"
    fi
done

echo
echo "=== NATS module test coverage ==="
for file in src/nats/*.rs; do
    if [[ "$file" == *"mod.rs" ]] || [[ "$file" == *"tests.rs" ]]; then
        continue
    fi
    
    module=$(basename "$file" .rs)
    echo -n "$module: "
    
    # Count public functions
    pub_count=$(rg "^pub fn|^pub async fn" "$file" --type rust | wc -l)
    
    # Check for test module or test file
    if rg "#\[cfg\(test\)\]" "$file" --type rust -q || [[ -f "src/nats/${module}_tests.rs" ]]; then
        echo "has tests (functions: $pub_count)"
    else
        echo "NO TESTS (functions: $pub_count)"
    fi
done