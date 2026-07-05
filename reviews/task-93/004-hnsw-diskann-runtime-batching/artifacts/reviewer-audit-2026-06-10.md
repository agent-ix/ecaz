# Reviewer Audit: Task 93 Packet 004

Date: 2026-06-10
Reviewer: codex

## Local Status

The packet directory existed locally but had no `request.md` and no
`artifacts/manifest.md`.

Command:

```text
find reviews/task-93/004-hnsw-diskann-runtime-batching -maxdepth 2 -type f -name 'request.md' -o -name 'manifest.md' -print
```

Observed output: empty.

## HNSW Counter Evidence

`reviews/task-93/004-hnsw-diskann-runtime-batching/artifacts/latency-hnsw-rabitq-real10k-kernel-on.log`
contains no `[block-kernel-counters]` rows and reports zero legacy Task 87
HNSW counters:

```text
[task87-counters] command=latency label=ef_search=80 surface=hnsw flushes=0 candidates=0 elapsed_nanos=0 elapsed_ms=0.000000 lut32_flushes=0 lut32_candidates=0
```

By contrast, the DiskANN kernel-on log does contain RaBitQ block-kernel rows:

```text
[block-kernel-counters] command=latency label=list_size=128 surface=diskann quant=rabitq isa=neon flushes=34 candidates=1088 ...
[block-kernel-counters] command=latency label=list_size=128 surface=diskann quant=rabitq isa=scalar flushes=3990 candidates=38265 ...
```

## Code Under Review

Current HEAD during this audit was `1516996f8880c51591eb263d4a4c6e3bdacf9af4`,
which wires RaBitQ batching into the binary-prefilter survivor branch in
`src/am/ec_hnsw/scan.rs` around the `rabitq_batch_eligible` check and the final
`flush_rabitq_search_code_batch(...)` call.

The existing HNSW artifacts therefore predate the current HEAD's HNSW fix and
cannot be used as evidence that the new HNSW path executes.
