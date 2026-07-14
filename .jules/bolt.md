## 2023-11-20 - Distance Calculation Bottleneck
**Learning:** Checking distances using `Vec3::distance` performs a square root operation. In `is_in_range`, this was happening for every combination of power pole and power consumer, running on every frame. Over multiple pairs, this creates a major drag on the update loop.
**Action:** Instead of `distance <= range`, use `distance_squared <= range * range`. This eliminates the expensive square root operation.

## 2023-11-20 - Redundant Query Evaluation in Systems
**Learning:** In `rebuild_power_grid`, `consumer_can_produce` (a moderately complex function involving iterating over `connected_consumers` and recipes) was called twice on the same query loop. The first pass calculated a total, the second pass applied the flag to the actual `PowerConsumer` components.
**Action:** Use a `Vec` to store the result of the first pass, and `zip` the results with the query `iter_mut` in the second pass. This cuts the function evaluations down by half for consumers, and is safe because Bevy `Query` iteration order is deterministic.
