# Task 138 Distinct-Recall Metric And Historical Re-Score

- Metric commit: `70ab4dfd0` (task-138 branch) — `distinct_recall@k`,
  `distinct_returned_count`, `ecaz bench rescore-identity`.
- Result head SHA: 26494644a plus the final result commit (this file's commit)
- Task bucket: `reviews/task-138/001-distinct-recall-rescore`
- Suite config (re-run cells): `artifacts/task138-n1024-b2-rerun-suite.json`
- Dry-run manifest: `artifacts/dryrun-manifest.json`
- Status: measurement complete; review requested (see `request.md`)

## What this packet contains

1. **Phase 0 evidence** — the runner now emits `distinct_recall@k` and
   `distinct_returned_{min,mean}` alongside the unchanged duplicate-tolerant
   `recall@k` on the recall table and spire-pipeline query metrics; identity
   JSONL records gain `distinct_returned_count`. Unit coverage in
   `recall.rs`, `spire_pipeline.rs`, `rescore_identity.rs` tests.
2. **Phase 1 inventory** — `artifacts/inventory.md`.
3. **Phase 2 re-score + re-runs**:
   - `artifacts/rescore/rescore-{10k,50k}-n128-b4-threshold-{off,on}.log`:
     `ecaz bench rescore-identity` over the Task 131 packet 027 identity
     JSONL. Corpus/queries TSVs: `data/task106_intel_dbpedia_staged` (10k),
     `data/task111a_real50k` (50k) — the same prepared sets packet 027 used.
     The re-scorer reproduces packet 027's published duplicate-tolerant
     recall to 4 decimals (0.9985 @10k, 1.0000 @50k), validating the
     brute-force truth path.
   - `artifacts/{10k,50k}-n1024-b2/` — fresh `spire-local-multinode` cells
     (nprobe=64, k=10, 200 queries, rabitq, `skip_fault_drills`) for the
     fine-list shape, which had no returned-ID artifacts anywhere. Bespoke
     config reason: the canonical lane config has no multi-instance steps;
     this packet needs the two n1024/b2 multi-instance cells only.
4. **Phase 3 conclusion re-ranking** — in `request.md`.

## Host / lane

- Intel desktop (WSL2), PG18.3 pgrx install, extension test-profile build
  from main (`a5afe0aa5` — no AM change in this task), CLI `target/debug/ecaz`
  from the task-138 branch. Matches the Task 123/131 multi-instance lane.
- Isolated one-index-per-table fixture instances, fresh per cell.

## Key Results

Corrected recall table (multi-instance production read, k=10; current =
duplicate-tolerant recall@10 as historically published; distinct =
distinct_recall@10):

- 10k n128/b4 nprobe=96 (packet 027 re-score, 200q): current 0.9985,
  distinct 0.5195, distinct_returned min 4 / mean 5.21, 183/200 dup queries.
- 50k n128/b4 nprobe=96 (packet 027 re-score, 1000q): current 1.0000,
  distinct 0.4146, min 4 / mean 4.15, 1000/1000 dup queries.
- 10k n1024/b2 nprobe=64 (fresh cell, 200q): current 0.9975, distinct 0.4930,
  min 4 / mean 4.94, 199/200 dup queries; latency p50/p95/p99
  537.928/569.824/684.124 ms.
- 50k n1024/b2 nprobe=64 (fresh cell, 200q): current 0.9980, distinct 0.5095,
  min 4 / mean 5.11, 200/200 dup queries; latency p50/p95/p99
  660.366/718.972/841.969 ms (within 0.6% of the Task 131 packet 024
  before-arms p50 663.809 ms).
- Cross-validation: the offline re-scorer reproduces packet 027's published
  recall to 4 decimals at both scales, and matches the live runner's new
  distinct columns exactly on both fresh cells.
- Threshold-off vs threshold-on packet 027 arms re-score identically
  (byte-identical identity files), preserving the Task 131 inter-arm no-op
  conclusion.
