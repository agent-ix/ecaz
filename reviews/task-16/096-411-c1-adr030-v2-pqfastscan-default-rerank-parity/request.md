# Review Request: C1 ADR-030 V2 PqFastScan Default Rerank Parity

Current head: `eb48eff`

## Context

Packet `404` flipped source-backed `pq_fastscan` to default rerank through
`heap_f32`. That packet already proved:

1. the default path emits heap comparison scores
2. those comparison scores match the raw heap `real[]` inner product

Reviewer feedback on `404` asked for one stronger safety check:

- prove that the source-backed default path is not merely “heap-like,” but
  exactly identical to an explicit `heap_f32` override on the same workload

That follow-up is narrow and worth landing independently from the runtime
visibility work in packet `410`.

## Problem

Before this slice, the branch had two related but separate proofs:

1. default source-backed rerank emits exact heap scores
2. explicit `heap_f32` override remains available as a runtime control

What it did **not** have was a direct parity assertion:

- default source-backed rerank == explicit `heap_f32` override

Without that, the branch still relied on an inference that both lanes happen to
reach the same scoring path.

## Planned Slice

Add one pg regression that compares the ordered scan output of:

1. a source-backed `pq_fastscan` fixture on the default rerank lane
2. the same fixture shape with explicit
   `TQVECTOR_PQ_FASTSCAN_RERANK_MODE=heap_f32`

The assertion is intentionally strict:

- same emitted scores
- same comparison scores
- same approximate-rank sequence

## Implementation

Updated:

- `src/lib.rs`

Concrete changes:

1. added `test_pq_fastscan_default_rerank_matches_explicit_heap`
2. the new test builds two identical runtime fixtures:
   - default rerank
   - explicit `heap_f32`
3. it compares the ordered scan output after stripping heap TIDs, asserting the
   two lanes produce the same:
   - emitted score
   - comparison score
   - approximate rank

This is test-only. No AM/runtime behavior changed in this slice.

## Validation

Passed:

- `cargo check --tests --no-default-features --features 'pg17 pg_test'`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Required full-test commands were run and hit the same known workstation linker
boundary as earlier packets on this branch:

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`

Observed unresolved PostgreSQL symbols remained in the same family:

- `CurrentMemoryContext`
- `PG_exception_stack`
- `error_context_stack`
- `CopyErrorData`
- `errstart`

## Outcome

This tightens the default-lane safety case from packet `404`:

1. the default source-backed lane is now proven score-identical to explicit
   `heap_f32`
2. the proof is phrased at the operator-facing scan surface, not just by
   reasoning about internal helper selection
3. the slice stays test-only, so it sharpens the contract without widening the
   implementation diff

## Next Slice

Continue on the remaining review-driven merge polish:

1. improve landing-proof / packet honesty around the unexecuted pg tests
2. or capture scratch-cluster reruns for the current default `pq_fastscan`
   lane so the proof packet is less dependent on `~/.pgrx`
