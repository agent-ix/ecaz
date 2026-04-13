# Review Request: C1 ADR-031 Tier 2 Pin-and-Hold Borrowed Score

## Context

Packet `289` cleared ADR-031 itself as the cause of the higher-`ef_search`
quality shift. The current Tier 1 ADR-031 path remains the best warm-latency
surface we have on the real `50k` seam, with the canonical `m=8`,
`ef_search=40` warm run around:

- `p50 ~= 1.48ms`
- `p99 ~= 2.4ms`
- `mean ~= 1.51ms`

Reviewer feedback on the ADR-031 arc identified one remaining hot-path copy on
the exact-score miss path:

- `LoadedElementScoreInput` still materializes `element.code.to_vec()`
- that copy exists because exact scoring happens after the page buffer is
  released

## Problem

Tier 1 eliminated most scan-cache churn, but the ADR-031 hot path still copies
the quantized code payload into an owned `Vec<u8>` before exact scoring can run
on newly loaded elements.

That leaves a clear next seam:

- hold the graph element buffer pinned while exact scoring happens
- score directly from borrowed `TqElementTupleRef.code`
- delete the remaining `element.code.to_vec()` copy from the ADR-031 exact
  score path

## Planned Slice

Implement the Tier 2 pin-and-hold path described in the review feedback:

1. split graph element reads into a pin-and-hold API in `src/am/graph.rs`
2. move exact scoring for newly loaded elements into that pin scope
3. score directly from borrowed tuple bytes instead of an owned copied buffer
4. keep the rest of the ADR-031 cache shape unchanged unless the pin scope
   forces a small supporting change

## Success Criteria

- exact scoring on the ADR-031 miss path no longer requires
  `element.code.to_vec()`
- the code remains correct under PostgreSQL buffer lifetime rules
- `cargo test`, `cargo pgrx test pg17`, and clippy are green
- the packet records whether the Tier 2 seam materially improves the canonical
  warm real-`50k` ADR-031 surface

## Experiment

The runtime slice was implemented in:

- `src/am/graph.rs`
- `src/am/scan.rs`

with a supporting pg_test reset hardening during validation in:

- `src/lib.rs`

The runtime change replaced the owned `LoadedElementScoreInput.code_bytes`
payload with a pin-and-hold path:

1. pin graph element buffers on ADR-031 cache miss
2. keep survivor candidates' buffers pinned across the binary approximate pass
3. exact-score survivors directly from borrowed `TqElementTupleRef.code`
4. remove the `element.code.to_vec()` allocation from the exact-score miss path

## Validation

The experimental code was validated successfully while in place:

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

## Warm Canonical Read

Command used:

```bash
scripts/bench_sql_latency_verified_scratch.sh \
  --prefix tqhnsw_real_50k \
  --m 8 \
  --ef-search 40 \
  --cache-state warm-after-prime3 \
  --warmup-passes 3 \
  --session-mode per-cell \
  --timing-mode cached-plan \
  --output /tmp/adr031_tier2_real_50k_m8_ef40_warm.summary
```

The first attempt hit the known WSL negative-timestamp guard and aborted cleanly:

```text
invalid negative per-query timings parsed for cached-plan: count=2 min=-949.636ms; rerun this cell
```

Two clean reruns both regressed versus the standing Tier 1 baseline:

```text
run 1: p50=1.608ms p95=2.226ms p99=2.668ms mean=1.643ms
run 2: p50=1.609ms p95=2.259ms p99=2.701ms mean=1.649ms
```

Standing Tier 1 baseline from packet `287`:

```text
run A: p50=1.480ms p99=2.390ms mean=1.507ms
run B: p50=1.485ms p99=2.422ms mean=1.510ms
```

## Readout

Tier 2 did remove the remaining owned code-copy seam, but on the canonical warm
real-`50k` ADR-031 surface it was consistently slower:

- `mean`: about `1.51ms -> 1.64-1.65ms`
- `p50`: about `1.48ms -> 1.61ms`
- `p99`: about `2.39-2.42ms -> 2.67-2.70ms`

That is a real regression, not a noise-level tie. The likely reason is that the
saved `Vec<u8>` copy is smaller than the cost of keeping multiple candidate
buffers pinned across the approximate pass on the hot warm path.

## Disposition

This Tier 2 variant is a discard.

The pin-and-hold borrowed-score runtime should not stay on the branch in this
shape. The next ADR-031 work should move elsewhere:

1. keep the current Tier 1 inline-cache path as the warm winner
2. treat deeper arena/pinned-cache ideas as ADR-032 territory rather than more
   incremental Tier 2 work on this exact seam
