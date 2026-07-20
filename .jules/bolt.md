## 2025-02-18 - Avoiding Dynamic HashSet Creation in Nested Loops
**Learning:** Building a dynamic `HashSet` inside nested loops or for every iteration of a Bevy query (like in `cultivator_ai`) causes severe allocation overhead per frame and O(N²) scaling.
**Action:** Use a lazily initialized `Vec`, populate it once per frame (using `Option::get_or_insert_with`), sort it, dedup it, and use `binary_search` for fast O(log N) presence checks to drastically reduce memory allocations and improve CPU cache coherency.
