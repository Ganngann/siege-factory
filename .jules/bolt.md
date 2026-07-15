## 2024-03-24 - Initial Setup
**Learning:** Found some needlessly copied allocations in `ChunkGrid::ensure_chunk_mut` passing a `Vec` around instead of a slice, and needless range loops. Need to check if there are any other low hanging performance issues.
**Action:** Be mindful of clones in tight loops or map generation algorithms.
