# Review Request: C1 Task16 Rerank-Source Reset Regression

Current head at execution: `2336557`

## Context

Two follow-ups were still open after packets `439`, `440`, and `447`:

1. Reviewer feedback on packet `439` asked for a real script-surface regression
   around:
   - `ALTER INDEX ... RESET (rerank_source_column)`
   - `ALTER INDEX ... SET (rerank_source_column = 'source_raw')`
   The packet-`440` measurement methodology relies on that ALTER cycle to flip
   the same TurboQuant index between build-source and persisted-rerank-source
   modes. Until it was locked in with a pg test, that reproducibility seam was
   only exercised manually.
2. Packet `447` answered the "`PLAIN` is fast, but what does it cost?" question,
   but reviewer feedback plus ADR-044 make the storage-policy point explicit:
   packet `447` is **not** enough data to declare a default for `ecvector`.
   ADR-043 had started to read too much like the answer was already settled.

This slice is intentionally narrow:

- add the missing `ALTER INDEX ... SET/RESET` regression
- correct ADR-043 so it defers the storage-policy default to ADR-044
- update the task-16 plan so the remaining ADR-044 matrix is explicit

No new timed benchmark cells ran in this slice.

## What changed

### 1. TurboQuant reloption round-trip is now locked in by pg regression

Added:

- `test_turboquant_rerank_source_reloption_reset_round_trip`

The test:

1. builds a TurboQuant fixture with:
   - `build_source_column = source`
   - persisted `rerank_source_column = source_raw`
2. forces the runtime into `heap_f32` rerank mode
3. asserts the emitted comparison scores match the `source_raw` exact scores
4. runs:

   ```sql
   ALTER INDEX ... RESET (rerank_source_column)
   ```

5. asserts:
   - `reloptions` no longer contain `rerank_source_column=source_raw`
   - emitted comparison scores now match the build-source `source` column
6. runs:

   ```sql
   ALTER INDEX ... SET (rerank_source_column = 'source_raw')
   ```

7. asserts both:
   - the reloption is persisted again
   - emitted comparison scores flip back to the `source_raw` exact scores

So the packet-`440` measurement seam is no longer a manual assumption. A future
reloption/refactor change now has to preserve both the catalog surface and the
runtime score-selection behavior.

### 2. ADR-043 no longer overstates the storage-policy answer

`spec/adr/ADR-043-native-ecvector-raw-f32-column-type.md` now trims the earlier
"storage policy guidance" section and replaces it with an explicit deferral:

- ADR-043 still owns the type decision (`ecvector(dim)` is the canonical row
  type)
- ADR-044 owns the storage/location decision
- only `EXTENDED` vs `PLAIN` and the `PLAIN` tradeoff have been measured so far
- `EXTERNAL`, `MAIN`, and `PLAIN + fillfactor` are still unmeasured

So current head does **not** pretend the default is settled. Until ADR-044's
matrix lands, `ecvector` keeps PostgreSQL's normal varlena default and `PLAIN`
remains an expert knob, not a declared product default.

### 3. Task 16 plan now reflects the actual remaining storage-policy work

`plan/tasks/16-turboquant-iteration.md` now does three things:

1. closes the old open hygiene item for the `ALTER INDEX ... SET/RESET`
   regression and points it at this packet
2. updates the outcome section so packet `447` is read as:
   - "`PLAIN` is fast"
   - "`PLAIN` is costly on churn-heavy rows"
   - default still open pending ADR-044
3. adds one explicit open checklist item for the ADR-044 matrix:
   - `EXTERNAL`
   - `MAIN`
   - `PLAIN + fillfactor`
   - larger touched-column update probe
   - detoast-vs-decompress decomposition when practical
   - C1 index-side cold-page sketch

That keeps task 16 honest about what is actually still required to land the
`ecvector` storage-policy decision.

## Why this matters

This slice is mostly about removing ambiguity:

- packet `440`'s reproducibility seam is now tested instead of assumed
- ADR-043 no longer implies a default from incomplete evidence
- the plan now tells the truth about the remaining task-16 storage-policy work

That is the right setup before running the ADR-044 cells. The next benchmark
work can happen against an explicit decision matrix instead of drifting doc
guidance.

## Validation

Ran on this exact tree:

```bash
cargo test
bash scripts/run_pgrx_pg17_test.sh
cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings
```

## Review focus

1. Does `test_turboquant_rerank_source_reloption_reset_round_trip` adequately
   lock in both halves of the packet-`440` seam:
   - reloption persistence in `pg_class.reloptions`
   - runtime score-source selection after `RESET` and `SET`
2. Is ADR-043 now scoped correctly, with the storage-policy default explicitly
   deferred to ADR-044 instead of inferred from packet `447`?
3. Does the task-16 plan now name the remaining ADR-044 closure work clearly
   enough that the next measurement slice can proceed without ambiguity?
