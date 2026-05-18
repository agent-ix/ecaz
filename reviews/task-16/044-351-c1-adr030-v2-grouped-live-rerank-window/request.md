# Review Request: C1 ADR-030 V2 Grouped Live Rerank Window

## Context

Packet `350` added a parameterized sliding-window simulation over grouped emitted comparison rows
without changing live scan behavior:

- grouped comparison rows already carried approximate score plus exact rerank comparison score
- grouped order/window diagnostics could already simulate `window_size = 1/2/4/8`
- SQL surfaces existed to compare baseline approximate order against simulated windowed order

That left ADR-030 with enough evidence to choose a small live rerank prefix, but the runtime was
still emitting pure approximate grouped order.

## Problem

The next runtime slice needed to make grouped-v2 queries materially closer to the intended
`binary -> grouped -> rerank` pipeline, but it had to avoid collapsing the measurement seams from
packets `346-350`.

Two constraints mattered:

1. live grouped output needed to start using a real rerank window instead of diagnostics only
2. grouped comparison/order/window SQL surfaces still needed to preserve baseline approximate
   semantics after live output order changed

Without explicit sidecars for baseline approximate rank/score, the existing diagnostics would have
silently redefined "approximate order" to mean "whatever the live reranked scan emitted."

## Planned Slice

Batch the next related runtime slices together:

1. choose a concrete grouped live rerank prefix of `4`
2. wire that window into the grouped-v2 graph scan path behind the existing
   `TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN` gate
3. preserve baseline approximate rank/score sidecars on emitted grouped results
4. keep grouped comparison/window diagnostics anchored to baseline approximate order after cutover
5. add focused proof tests for the window simulation edge cases called out in packet `350` feedback
6. prove the live grouped runtime now matches the `window_size = 4` simulation on a real pg query

This slice intentionally excludes:

- no output-score cutover yet
- no gate lift
- no corpus recall or latency claims yet
- no planner/runtime broadening beyond grouped-v2 scans already behind the existing gate

## Implementation

Updated:

- `src/am/scan.rs`
- `src/am/scan_debug.rs`
- `src/lib.rs`

Concrete changes:

1. added an inline grouped live-rerank buffer in `src/am/scan.rs` with a fixed width of `4`
2. changed grouped graph-result prefetch to:
   - keep consuming approximate frontier candidates into that buffer
   - preserve each candidate's baseline approximate rank base and approximate score
   - select the buffered candidate with the best exact rerank comparison score, breaking ties by
     baseline approximate rank
3. changed grouped result emission bookkeeping to retain:
   - last emitted baseline approximate score
   - last emitted baseline approximate rank
   - last emitted exact rerank comparison score
4. updated grouped debug row construction so grouped comparison/window diagnostics sort by the
   preserved approximate-rank sidecar instead of live emitted order
5. folded the packet `345` reviewer nit into `candidate_score_dispatch(...)` so grouped score
   context is computed once instead of twice
6. documented the window-simulation missing-comparison fallback and tail-drain semantics inline
7. added pure Rust tests proving:
   - `window_size = 1` is a no-op
   - tied exact scores preserve baseline approximate order
   - `window_size >= emitted_count` behaves like the full emitted-set rerank case
8. added a pg proof that:
   - a live grouped query now emits the same order as the `window_size = 4` simulation
   - the selected query actually changes live order versus baseline approximate order
   - grouped comparison rows still expose the preserved baseline approximate order after cutover

## Measurements

This packet changes live grouped output order behind the existing experimental scan gate, but it
still does not make corpus-scale recall or latency claims.

Validation results for this checkpoint:

- focused validation:
  - `cargo test grouped_window_simulation -- --nocapture`: passed
  - `cargo test test_grouped_v2_runtime_live_window_matches_windowed_simulation -- --nocapture`: passed
- full checkpoint:
  - `cargo test`: passed
  - `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`: passed
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`: passed

## Outcome

ADR-030 grouped-v2 scans now use a real narrow rerank window in the live graph path while keeping
the existing diagnostic ladder meaningful.

What this de-risks:

1. grouped-v2 ordered queries are no longer "approximate only" once the scan gate is enabled
2. packet `346-350` diagnostics remain comparable across the cutover because baseline approximate
   rank/score are preserved explicitly
3. the next benchmark slice can measure the actual live grouped runtime rather than only a
   simulated window

## Next Slice

The next batch should move from runtime cutover to real operating-point evidence:

1. build a scratch grouped-v2 real-corpus index with the existing experimental build/scan gates
2. run external recall and SQL-latency comparisons against the current scalar baseline
3. use those numbers to decide whether the live `window = 4` choice is sufficient or needs a
   broader prefix before any gate-lift conversation
