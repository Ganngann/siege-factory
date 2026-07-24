## 2024-07-24 - Cultivator loop nested set optimization
**Learning:** `find_plantable_tile_spiral` used to unconditionally allocate a `HashSet` dynamically inside the cultivator loop over all crops, leading to O(N * M) allocations and operations per frame where N is the number of cultivators and M is the number of crops.
**Action:** Replaced dynamic `HashSet` allocations inside tight AI loops by passing down a lazily initialized `Option<Vec>` which is populated and deduplicated on first access, allowing subsequent calls to benefit from O(log M) binary search checks without allocating.
