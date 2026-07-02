# Task 137 Source-Identity Dedupe A/B (local multi-instance)

- Pre-registration head SHA: 060a00746 (task-137 branch)
- Result head SHA: TBD (filled at result commit)
- Task bucket: `reviews/task-137/001-source-identity-dedupe-ab`
- Suite config: `artifacts/task137-identity-ab-suite.json`
- Dry-run manifest: `artifacts/dryrun-manifest.json`
- Status: TBD

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

TBD — filled after the run.
