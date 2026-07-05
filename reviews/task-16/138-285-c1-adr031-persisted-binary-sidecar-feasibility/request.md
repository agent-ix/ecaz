# Review Request: C1 ADR-031 Persisted Binary Sidecar Feasibility

## Context

Packet `281` landed the cached ADR-031 runtime path on `main`.

Packets `282` and `283` then established:

- the cached ADR-031 path clears `NFR-001` on the normative real `50k` lane at
  `m=8`, `ef_search=40`
- the same live graph path does not introduce a new runtime recall loss versus
  exact quantized results at that target seam

That makes cached ADR-031 a real keep. The next implementation question is
whether the binary-sign codes should stay scan-local and derived on cache miss,
or whether they should be persisted in the index tuples as a sidecar.

## Problem

Persisting the ADR-031 binary codes could remove query-time derivation cost on
cache miss and simplify the hot path, but it would also add durable storage
overhead and may require tuple-layout or index-version changes.

Before implementing anything, we need a concrete answer to:

- where the binary sidecar would live in the current tuple/page layout
- whether it can fit as a backwards-compatible extension
- which subsystems would need to write and read it

## Planned Investigation

Inspect the current seams for:

- element tuple encoding/decoding
- page-local tuple storage layout
- build-time element tuple emission
- any existing version or optional-payload support that could host a `192B`
  binary sidecar

If the answer is "feasible in the current format", the next step should be a
small implementation slice on write/read plumbing.

If the answer is "this is really an index-v2 change", record that explicitly
instead of pretending it is a cheap patch.

## Storage Readout

Relevant seams:

- [src/am/page.rs](/home/peter/dev/tqvector/src/am/page.rs) defines the element
  tuple payload directly in `TqElementTuple` / `TqElementTupleRef`
- [src/am/build.rs](/home/peter/dev/tqvector/src/am/build.rs) writes element
  tuples during bulk build
- [src/am/insert.rs](/home/peter/dev/tqvector/src/am/insert.rs) writes element
  tuples during incremental insert and computes page-fit / max-insert-level from
  `TqElementTuple::encoded_len(code_len)`
- [src/am/graph.rs](/home/peter/dev/tqvector/src/am/graph.rs) and
  [src/am/scan.rs](/home/peter/dev/tqvector/src/am/scan.rs) are the main read
  consumers

Important facts:

- the metadata page currently has no explicit format/version field
- element tuples are fixed-length for a given `code_len`
- the current element payload is:
  - tag / level / deleted
  - inline heap tids
  - heap-tid count
  - gamma
  - neighbor tuple tid
  - packed code bytes
- build and insert both assume element tuple length comes from
  `TqElementTuple::encoded_len(code_len)`

Feasibility conclusion:

- persisted ADR-031 sidecars do **not** look like an automatic index-v2 change
- the cleanest shape is an **optional trailing payload** after `code`
- old tuples can keep the current payload length
- new tuples can append persisted binary-sign bytes
- decoders can distinguish old vs new tuples from the tuple length and expose an
  optional borrowed binary slice

Why this is viable:

- `update_raw_tuple(...)` already enforces same-length rewrites, so existing old
  tuples would stay old instead of being silently reshaped in place
- inserts into an older index could append new-format tuples without breaking
  old-format tuple reads, as long as decode handles both lengths
- scan can use persisted binary words when present and keep the current
  derivation fallback when absent

Page-fit impact:

- current 1536-dim, 4-bit no-QJL element payload is `74B + 768B = 842B`
- a persisted binary sidecar adds `192B`, bringing the element payload to
  `1034B`
- with `m=8`, the level-0 neighbor payload is `99B`, so a colocated
  element+neighbor pair still fits comfortably on an `8KB` page
- the higher-level insert cap would shrink slightly, but only because the
  element tuple gets larger; this does not look like a catastrophic layout
  break

## Readout

Persisted ADR-031 sidecars look like a **contained extension**, not an
automatic index-v2 project.

The next implementation slice should be:

1. extend `TqElementTuple` / `TqElementTupleRef` with an optional trailing
   binary sidecar
2. teach build and insert to write it for the no-QJL 4-bit lane
3. teach graph/scan reads to use the persisted sidecar when present and derive
   only as fallback

## Implementation Checkpoint

The first persisted-sidecar slice is now implemented and green.

What landed:

- [src/am/page.rs](/home/peter/dev/tqvector/src/am/page.rs)
  - `TqElementTuple` now supports an optional trailing binary sidecar
  - `TqElementTupleRef` can decode both old tuples and new tuples with trailing
    persisted binary words
  - borrowed readers can access `binary_word_count()` /
    `collect_binary_words()`
- [src/am/build.rs](/home/peter/dev/tqvector/src/am/build.rs)
  - bulk build now writes persisted ADR-031 binary words on the supported
    no-QJL `4-bit` lane
  - initial page-fit checks account for the larger element tuple size
- [src/am/scan.rs](/home/peter/dev/tqvector/src/am/scan.rs)
  - scan-local graph caching now prefers the persisted binary sidecar when it
    exists and only derives from code bytes as fallback
- [src/am/insert.rs](/home/peter/dev/tqvector/src/am/insert.rs)
  - incremental inserts still write the old shape for now (`binary_words = []`)
  - read compatibility is preserved because decode accepts both tuple lengths

What did **not** land yet:

- no reloption or user-facing knob
- no insert-path sidecar write support yet
- no A/B comparison yet against the pre-sidecar cached-only branch

Validation:

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

All three gates are green for this checkpoint.

## Next Step

Install the release build into the scratch pg17 server, rebuild the real `50k`
fixture so element tuples actually contain persisted sidecars, then take the
first cold-path ADR-031 measurement. That will tell us whether persisted
sidecars are worth carrying beyond the already-good cached warm path.

## Cold Measurement

The first bounded cold read is now complete on the rebuilt persisted-sidecar
index:

```bash
scripts/bench_sql_latency_verified_scratch.sh \
  --prefix tqhnsw_real_50k \
  --m 8 \
  --ef-search 40 \
  --query-limit 1 \
  --cache-state cold-after-rebuild \
  --warmup-passes 0 \
  --session-mode per-query \
  --timing-mode plain-server \
  --output /tmp/adr031_persisted_real_50k_m8_ef40_cold_q1.summary
```

Observed output:

```text
m=8
ef_search=40
query_limit=1
cache_state=cold-after-rebuild
session_mode=per-query
timing_mode=plain-server
p50=5.537ms
p95=5.537ms
p99=5.537ms
mean=5.537ms
server_qps=180.60
wall=0.02s
```

Reference point:

- cached ADR-031 warm canonical `50k` seam from packet `282`:
  `p50=4.633ms`, `p99=7.661ms`, `mean=4.727ms`

This says the persisted-sidecar branch still starts in the same rough latency
band as the already-good warm cached path. What it does **not** say yet is how
much improvement came from persisted sidecars versus the prior cached-only
branch.

## Updated Next Step

The next question is no longer "does persisted ADR-031 work at all?" It does.

The next question is "is it worth carrying?" To answer that cleanly, the next
slice should produce an A/B comparison against the cached-only branch or add a
same-build toggle that forces binary-word derivation even when the sidecar is
present.

## Success Criteria

- the packet records the relevant storage/layout seams
- the packet makes a clear call on whether persisted ADR-031 sidecars are a
  contained extension or a format-change project
- the packet records the next implementation step or blocker explicitly
