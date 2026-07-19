## 2026-07-19 - Avoid O(N²) array lookups in nested loops with lazy O(log N) structures

**Learning:** `Vec::contains` inside nested loops over spatial areas creates an O(N²) bottleneck, but unconditionally cloning and sorting the vector ahead of the loop is an anti-pattern when the condition is usually bypassed (e.g., when most chunks are already spawned).

**Action:** Lazily initialize a sorted, deduplicated `Vec` using `Option::get_or_insert_with` only when the outer loop condition evaluates to true. Then use `binary_search` for fast O(log N) presence checks inside the loop to avoid allocations and sorting when unnecessary.
