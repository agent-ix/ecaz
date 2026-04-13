# Review Request: C1 ADR-031 Persisted Sidecar A/B Compare

## Context

Packet `285` landed the first persisted ADR-031 sidecar slice:

- optional trailing binary sidecars on element tuples
- bulk-build write support on the supported no-QJL `4-bit` lane
- scan reads prefer persisted sidecars and derive only as fallback

That slice is already green and pushed, and the first cold read on the rebuilt
real `50k` index came back at `5.537ms` for a single `m=8`, `ef_search=40`
query.

## Problem

We now know persisted ADR-031 sidecars work. We still do **not** know whether
they are worth carrying.

The missing evidence is an A/B measurement against the same codebase with
persisted sidecars deliberately ignored at runtime, so the scan falls back to
binary-word derivation on cache miss.

Without that same-build comparison, the cold read from packet `285` is just an
absolute number, not a clear value judgment.

## Planned Investigation

Add the smallest safe comparison seam that:

- leaves persisted sidecars on disk
- forces scan-time binary-word derivation instead of using them
- is easy to switch on and off for a local benchmark

Then run cold real-`50k`, `m=8`, `ef_search=40` reads on both modes and record
the delta.

## Success Criteria

- the packet records the exact A/B switch used
- the packet records cold measurements for persisted-sidecar `auto` vs
  `derive-only`
- the packet makes a clear keep/drop call for persisted ADR-031 sidecars

## A/B Switch

The same-build compare seam uses the hidden database GUC added in
[src/am/options.rs](/home/peter/dev/tqvector/src/am/options.rs) and consumed in
[src/am/scan.rs](/home/peter/dev/tqvector/src/am/scan.rs):

- `tqhnsw.force_binary_derivation = off`:
  use persisted binary sidecars when present (`auto`)
- `tqhnsw.force_binary_derivation = on`:
  ignore persisted sidecars at scan time and derive binary words from `code`
  bytes on cache miss (`derive-only`)

Both modes read the same rebuilt real-`50k` index with persisted sidecars on
disk. Only the scan-time read path changes.

## Cold Measurement

Cold real-`50k`, `m=8`, `ef_search=40`, single-query reads were measured after
explicit scratch-postmaster restarts so shared buffers started cold for each
sample.

Auto mode command:

```bash
scripts/bench_sql_latency_verified_scratch.sh \
  --prefix tqhnsw_real_50k \
  --m 8 \
  --ef-search 40 \
  --query-limit 1 \
  --cache-state cold-sidecar-auto-reconfirm \
  --warmup-passes 0 \
  --session-mode per-query \
  --timing-mode plain-server \
  --output /tmp/adr031_sidecar_auto_reconfirm_cold_q1.summary
```

Observed reconfirmed output:

```text
m=8
ef_search=40
query_limit=1
cache_state=cold-sidecar-auto-reconfirm
session_mode=per-query
timing_mode=plain-server
p50=6.164ms
p95=6.164ms
p99=6.164ms
mean=6.164ms
server_qps=162.23
wall=0.02s
```

Derive-only command:

```bash
scripts/bench_sql_latency_verified_scratch.sh \
  --prefix tqhnsw_real_50k \
  --m 8 \
  --ef-search 40 \
  --query-limit 1 \
  --cache-state cold-sidecar-derive-only-reconfirm \
  --warmup-passes 0 \
  --session-mode per-query \
  --timing-mode plain-server \
  --output /tmp/adr031_sidecar_derive_only_reconfirm_cold_q1.summary
```

Observed reconfirmed output:

```text
m=8
ef_search=40
query_limit=1
cache_state=cold-sidecar-derive-only-reconfirm
session_mode=per-query
timing_mode=plain-server
p50=9.569ms
p95=9.569ms
p99=9.569ms
mean=9.569ms
server_qps=104.50
wall=0.02s
```

Supporting first-pass samples on the same seam showed the same ordering:

- `auto`: `6.610ms`
- `derive-only`: `9.426ms`

## Readout

Persisted ADR-031 sidecars are a keep.

The same-build A/B delta is large and stable enough to matter:

- reconfirmed cold q1: `6.164ms` auto vs `9.569ms` derive-only
- supporting first pass: `6.610ms` auto vs `9.426ms` derive-only
- rough effect size: about `3.1ms` faster cold startup, or about `33%` lower
  latency versus forced derivation

That is the exact use case packet `285` predicted:

- warm ADR-031 already hides derivation cost once the scan-local cache is hot
- persisted sidecars pay off on cold cache misses, where derivation is still on
  the critical path

This does **not** prove a warm-path gain from persisted sidecars, and it does
**not** make the insert-path gap disappear. Incremental inserts still need
binary-sidecar writes before this can be considered fully production-ready.

It does answer the keep/drop question: the persisted sidecar is pulling its
weight on the cold path and should stay.

## Next Step

With the persisted-sidecar question answered, the next ADR-031 slice should
return to the reviewer-identified scan hot path:

1. Tier 1: replace `Vec<u64>` / `Vec<ItemPointer>` in the cached graph-element
   path with inline arrays and counts where the cardinality is bounded
2. Tier 2: split page reads into a pin-and-hold shape so exact scoring can read
   borrowed code bytes directly from `TqElementTupleRef` instead of copying
   `element.code.to_vec()`
