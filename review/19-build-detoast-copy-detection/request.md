# Review Request: Build Detoast Copy Detection

Scope:
- `src/am/mod.rs`

What changed:
- `build_heap_tuple` and `build_heap_tuple_with_source` no longer infer detoast ownership from varlena header flags on the original datum.
- Both paths now compare the original datum pointer with the return value from `pg_detoast_datum_packed` and only `pfree` when PostgreSQL returned an allocated copy.
- This change was prompted by the external review bundle under `review/external/2026-04-05-claude-opus/`.

Review focus:
- Whether pointer comparison is the correct ownership test for these two build-time detoast paths
- Whether the updated `pfree` boundary is safe for both plain build and `build_source_column` build
- Whether existing build coverage is sufficient for this narrow memory-safety fix

Questions to answer:
- Is there any remaining detoast ownership edge case in the build path after switching to pointer comparison?
- Should this detoast pattern be factored into a shared helper now, or is keeping it inline in the two call sites clearer for the current stage?
- Is there a missing regression test that specifically exercises a toasted tqvector datum during build, or is current large-datum build coverage enough?
