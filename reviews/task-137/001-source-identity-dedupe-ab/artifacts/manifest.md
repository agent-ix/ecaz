# Task 137 Source-Identity Dedupe A/B (local multi-instance)

- Pre-registration head SHA: 060a00746 (task-137 branch)
- Result head SHA: the commit adding this Key Results section (code at
  `51ca0f228`, ADR at `060a00746`, pre-registration at `3870787b3`)
- Task bucket: `reviews/task-137/001-source-identity-dedupe-ab`
- Suite config: `artifacts/task137-identity-ab-suite.json`
- Dry-run manifest: `artifacts/dryrun-manifest.json`
- Status: complete — all six cells ran; the fix meets every pre-registered
  criterion at all three scales; review requested

## Matrix

- Runner: `ecaz bench suite` (`target/debug/ecaz`, task-137 branch build; the
  installed PG18 extension is unchanged from main — this change is loader/DDL
  wiring only).
- Fixture: `spire-local-multinode` (coordinator + 3 remote PG18 instances),
  fresh per cell.
- Scale/index: `10k`, `50k`, `100k`, all `n128/b4` representative shape
  (`nlists=128, recursive_fanout=8, top_graph_* standard, boundary_replica_count=4`).
- Storage: `rabitq`. `k=10`, `nprobe=96`, 200 prepared queries per cell.
- Variant axis (the change under test, A/B in isolation):
  - `identity-off`: current default — node-local `0x01` vec_ids, independent
    per-node remote indexes. This is the pre-fix arm and reproduces the
    Task 131 packet 027 duplicate defect.
  - `identity-on`: `--reloption source_identity=include` — corpus tables gain
    the stored generated 16-byte identity column
    (`sha256(int8send(id))[..16]`), all indexes built with
    `INCLUDE (source_identity)`, so every copy of a corpus row carries the
    same global `0x02` vec_id and the final merge dedupes replicas
    cross-node (ADR-083).
- Fault drills skipped (`skip_fault_drills=true`): this packet measures result
  identity, recall, latency, and storage for the identity provider A/B; the
  strict/degraded read path is not touched by the change (no AM code change).
- Query counts: 200 per cell at every scale (packet 027 used 1000 at 50k; the
  pre-fix duplicate evidence at 1000q is cited from packet 027 directly).

## Pre-Registered Decision Rule

The fix is correct if, at every scale:

1. `identity-on` returns **zero duplicate returned IDs** in every top-10
   result (identity JSONL `distinct_returned_count == returned_count`), while
   `identity-off` reproduces the duplicate defect;
2. `identity-on` `distinct_recall@10` is at least `identity-off`
   `distinct_recall@10` (duplicates can only hide real neighbors, never add
   them);
3. latency p50/p95/p99 and storage deltas are reported and either neutral or
   explicitly accounted (the identity column adds 16 bytes/row heap + INCLUDE
   payload per index entry).

Latency/recall regressions in `identity-on` beyond run-to-run noise, or any
duplicate IDs in the `identity-on` arm, reject the fix.

## Artifacts

- `artifacts/task137-identity-ab-suite.json` — suite config
- `artifacts/dryrun-manifest.json`
- `artifacts/suite-manifest.json`, `artifacts/suite-run.log` (TBD)
- Per cell (`{10k,50k,100k}-identity-{off,on}/bench-suite/`):
  `production-read-k10-default.log`,
  `production-read-k10-default-identity.jsonl`, `results.jsonl`,
  `storage.log`, `suite-run.log`
- `artifacts/rescore/` — `ecaz bench rescore-identity` outputs for both arms
  at each scale (distinct_recall@10 vs duplicate-tolerant recall@10)

## Key Results

Distinct/duplicate metrics from `ecaz bench rescore-identity` (task-139
integration binary, brute-force truth from the same prepared TSVs; logs under
`artifacts/rescore/`). Latency from the coordinator query-metrics table in
each cell's `production-read-k10-default-default.log`; per-vector index bytes
from each cell's `storage.log`.

| Scale | Arm | recall@10 (dup-tolerant) | distinct_recall@10 | distinct min/mean | dup queries | latency p50/p95/p99 (ms) | per-vector index bytes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 10k | identity-off | 0.9985 | 0.5195 | 4 / 5.21 | 183/200 | 595.742 / 674.392 / 883.210 | 4256.6 |
| 10k | identity-on | 0.9855 | 0.9855 | 10 / 10.00 | 0/200 | 598.089 / 732.271 / 855.905 | 4701.4 |
| 50k | identity-off | 1.0000 | 0.4115 | 4 / 4.12 | 200/200 | 2633.331 / 3042.619 / 3485.029 | 4129.1 |
| 50k | identity-on | 0.9730 | 0.9730 | 10 / 10.00 | 0/200 | 2588.074 / 3191.022 / 3461.909 | 4581.5 |
| 100k | identity-off | 1.0000 | 0.4265 | 4 / 4.26 | 200/200 | 5474.397 / 6369.231 / 6645.503 | 4112.6 |
| 100k | identity-on | 0.9810 | 0.9810 | 10 / 10.00 | 0/200 | 5397.103 / 6241.625 / 6493.409 | 4566.9 |

Pre-registered decision rule readout:

1. Zero duplicate returned IDs in every identity-on result at every scale
   (0/600 queries), while every identity-off arm reproduces the defect —
   the 10k off arm matches Task 131 packet 027 exactly (183/200 duplicated,
   distinct 0.5195). SATISFIED.
2. identity-on distinct_recall@10 (0.9855/0.9730/0.9810) far exceeds
   identity-off distinct_recall@10 (0.5195/0.4115/0.4265) at every scale —
   true recall roughly doubles because slots previously wasted on replica
   copies now carry real neighbors. SATISFIED.
3. Latency: p50 delta +0.4% (10k), -1.7% (50k), -1.4% (100k) — neutral within
   run noise; off arms track the historical packet 027/task-123 baselines.
   Storage: per-vector index bytes +10.4%/+11.0%/+11.0% — the 16-byte
   identity carried in each assignment copy (primary + 4 boundary replicas at
   b4) plus the wider global vec_id; explicitly accounted. Coordinator load
   (100k): 976.77s identity-on vs 951.92s identity-off (+2.6%). SATISFIED.

Additional observations:

- Duplicate-tolerant recall vs distinct recall converge exactly in every
  identity-on arm (the signature of a duplicate-free surface).
- The honest distinct recall of the representative shape at nprobe=96 is
  ~0.97-0.99, not the 1.0000 history published; the delta is duplicate
  inflation, now quantified per scale.
- Strict/degraded semantics untouched: no AM code changed in this task; fault
  drills were not rerun (the read path is byte-identical code).
