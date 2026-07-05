# Review Request: C1 ADR-030 V2 Storage-Format Matrix And Runtime Gate Closure

Current head: `4fec776`

This packet covers local uncommitted work on top of that head.

## Context

Reviewer feedback on packets `409` and `417` identified two real remaining gaps:

1. the storage-format REINDEX guardrail only had one mismatch direction covered
2. there was no happy-path proof that `ALTER INDEX ... SET (storage_format = ...)`
   followed by `REINDEX` restores normal scan/insert/vacuum behavior
3. `417`'s one non-mechanical AM runtime change,
   `grouped_binary_traversal_score_enabled(...)`, still lacked a direct unit test
4. the default-vs-explicit source-backed rerank parity test compared output rows,
   but did not assert the runtime helper's resolution reason on each lane

This slice closes those proof gaps without expanding the product surface beyond
the current branch contract.

## Problem

Before this slice:

1. `409` proved `turboquant -> pq_fastscan` mismatch rejection on scan/insert/vacuum
2. the reverse direction, `pq_fastscan -> turboquant`, was untested
3. there was no explicit pg proof that a matching `REINDEX` clears the guardrail
4. the default rerank parity test did not verify that:
   - source-backed defaults resolve to `heap_f32` because of `build_source_column`
   - explicit overrides resolve to `heap_f32` because of the env override
5. the binary traversal gate tightening in `src/am/scan.rs` had no direct unit
   coverage proving it only activates for actual `PqFastScan` layouts

The code shape was close, but the test matrix was still incomplete.

## Planned Slice

Close the remaining matrix and contract gaps:

1. add reverse-direction mismatch coverage on ordered scan, insert, and vacuum
2. add a `REINDEX` happy path that proves runtime behavior is restored after the
   reloption and on-disk format are brought back into alignment
3. make the source-backed rerank parity test assert runtime-resolution reasons
4. add a direct unit test for the binary traversal gate

## Implementation

Updated:

- `src/lib.rs`
- `src/am/scan.rs`

Concrete changes:

1. named the float comparison tolerance as `SCORE_ASSERT_EPSILON`
2. tightened `test_pq_fastscan_default_rerank_matches_explicit_heap` so it now
   asserts:
   - default fixture `pq_fastscan_rerank_mode = heap_f32`
   - default fixture
     `pq_fastscan_rerank_mode_resolution = default_heap_f32_with_build_source_column`
   - explicit override fixture `pq_fastscan_rerank_mode = heap_f32`
   - explicit override fixture
     `pq_fastscan_rerank_mode_resolution = env_override`
   - emitted ordered results still match exactly between the two lanes
3. refactored the turboquant runtime fixture helper so it can build either:
   - source-less turboquant fixtures
   - source-backed turboquant fixtures with `build_source_column = 'source'`
4. added reverse mismatch pg coverage:
   - `test_tqhnsw_storage_format_switch_reverse_requires_reindex`
   - `test_tqhnsw_storage_format_switch_reverse_rejects_insert`
   - `test_tqhnsw_storage_format_switch_reverse_rejects_vacuum`
5. added the happy-path guardrail-clear pg test:
   - `test_tqhnsw_storage_format_switch_reindex_restores_runtime`
   - starts from a source-backed turboquant fixture
   - flips the reloption to `pq_fastscan`
   - runs `REINDEX`
   - proves metadata is now `PqFastScan`
   - proves ordered scan still returns self-ranked results
   - proves insert succeeds after `REINDEX`
   - proves vacuum still removes a deleted heap tid after `REINDEX`
6. added the direct AM unit test:
   - `am::scan::tests::grouped_binary_traversal_score_gate_requires_pq_fastscan_storage`
   - off for `TurboQuant` even with binary mode
   - on for `PqFastScan` with binary mode
   - off again when mode changes back to grouped-PQ

## Validation

Passed locally on the current tree:

- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
- `cargo test test_tqhnsw_storage_format_switch_reindex_restores_runtime -- --nocapture`
- `cargo test grouped_binary_traversal_score_gate_requires_pq_fastscan_storage -- --nocapture`

Wrapper status:

1. `bash scripts/run_pgrx_pg17_test.sh` inside the sandbox still fails when
   `cargo pgrx install --test` cannot write
   `/home/peter/.pgrx/17.9/pgrx-install/share/postgresql/extension/tqvector.control`
   (`Read-only file system`, `os error 30`)
2. the same wrapper rerun outside the sandbox passes on the current tree

## Outcome

This closes the specific merge-readiness gaps called out in the review:

1. both storage-format mismatch directions are now covered on the real AM entry
   paths
2. there is an explicit `REINDEX`-restores-runtime proof instead of only
   mismatch coverage
3. the source-backed rerank parity test now proves both output equality and
   runtime-resolution reasons
4. the only non-mechanical AM runtime tweak from the earlier fixture-alignment
   batch now has direct unit coverage

## Next Slice

Keep the remaining proof focused on merge readiness:

1. capture the specific green test names the reviewer asked for
2. separate local proof from sandbox install restrictions
3. package the final landing case as an evidence-only review packet
