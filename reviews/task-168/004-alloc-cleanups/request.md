# Review request: Task 168 Phase 4 — alloc cleanups + rabitq default flip

- Branch: `task-168-diskann-batched-beam`
- Commits: `fa0861ddd` (pooled decode + buffer reuse + TidHasher),
  `4050ea830` (StorageFormat::DEFAULT → RaBitQ), `87d106d52` (hasher
  avalanche fix).
- Evidence: `artifacts/manifest.md` — four A/B arms against the packet-002
  W=4 baseline; decision arm on `87d106d52` with the full recall+latency
  protocol.

## Summary

- **Landed**: zero-steady-state-allocation beam loop (pooled
  `VamanaNodeTuple::decode_into` + recycled neighbor Vecs + reused score
  buffer) and a multiply-shift TID hasher. Decision arm: −4.6 to −7.5% at
  L≥400 across scales, recall bit-identical at every cell.
- **Debug story worth reading** (manifest findings 2–3): the first arm
  regressed up to +29%. Two causes untangled: (a) the hasher's unmixed low
  bits clustered hashbrown probes — fixed by folding the high half into
  `finish()`; (b) latency-only suite runs inflate the first sweep point by
  10–18% — A/B arms must replicate the baseline's step protocol.
- **Default flip**: `ec_diskann` now builds rabitq by default (the
  benchmarked codec); the prefilter-kind override pg_test pins
  `pq_fastscan` explicitly since it tests that lane's sidecar switching.
- Skipped with evidence: frontier-heap bounding (<3% of residual),
  scan-lifetime node cache (dedup guarantees single read per node/scan).
- Pre-existing failure flagged (NOT from this task):
  `diskann_turboquant_prepared_prefilter_batch_scores_and_records_counters`
  fails identically on unmodified main-derived src on this host —
  reviewer may want a tracking task.

## Asks

1. Approve landing the Phase 4 bundle (three commits above).
2. Concur that the 50k L=64/128 +4.5% cells are noise (see the per-arm
   spread in the manifest) rather than a real low-L cost.
3. Decide whether the pre-existing turboquant counters failure gets its
   own task.
