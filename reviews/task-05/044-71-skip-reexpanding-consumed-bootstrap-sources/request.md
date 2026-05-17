# Request: Skip Re-Expanding Consumed Bootstrap Sources

Commit: `7b62152`

Summary:
- Avoids rereading the consumed frontier source during bootstrap refill when that source was already expanded during initial seeding or a prior refill step.
- Keeps bounded frontier top-up behavior unchanged otherwise: if the consumed source is still unexpanded, it expands first; if not, refill proceeds from other remaining unexpanded frontier candidates.

Files:
- `src/am/scan.rs`

Why this matters:
- This removes redundant graph-adjacency work from the bounded traversal scaffold without changing SQL-visible scan semantics.
- It also makes `expanded_source_tids` a more coherent execution contract: once a source has been expanded in the current scan, consume/refill no longer re-expands it opportunistically.

Review focus:
- Whether the consumed-source skip boundary is correct relative to `expanded_source_tids`
- Whether the helper still tops up bounded frontier width correctly when the consumed source is already expanded
- Whether the new unit coverage is sufficient for this execution-only optimization seam
