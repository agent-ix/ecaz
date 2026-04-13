# Review Request: C1 Inline Heaptid Element Decode

## Context

Packet `275` ruled out the first live ADR-029 runtime seam: the source-local
survivor gate regressed the verified warm real-corpus surface and was
discarded.

The remaining low-risk C1 work is still the graph decode/materialization path.
Reviewer feedback on packets `262` and `263` explicitly called out inline
heaptids as the smaller, safer next step before another direct-decode attempt.

Current element decode still does avoidable allocation churn:

- `src/am/page.rs` decodes all heap tids into a `Vec<ItemPointer>`
- then truncates that `Vec` down to the actual count
- `src/am/graph.rs` stores the same heap tids in another `Vec<ItemPointer>`

That work sits on every element load even though the tuple format already has a
fixed `HEAPTID_INLINE_CAPACITY`.

## Problem

The current `Vec<ItemPointer>` decode path adds allocator and copy overhead to
every loaded graph element, but earlier packet `262` showed that smaller copy
boundary trims can still produce measurable warm wins. This slice needs to
remove the heaptid allocation churn without reopening the larger direct-decode
regression from packet `263`.

## Planned work

1. Change `TqElementTuple` to store heap tids in an inline
   `[ItemPointer; HEAPTID_INLINE_CAPACITY]` plus count.
2. Mirror that ownership shape in `GraphElement`.
3. Keep decode control flow otherwise stable so the slice isolates heaptid
   allocation removal rather than rewriting the whole decoder.
4. Update scan/materialization call sites and tests.
5. Validate on the verified warm real-corpus surface and record whether the
   slice is a keep or a failed experiment.

## Exit criteria

- `TqElementTuple` no longer allocates a heap-tid `Vec` during decode
- `GraphElement` no longer allocates a heap-tid `Vec` for loaded elements
- the checkpoint gate is green:
  - `cargo test`
  - `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
- the packet records the verified warm real-corpus before/after readout

## Outcome

Discarded.

The attempted ownership change replaced `Vec<ItemPointer>` heap-tid payloads in
`TqElementTuple`, `GraphElement`, and `SelectedScanResult` with an inline
`[ItemPointer; HEAPTID_INLINE_CAPACITY]` plus count, while keeping the broader
decode flow intact.

The local targeted page-codec checks passed:

- `cargo test element_tuple_roundtrip -- --nocapture`
- `cargo test inline_heaptids -- --nocapture`

But the full checkpoint gate did not stay clean. Under the full `cargo test`
suite, two bootstrap-frontier pg tests failed in separate reruns:

- `tests::pg_test_tqhnsw_frontier_head_refills_from_consumed_neighbors`
- `tests::pg_test_tqhnsw_rescan_builds_bootstrap_candidate_frontier`

Both failures indicated that the graph bootstrap frontier could collapse to only
the seeded entry candidate under full-suite order, even though the same tests
passed in isolation after the rollback. That is not a safe checkpoint for a
small allocation win, so the code change was reverted instead of being pushed.

## Validation

- `cargo test tests::pg_test_tqhnsw_frontier_head_refills_from_consumed_neighbors -- --exact`
  passed after the rollback
- `cargo test tests::pg_test_tqhnsw_rescan_builds_bootstrap_candidate_frontier -- --exact`
  passed after the rollback

## Next step

Stay on the zero-copy / allocation-reduction track, but move to a different
seam than inline heap-tid ownership. The next candidate should avoid changing
bootstrap-frontier-visible ownership semantics while still reducing decode/copy
churn in the graph path.
