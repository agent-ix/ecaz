# Review Request: Encode Dimension Boundary

Scope:
- `src/lib.rs`

What changed:
- `encode_to_tqvector` now routes through an internal helper that validates the embedding length before packing.
- The helper rejects dimensions above `65535` instead of silently truncating the persisted `u16` dimension field.
- Added direct unit coverage for the oversized-dimension failure path.

Review focus:
- Whether the new helper boundary is the right place for the persisted-dimension check
- Whether the error path is explicit and stable enough for the SQL-facing wrapper
- Whether the current unit coverage is sufficient for this narrow correctness fix

Questions to answer:
- Is the explicit `65535` boundary the right contract for the public SQL function at this stage?
- Should there also be SQL-surface regression coverage for the oversized-dimension error, or is unit coverage enough here?
- Is there any related `usize -> u16` conversion in tqvector encoding that should be brought under the same helper boundary now?
