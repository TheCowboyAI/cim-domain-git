#!/bin/bash

echo "=== Checking for common code issues ==="

# Check for unused imports
echo -e "\n--- Checking for unused imports ---"
grep -r "^use " src/ | grep -v "//" | sort | uniq -c | sort -nr | head -10

# Check for TODO/FIXME comments
echo -e "\n--- Checking for TODO/FIXME comments ---"
grep -r "TODO\|FIXME" src/ --include="*.rs" || echo "No TODO/FIXME found"

# Check for unwrap() calls that should be handled
echo -e "\n--- Checking for unwrap() calls ---"
grep -r "\.unwrap()" src/ --include="*.rs" | grep -v "test" | grep -v "example" | head -10 || echo "No unwrap() in non-test code"

# Check for panic! calls
echo -e "\n--- Checking for panic! calls ---"
grep -r "panic!" src/ --include="*.rs" | grep -v "test" | grep -v "unimplemented!" || echo "No panic! in non-test code"

# Check for missing documentation
echo -e "\n--- Checking for missing documentation on public items ---"
grep -B1 "^pub " src/ --include="*.rs" | grep -v "///" | grep -v "^--" | grep "^pub " | head -10

# Check for #[allow] or #[warn] attributes
echo -e "\n--- Checking for allow/warn attributes ---"
grep -r "#\[allow\|#\[warn" src/ --include="*.rs" || echo "No allow/warn attributes found"

# Check for unused variables (pattern _var)
echo -e "\n--- Checking for unused variable patterns ---"
grep -r "_[a-zA-Z]" src/ --include="*.rs" | grep -v "phantom" | head -5 || echo "No obvious unused variables"

# Check formatting issues
echo -e "\n--- Running cargo fmt check ---"
cargo fmt -- --check 2>&1 | head -20 || echo "Format check requires working cargo"