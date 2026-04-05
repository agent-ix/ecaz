# Review Request: Rescan Query Validation

Scope:
- `src/am/mod.rs`
- `src/lib.rs`

What changed:
- `amrescan` no longer immediately hard-errors.
- It now rejects index quals, requires exactly one ORDER BY query, requires a non-NULL/non-empty `real[]` query, validates query dimensions against index metadata, and records minimal scalar state in the scan opaque.
- `amgettuple` still hard-errors, so no tuples are returned yet.

Review focus:
- ORDER BY query validation semantics
- Scan-key decoding safety and SQL-surface behavior
- Whether the current state recording is the right narrow boundary before tuple production exists

Questions to answer:
- Are the current `amrescan` preconditions too strict or still too loose for the existing SQL surface?
- Is decoding the query from `ScanKey.sk_argument` safe as written for this stage?
- Are there missing tests around NULL queries, multiple order-bys, or index quals?

---

## Review Comments

Status at `0abf7d9`:
- Comments 1-6: not needed. These confirm the current `amrescan` validation contract rather than identifying new work for this stage.
- Comment 7 addressed by adding explicit regression coverage for NULL queries, empty queries, index quals, and multiple ORDER BY keys.
- Comment 8: not needed for now. Sticky `rescan_called` remains acceptable until real scan execution exists.

### 1. The validation sequence is correctly ordered and complete for the narrow scope

The checks in `amrescan` (lines 367-412) proceed in a logical order:
1. Null scan descriptor (line 367-369)
2. Index quals rejected (line 370-372) — correct since tqhnsw only supports ORDER BY
3. Exactly one ORDER BY required (line 373-375)
4. Null orderbys pointer (line 376-378)
5. SK_ISNULL flag check (line 381-383)
6. Datum decode to `Vec<f32>` (line 385-390)
7. Empty query rejection (line 391-393)
8. Dimension validation against metadata (line 395-402)

This is a thorough validation chain. Each check produces a distinct, descriptive error message.

### 2. `SK_ISNULL` check before datum access is essential and correctly placed

At line 381, checking `SK_ISNULL` before accessing `sk_argument` (line 385) prevents interpreting a null datum as a valid float array. Without this check, `from_polymorphic_datum` with a null datum could produce undefined behavior or a misleading error. Good.

### 3. Dimension check allows empty indexes (dimensions == 0)

At line 396, the condition `metadata.dimensions != 0 && query.len() != metadata.dimensions` skips validation when the index was built empty (dimensions == 0 in initial metadata). This means a scan on an empty index with any query dimension will pass validation. Since `amgettuple` will error regardless, this is safe for now. But when real scan execution is implemented, this needs to be revisited — either the first scan should set dimensions, or an empty index should return zero results without erroring.

### 4. The query vector is decoded and then dropped — no state leak, but also no preservation

At line 385-390, the query is decoded into a `Vec<f32>`, validated, then only `query.len()` is saved to opaque state (line 411-412). The actual query values are dropped at the end of the closure. This is correct for this scaffolding slice — the query will need to be stored in the opaque state when real scan execution is implemented, but saving it now would be premature.

### 5. `xs_recheck` and `xs_recheckorderby` are set to false — correct for exact distance AM

At lines 404-407, both recheck flags are set to false, and the orderby value/null arrays are nulled. This tells the executor that the AM's distance ordering is exact and doesn't need rechecking. For an inner-product based index, this is the right contract (the AM computes exact distances, not lossy approximations). The null pointer for `xs_orderbyvals` is fine since no tuples are returned yet.

### 6. `read_metadata_page` takes a shared lock inside `amrescan`

At line 395, `read_metadata_page` acquires `BUFFER_LOCK_SHARE` on the metadata page. This is appropriate — `amrescan` only needs to read dimensions for validation. The lock is released before the function returns (inside `read_metadata_page`). No lock ordering concern since no data page locks are held.

### 7. Test coverage

- `test_tqhnsw_rescan_scaffold_records_query_dimensions` (lib.rs:1519) — verifies happy path, correct dimensions stored
- `test_tqhnsw_rescan_scaffold_rejects_wrong_query_dimensions` (lib.rs:1548) — verifies dimension mismatch error

**Missing tests:**
- **NULL query:** The `SK_ISNULL` path (line 381-383) is not explicitly tested. The test helper `debug_rescan_query_dimensions` always passes a valid `Vec<f32>` datum. A test that sets `SK_ISNULL` flag on the `ScanKeyData` would exercise this path.
- **Empty query:** No test passes an empty `real[]` (zero-length vector). A `debug_rescan_query_dimensions(index_oid, vec![])` call should trigger the "must not be empty" error at line 391-393.
- **Index quals:** No test passes `nkeys != 0`. The test helper always passes `nkeys = 0`. A test that exercises the "does not support index quals" error would be good documentation.
- **Multiple ORDER BY:** No test passes `norderbys != 1`. The test helper always passes `norderbys = 1`.

Of these, the NULL query test is the most important since it validates a safety-critical check. The others are defensive checks that would need custom test helpers to exercise.

### 8. `rescan_called` flag is set but never reset

At line 410, `opaque.rescan_called = true` is set, but there's no code that resets it to `false`. In the PostgreSQL executor, `amrescan` can be called multiple times (e.g., for a nested loop inner scan). Each call overwrites the opaque state, which is correct — `query_dimensions` gets overwritten too. The `rescan_called` flag being "sticky true" is fine since it only gates `amgettuple`, and a second `amrescan` always re-validates. No issue.
