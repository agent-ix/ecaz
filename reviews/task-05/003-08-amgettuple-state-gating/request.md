# Review Request: `amgettuple` State Gating

Scope:
- `src/am/mod.rs`
- `src/lib.rs`

What changed:
- `amgettuple` no longer immediately falls through to the generic build-only error.
- It now verifies that the scan descriptor exists, that opaque scan state exists, and that `amrescan` has been called first.
- After those checks, it still rejects actual tuple production with a narrow "not implemented yet" error.

Review focus:
- Scan-callback state machine correctness
- Error-surface coherence between `ambeginscan`, `amrescan`, and `amgettuple`
- Whether the current gating is the right narrow boundary before real result iteration exists

Questions to answer:
- Are there any scan lifecycle paths where the new `amgettuple` checks are still too weak?
- Are the two failure modes distinct and useful enough for debugging executor behavior?
- Are there missing tests around null scan descriptors, missing opaque state, or repeated rescans?

---

## Review Comments

Status at `41cfdfa`:
- Questions above: closed for this stage as skipped/not needed.
- Comments 1-5 and 7: not needed. These comments confirm the current gating behavior rather than identifying changes required before real scan execution exists.
- Comment 6: not needed for now. Repeated-rescan coverage stays blocked on the current fatal `amgettuple` capability boundary and would add helper complexity without increasing behavioral confidence.

### 1. The three-stage gate is correctly ordered and sufficient

`amgettuple` (lines 417-439) checks:
1. Null scan descriptor (line 423-425)
2. Null opaque pointer (line 427-429)
3. `rescan_called` flag (line 432-434)
4. Final "not implemented" error (line 437)

This correctly models the PostgreSQL scan state machine: `ambeginscan` → `amrescan` → `amgettuple`. Each check catches a different class of misuse, and the error messages are distinct enough to diagnose which stage was skipped.

### 2. The null opaque check (stage 2) is technically unreachable given current code

Since `ambeginscan` always allocates opaque state (line 352), and `amendscan` nulls it out (but the executor shouldn't call `amgettuple` after `amendscan`), the only way `opaque` could be null is if:
- Someone calls `amgettuple` without `ambeginscan` (a PostgreSQL bug)
- Memory corruption

This check is still valuable as a defense-in-depth assertion. Not a problem.

### 3. Error messages form a coherent diagnostic chain

The three errors have distinct prefixes:
- `"tqhnsw amgettuple received a null scan descriptor"` — infrastructure failure
- `"tqhnsw amgettuple missing scan opaque state"` — lifecycle violation
- `"tqhnsw amgettuple requires amrescan before scan execution"` — protocol violation
- `"tqhnsw scan execution is not implemented yet: amgettuple"` — capability boundary

These four levels make debugging straightforward. The last message clearly separates "you called the right things in the right order, but the feature isn't done yet" from "you called things wrong."

### 4. `_direction` parameter is ignored — correct for now

At line 419, `_direction` is unused. Since the function always errors, there's nothing to do with direction. When real scan execution is implemented, this will need to handle `ForwardScanDirection` (and potentially reject `BackwardScanDirection` since `amcanbackward = false` at line 73). No issue for this slice.

### 5. No scan state is mutated — clean for error recovery

Since `amgettuple` only reads the opaque state and never mutates it, if the "not implemented" error is caught by a subtransaction or savepoint, the scan state remains consistent for `amendscan` to clean up. This is a nice property of the current gating approach.

### 6. Test coverage

- `test_tqhnsw_gettuple_scaffold_requires_rescan` (lib.rs:1574) — calls `amgettuple` without `amrescan`, expects the "requires amrescan" error. This exercises stage 3.
- `test_tqhnsw_gettuple_scaffold_rejects_execution_after_rescan` (lib.rs:1599) — calls `amrescan` then `amgettuple`, expects the "not implemented" error. This exercises stage 4.

**Coverage is good for the two reachable error paths.** The null-descriptor and null-opaque paths (stages 1 and 2) aren't testable without low-level pointer manipulation, which is reasonable to skip.

**Missing test: repeated rescan + gettuple cycle.** A test that does `ambeginscan` → `amrescan(query_a)` → attempts `amgettuple` (errors) → then somehow does `amrescan(query_b)` → attempts `amgettuple` again would confirm the opaque state is properly overwritten across rescans. However, since `amgettuple` currently errors fatally, this can't be tested without catching the error, which is difficult in pgrx test harness. This is fine to defer until real scan execution is implemented.

### 7. No issues found — this is a well-structured gating slice

The state machine checks are correct, the error messages are clear and distinct, and the test coverage matches what's practically testable. The gate cleanly separates "lifecycle correctness" from "feature completeness" and sets up a natural insertion point for real scan logic to replace the final error.
