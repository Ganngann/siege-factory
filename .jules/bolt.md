## 2025-01-20 - Avoid dynamic HashSet creation in nested AI loops
**Learning:** Creating a `HashSet` dynamically inside a nested loop (such as for spatial queries or AI pathfinding) causes significant allocation overhead and O(N^2) complexity, leading to severe frame rate drops.
**Action:** Replace the dynamic `HashSet` with a lazily-initialized sorted `Vec`. Guard it with `Option::get_or_insert_with` and populate, sort, and deduplicate it only once per frame if needed. Use `binary_search` for fast O(log N) presence checks instead of `.contains()`.
