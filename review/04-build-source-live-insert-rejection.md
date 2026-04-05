# Review Request: Rejecting Live Inserts For `build_source_column`

Scope:
- `src/am/mod.rs`
- `src/lib.rs`
- `sql/bootstrap.sql`

What changed:
- Index builds may still use a configured raw `real[]` source column for graph construction.
- Live `aminsert` now rejects such indexes instead of trying to mix raw-source build semantics with code-byte-only insert semantics.

Review focus:
- Whether this restriction is the right current boundary
- Error-path clarity and SQL-surface behavior
- Any overlooked cases where the AM could still accept an unsupported live insert for a source-column index

Questions to answer:
- Is there any hole in the current rejection path?
- Is there a smaller or clearer SQL-surface restriction that would better reflect current capability?
- Are there missing tests around reloptions or index definitions here?

---

## Review Comments

Status at `9444d4b`:
- Questions above: closed for this stage as skipped/not needed.
- Comments 1-6: not needed for now. The review found the current rejection boundary correct and sufficiently covered for the current capability set.

### 1. The rejection is correctly placed and uses the right detection mechanism

At lines 206-210, the check reads `options.build_source_column` via `relation_options()`, which parses the index's `reloptions` from `pg_class`. If `build_source_column` is set to any non-empty string, the insert is rejected with a clear error message that names the specific column. This is the right approach — it checks the index definition, not the incoming data.

### 2. No hole in the rejection path

The `build_source_column` check is the **first** thing after decoding the tuple and reading options (line 206), before any metadata validation or page writes. There's no code path that could bypass it — the function returns `false` or errors for every branch, and the source column check is unconditional.

### 3. The restriction is the right current boundary

Since the build path uses `build_source_column` to construct HNSW graph edges using raw float vectors (which aren't available at insert time), allowing live inserts would produce disconnected nodes that were never considered during graph construction. Rejecting is strictly correct. The error message clearly communicates this.

### 4. `relation_options` correctly handles the reloption parsing

The reloption is registered as a string via `add_local_string_reloption` (line 320-329) with a `None` validator. The parsing in `relation_options` (line 429-435) correctly strips the `build_source_column=` prefix and rejects empty values. One minor note: if PostgreSQL ever stores the reloption with different quoting or casing, the `strip_prefix` approach could miss it — but PostgreSQL normalizes reloption strings to lowercase unquoted form, so this is safe in practice.

### 5. Test coverage is good

`test_tqhnsw_insert_rejects_build_source_column_index` (lib.rs:1388) creates an index with `build_source_column = 'source'`, then attempts a live insert and asserts it panics with the expected message. This covers the happy path of the rejection.

**No missing tests** for the current scope. The build-path tests already cover `build_source_column` during index creation (the various `source_build` tests starting around lib.rs:739). The only gap would be if someone could create an index with `build_source_column` set to a value that doesn't trigger `strip_prefix("build_source_column=")`, but PostgreSQL's reloption machinery guarantees the format.

### 6. Minor: the error message could hint at the alternative

The current message is: `"tqhnsw aminsert does not support build_source_column indexes yet: {source_column}"`. The "yet" is appropriate — it signals this is a known limitation, not a data corruption scenario. No change needed.
