#!/bin/sh
# Copyright 2025 Cowboy AI, LLC.

# Generate test coverage report

echo "=== Git Domain Test Coverage Report ==="
echo "Generated at: $(date)"
echo

# Count test functions
echo "=== Test Coverage Summary ==="
echo

echo "Module Test Files:"
find src -name "*test*.rs" -o -name "tests.rs" | sort | while read -r file; do
    count=$(rg "#\[test\]" "$file" | wc -l)
    echo "  $file: $count tests"
done

echo
echo "Test Modules by Directory:"
for dir in src/nats src/events src/aggregate src/value_objects src/handlers src/commands src/queries; do
    if [ -d "$dir" ]; then
        echo -n "  $dir: "
        test_count=$(find "$dir" -name "*test*.rs" -o -name "tests.rs" | xargs -I {} rg "#\[test\]" {} 2>/dev/null | wc -l)
        echo "$test_count tests"
    fi
done

echo
echo "=== Public API Coverage ==="
echo

# Count public functions without tests
echo "Public functions potentially missing tests:"
for file in src/**/*.rs; do
    if [[ "$file" == *"test"* ]] || [[ "$file" == *"/tests.rs" ]]; then
        continue
    fi
    
    pub_fns=$(rg "^pub fn|^pub async fn" "$file" 2>/dev/null | wc -l)
    if [ "$pub_fns" -gt 0 ]; then
        # Check if there's a test module or file
        base=$(basename "$file" .rs)
        dir=$(dirname "$file")
        has_tests=false
        
        if rg -q "#\[cfg\(test\)\]" "$file" || \
           [ -f "$dir/${base}_tests.rs" ] || \
           [ -f "$dir/tests.rs" ]; then
            has_tests=true
        fi
        
        if [ "$has_tests" = false ]; then
            echo "  $file: $pub_fns public functions"
        fi
    fi
done

echo
echo "=== Test Organization ==="
echo

echo "Unit test locations:"
rg -l "#\[cfg\(test\)\]" src --type rust | grep -v "test" | head -10

echo
echo "Integration test files:"
find tests -name "*.rs" 2>/dev/null | sort

echo
echo "=== Code Quality Checks ==="
echo

# Check for unwrap() in non-test code
echo "Unsafe unwrap() calls in non-test code:"
rg "\.unwrap\(\)" src --type rust | grep -v test | grep -v "tests.rs" | wc -l

# Check for println! in non-example code
echo "Debug println! statements:"
rg "println!" src --type rust | grep -v test | grep -v example | wc -l

# Check for TODO/FIXME
echo "TODO/FIXME comments:"
rg -i "todo|fixme" src --type rust | grep -v test | wc -l

echo
echo "=== Recommendations ==="
echo

if ! command -v cargo-tarpaulin &> /dev/null; then
    echo "- Install cargo-tarpaulin for detailed coverage: cargo install cargo-tarpaulin"
fi

echo "- Run tests with: cargo test"
echo "- Run tests with coverage: cargo tarpaulin --out Html"
echo "- Run clippy: cargo clippy -- -D warnings"
echo "- Check formatting: cargo fmt -- --check"