# Simba Patch for Nightly Rust Compatibility

## Problem
The `simba` crate version 0.9.1 is incompatible with the latest Rust nightly (2026-02-03) because it's missing the `std::simd::Select` trait import, which is required for the `select()` method on `Mask` types.

## Solution
We've created a local patched copy of simba in `vendor/simba-0.9.1` with the following fix:

**File:** `vendor/simba-0.9.1/src/simd/portable_simd_impl.rs`

**Change:** Added `Select` to the `std::simd` import list:
```rust
simd::{
    self as portable_simd, cmp::SimdOrd, cmp::SimdPartialEq,
    cmp::SimdPartialOrd as PortableSimdPartialOrd, num::SimdFloat, num::SimdInt, num::SimdUint,
    Select, StdFloat,  // <-- Added Select here
},
```

## Current Status
✅ **Simba patch applied successfully** - simba now compiles without errors.

⚠️ **Remaining issue:** The `aosoa` crate has compatibility issues with `core::simd::LaneCount` and `core::simd::SupportedLaneCount` types in the latest nightly. These types may have moved or require a different import path.

## Next Steps
1. Check if `LaneCount` and `SupportedLaneCount` are available in `std::simd` instead of `core::simd`
2. Or use an older nightly version that's compatible with these types
3. Or update `aosoa` to use the correct import path for these types

## Applying the Patch
If you need to re-apply the patch after updating dependencies:

```bash
cd /Users/danieldonovan-achi/Desktop/UAV/uas
python3 << 'EOF'
import os
import re

simba_file = "vendor/simba-0.9.1/src/simd/portable_simd_impl.rs"

if not os.path.exists(simba_file):
    print(f"Error: {simba_file} not found")
    exit(1)

with open(simba_file, 'r') as f:
    content = f.read()

if 'Select,' in content and 'simd::{' in content:
    if 'Select' not in content.split('simd::{')[1].split('}')[0]:
        # Need to add Select
        content = content.replace(
            'simd::{',
            'simd::{Select, ',
            1
        )
        with open(simba_file, 'w') as f:
            f.write(content)
        print("✓ Patched simba")
    else:
        print("Already patched")
else:
    print("Could not find insertion point")
EOF
```
