# Review Request: C1 QJL Build Offset And Grouped PQ Study Alignment

Current head: `a4ccba9`

This packet covers local uncommitted work on top of that head.

## Context

While rerunning the repaired pq-fastscan/test surfaces, two lower-level
assumptions were still out of date with the current quantization/build code:

1. scalar HNSW build distance offset for 4-bit QJL-enabled codes still used a
   centroid-bound heuristic
2. the `approx_score_study` grouped-PQ nibble-pack unit test still used tiny
   codebooks that do not represent the real 4-bit grouped-PQ lane

Both issues sat below the main pq-fastscan runtime path, but they weaken the
same checkpoint if left behind.

## Problem

Before this slice:

1. `BuildCodeDistance::new(...)` could derive an offset that was too tied to a
   centroid heuristic instead of the encoded tuples actually being inserted into
   HNSW
2. the grouped-PQ nibble-pack study test was proving only a toy shape, not the
   full 4-bit / 16-centroid-per-group layout the current code actually packs

That meant both the build-time non-negative-distance translation and the study
test fixture were weaker than the real code path.

## Planned Slice

Bring both low-level assumptions back in line with the current implementation:

1. derive the HNSW offset from actual encoded self-scores across the build
   tuples
2. add direct unit coverage that QJL-enabled 4-bit scalar codes still build a
   non-empty graph
3. expand the grouped-PQ study test fixture to the real nibble-packed 4-bit
   shape

## Implementation

Updated:

- `src/am/build.rs`
- `src/bin/approx_score_study.rs`

Concrete changes:

1. changed `BuildCodeDistance::new(...)` in `src/am/build.rs` to accept the
   current `BuildTuple` slice
2. replaced the old centroid-magnitude heuristic with the maximum actual
   encoded self-score:
   - `score_code_inner_product(dimensions, bits, seed, &tuple.code, &tuple.code)`
3. updated `build_hnsw_graph(...)` to pass `state.heap_tuples` into that helper
4. added `hnsw_graph_builds_for_qjl_enabled_scalar_codes` to prove the 4-bit
   QJL-enabled scalar build path still produces a graph
5. expanded `grouped_pq_encode_packs_two_nibbles_per_byte` in
   `src/bin/approx_score_study.rs` to use full 16-centroid grouped codebooks so
   the nibble-pack assertion exercises the real 4-bit grouped-PQ layout

## Validation

Passed:

- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Still failing in this environment:

- `bash scripts/run_pgrx_pg17_test.sh`

Observed failure remains the same read-only `cargo pgrx install --test`
destination:

- `/home/peter/.pgrx/17.9/pgrx-install/share/postgresql/extension/tqvector.control`
- `Read-only file system (os error 30)`

## Outcome

This tightens two low-level assumptions under the repaired runtime/tests:

1. HNSW build distance translation now keys off the actual encoded tuples being
   built, including QJL-enabled 4-bit scalar lanes
2. the grouped-PQ study/unit test now matches the current nibble-packed 4-bit
   codebook shape instead of a toy table

## Next Slice

Unless new feedback lands on these packets, the remaining work on this local
batch is packaging and commit hygiene rather than more pq-fastscan behavior
change.
