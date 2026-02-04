#!/bin/bash
# Script to patch simba crate for compatibility with latest nightly Rust
# This fixes the missing std::simd::Select trait import

set -e

echo "Searching for simba source in Cargo registry..."

# Find simba source directory
SIMBA_DIR=$(find ~/.cargo/registry/src -name "simba-0.9.1" -type d 2>/dev/null | head -1)

if [ -z "$SIMBA_DIR" ]; then
    echo "Error: simba-0.9.1 not found in Cargo registry"
    echo "Please run 'cargo fetch' or 'cargo build' first to download dependencies"
    exit 1
fi

SIMBA_FILE="$SIMBA_DIR/src/simd/portable_simd_impl.rs"

if [ ! -f "$SIMBA_FILE" ]; then
    echo "Error: $SIMBA_FILE not found"
    exit 1
fi

# Check if already patched
if grep -q "use std::simd::Select" "$SIMBA_FILE"; then
    echo "simba is already patched"
    exit 0
fi

echo "Patching simba at: $SIMBA_FILE"

# Create backup
cp "$SIMBA_FILE" "$SIMBA_FILE.bak"

# Use Python for reliable patching (more portable than sed)
python3 << 'PYTHON_SCRIPT'
import sys
import re

file_path = sys.argv[1]

with open(file_path, 'r') as f:
    content = f.read()

# Check if already patched
if 'use std::simd::Select;' in content:
    print("Already patched")
    sys.exit(0)

# Find the first occurrence of "use std::simd::" and add Select import before it
# Look for the pattern and insert before the first match
pattern = r'(use std::simd::[^;]+;)'
match = re.search(pattern, content)

if match:
    # Insert the Select import before the first std::simd use
    insert_pos = match.start()
    new_content = content[:insert_pos] + 'use std::simd::Select;\n' + content[insert_pos:]
    
    with open(file_path, 'w') as f:
        f.write(new_content)
    print("✓ Successfully patched simba")
else:
    # Fallback: add after any existing use statements at the top
    # Find the first non-use, non-comment, non-blank line
    lines = content.split('\n')
    insert_pos = 0
    for i, line in enumerate(lines):
        stripped = line.strip()
        if stripped and not stripped.startswith('//') and not stripped.startswith('use '):
            insert_pos = i
            break
    
    lines.insert(insert_pos, 'use std::simd::Select;')
    new_content = '\n'.join(lines)
    
    with open(file_path, 'w') as f:
        f.write(new_content)
    print("✓ Successfully patched simba (fallback method)")

PYTHON_SCRIPT
"$SIMBA_FILE"

# Verify patch
if grep -q "use std::simd::Select" "$SIMBA_FILE"; then
    echo "✓ Patch verified successfully"
else
    echo "✗ Failed to patch simba, restoring backup"
    mv "$SIMBA_FILE.bak" "$SIMBA_FILE"
    exit 1
fi

echo "Done! You can now build the project."
