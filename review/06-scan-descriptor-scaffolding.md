# Review Request: Scan Descriptor Scaffolding

Scope:
- `src/am/mod.rs`
- `src/lib.rs`

What changed:
- `ambeginscan` now allocates a real PostgreSQL `IndexScanDesc` for `tqhnsw`.
- The AM attaches a small opaque scan-state struct to that descriptor.
- `amendscan` frees that opaque state instead of hard-erroring.
- Actual scan execution is still unsupported; this slice only establishes descriptor lifecycle.

Review focus:
- Scan descriptor ownership and cleanup
- Memory-context safety of the opaque scan state
- Whether the current begin/end behavior matches PostgreSQL AM expectations closely enough for a narrow groundwork slice

Questions to answer:
- Is the descriptor/opaque lifecycle correct under normal executor cleanup?
- Is there any double-free or leak risk in the current `amendscan` plus `IndexScanEnd` split?
- Is there any smaller or safer way to stage scan groundwork here?

---

## Review Comments

Status at `41cfdfa`:
- Comments 1-5 and 7: not needed. These notes validate the current descriptor lifecycle rather than requiring changes.
- Comment 6 addressed by adding repeated-`amendscan` idempotency coverage.

### 1. `PgBox::alloc0` allocates in CurrentMemoryContext — this is correct for scan opaque

At line 352, `PgBox::<TqScanOpaque>::alloc0()` allocates via `palloc0` in whatever `CurrentMemoryContext` is active when `ambeginscan` is called. PostgreSQL calls `ambeginscan` with the per-query or per-scan memory context active, which is the standard lifetime for scan opaque state. The `amendscan` call to `pfree` at line 451 (via the opaque pointer) properly releases it in the same context. This matches how other AMs (e.g., `btbeginscan`/`btendscan`) handle opaque state.

### 2. `amendscan` correctly handles null scan descriptors

At line 445-447, `amendscan` returns early if `scan` is null. While PostgreSQL shouldn't pass a null descriptor to `amendscan`, this defensive check is harmless and matches the pattern in other AM implementations.

### 3. No double-free risk in the `amendscan` + `IndexScanEnd` split

The test helper `debug_begin_end_scan` (mod.rs:1977-1989) calls `amendscan` then `IndexScanEnd`. After `amendscan` nulls out `(*scan).opaque` (line 452: setting it to `ptr::null_mut()`), a subsequent call to `amendscan` would see the null and skip the free. `IndexScanEnd` frees the `IndexScanDesc` itself (the outer struct), not the opaque pointer. These are separate allocations with separate owners. No double-free.

### 4. Potential leak if `amendscan` is never called

If the executor errors out between `ambeginscan` and `amendscan` (e.g., during `amrescan` or `amgettuple`), the opaque state allocated via `palloc0` will be freed when the memory context is reset/destroyed by PostgreSQL's error cleanup. So there's no persistent leak — PostgreSQL's memory context system handles this. This is the standard AM contract.

### 5. `TqScanOpaque` is `#[repr(C)]` — correct for a pointer stored in a C struct

The struct (line 1066-1071) is `#[repr(C)]`, `Debug`, `Default`, and `Copy`. Since it's allocated via `palloc0` and accessed through a raw pointer cast, `repr(C)` ensures the layout is deterministic and the `alloc0` zeroing produces a valid `Default` state (`rescan_called: false`, `query_dimensions: 0`). This is clean.

### 6. The test verifies the full lifecycle

`test_tqhnsw_scan_scaffold_allocates_and_frees_state` (lib.rs:1495) asserts that after `ambeginscan`, opaque is non-null, and after `amendscan`, opaque is null. This is a good lifecycle test.

**Missing edge case:** There's no test for calling `amendscan` twice on the same descriptor (idempotency). The code handles this correctly (null check at line 449-452), but an explicit test would document the contract. Low priority.

### 7. No issues found — this is a clean, minimal scaffolding slice

The begin/end pair does exactly what's needed: allocates a descriptor with opaque state, and cleans it up. No premature complexity, no hidden state mutations. Ready for the next slice to build on.
