# Review Request: Narrow `aminsert` Groundwork

Scope:
- `src/am/mod.rs`
- `src/am/page.rs`
- `src/lib.rs`

What changed:
- `tqhnsw` now persists `seed` in metadata so live inserts can validate the build-time single-shape invariant.
- `aminsert` accepts a narrow append-only path:
  - decodes the incoming `tqvector`
  - validates `(dimensions, bits, seed)` against index metadata
  - appends one empty neighbor tuple plus one element tuple
  - initializes `entry_point` when inserting into a previously empty index

Review focus:
- Metadata invariant correctness for `(dimensions, bits, seed)`
- Empty-index initialization behavior
- Any WAL, locking, or page-state assumptions that are too weak for this narrow live path
- Whether callback-side errors stay coherent and SQL-visible

Questions to answer:
- Is there a concrete correctness bug in the current append-only `aminsert` path?
- Is there a missing regression test for a realistic edge case in this narrow scope?
- Is any metadata transition unsafe or under-validated?

---

## Review Comments

Status at `52de780`:
- Questions above: closed for this stage as addressed or skipped/not needed.
- Comment 1 addressed by serializing metadata initialization and validation under an exclusive metadata-page lock in `aminsert`.
- Comment 4 addressed by adding a sequential empty-index second-insert regression test.
- Comments 2, 3, and 5: not needed for now. They are optimization/future-invariant notes rather than defects in the current narrow insert path.

### 1. TOCTOU on metadata read vs. update (correctness concern)

In `tqhnsw_aminsert` (mod.rs:203-246), the metadata page is read with a shared lock (`read_metadata_page` takes `BUFFER_LOCK_SHARE` at line 1863), then released. Later, after the element is appended, `update_metadata_page` re-opens the buffer with an exclusive lock. Between the read and the write, a concurrent `aminsert` from another backend could have initialized the same empty-index metadata, or changed the `entry_point`.

For the empty-index case (line 212-215: `dimensions == 0 && bits == 0`), two concurrent inserters could both see the zeroed metadata, both decide they are the initializer, and both write their shape — the last writer wins, but the first writer's element tuple is already on disk with a potentially different shape interpretation. This is a **concrete correctness bug** for the concurrent empty-index init path.

**Suggestion:** Either hold an exclusive lock on the metadata buffer for the entire `aminsert` body (acceptable for this narrow path since it's not performance-critical yet), or re-read metadata under exclusive lock before the conditional update.

### 2. `entry_point` update is unconditionally written even when unchanged

At line 243-246, the metadata is always written via `update_metadata_page`, even when `entry_point` was already valid and wasn't changed. This is harmless but generates a full-page-image WAL record on every non-duplicate insert. Not a bug, but worth noting for future optimization.

### 3. Empty-index initialization doesn't set `max_level`

When the first insert initializes metadata (line 212-215), `dimensions`, `bits`, and `seed` are set, but `max_level` stays at whatever value was in the initial metadata (0 from `initial_metadata()`). Since live inserts currently always assign `level: 0`, this is consistent. But if a future change assigns non-zero levels during live insert, the metadata `max_level` won't track it — the update at line 246 writes the metadata as-is without bumping `max_level`. The test at line 1439 asserts `max_level == 0` which confirms the current contract.

Not a bug today, but the asymmetry with the build path (which sets `max_level` from the graph) is worth a `// TODO` or a defensive `max_level = max_level.max(0)` to make the invariant explicit.

### 4. Test coverage is solid for the narrow scope

The `test_tqhnsw_empty_index_insert_initializes_shape_metadata` test (lib.rs:1417) covers the empty-index path well. The `test_tqhnsw_insert_appends_new_element_tuple` test (lib.rs:1100) covers the non-empty append. Both verify metadata fields and tuple structure.

**Missing edge case:** There's no test for `aminsert` into an empty index followed by a *second* insert that validates against the just-initialized metadata. The current tests do one insert into empty, or insert into a pre-built index. A test that does `CREATE INDEX` on an empty table, then two sequential inserts (different vectors, same shape), would exercise the "initialized metadata validation" path.

### 5. `relation_options` uses SPI inside `aminsert` (minor concern)

`relation_options` (line 409-441) runs an SPI query inside `aminsert`. This works but is unusual for an AM callback — it means every live insert does a catalog lookup. For the current narrow path this is fine, but it's worth caching the parsed options on the scan descriptor or a relcache callback if insert throughput ever matters.
