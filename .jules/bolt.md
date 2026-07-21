## 2024-05-19 - Removed HashSet allocation bottleneck in cultivator AI

**Learning:** Creating a `HashSet` dynamically inside a highly repeated function like `find_plantable_tile_spiral` causes an O(N²) allocation bottleneck. This is because for every valid unreserved tile checked, the entire list of `crops` was re-collected into a `HashSet`.

**Action:** Replaced the `HashSet` creation with a lazily-initialized sorted `Vec` and `binary_search`. This shifts the complexity from O(N) allocation per iteration to O(N log N) sorting once, plus O(log N) lookup per iteration, and prevents unnecessary allocations entirely when short-circuiting.
