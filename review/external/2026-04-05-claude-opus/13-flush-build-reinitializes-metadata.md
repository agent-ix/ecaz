# Review: flush_build_state Calls initialize_metadata_page Twice

**File:** `src/am/mod.rs:132-177` and `1701-1764`
**Severity:** Low (correct but wasteful)
**Category:** Correctness / clarity

## Finding

During `ambuild`:

1. Line 141: `initialize_metadata_page(index_relation, state.initial_metadata())` -- writes a blank metadata page with zeros for dimensions/bits/seed
2. Line 160: `flush_build_state(index_relation, &state)` which calls...
3. Line 1751: `initialize_metadata_page(index_relation, ...)` again with the real metadata (dimensions, bits, entry_point, etc.)

The second call overwrites the first. The first initialization is needed for `ambuildempty` (line 183) and for the case where no tuples are scanned (line 157-158 skips `flush_build_state`). For non-empty builds, the first `initialize_metadata_page` creates a WAL record that is immediately superseded.

This means the metadata page is written with a full-image WAL record twice during every non-empty build. While not a correctness issue, it's unnecessary WAL amplification.

## Recommendation

Consider restructuring so that `flush_build_state` calls `update_metadata_page` instead of `initialize_metadata_page`, since the metadata page already exists from the first initialization. Or, move the first initialization inside the `heap_tuples.is_empty()` branch.

## Action Required

Low priority. Consider refactoring to avoid the double metadata page write during build.
