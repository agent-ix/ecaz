# Packet 172/001 artifacts manifest

- task bucket / packet: reviews/task-172/001-real-multinode-benchmark
- head SHA: 63862c9f4 (or later on task-165-ec-distann-m3)
- surface: **real** 3×PG18 local multi-instance ec_distann, release `.so`
  installed on the shared pgrx-install (`cargo pgrx install --release --features
  pg18`), orchestrated by `target/debug/ecaz` (orchestration only — all index
  work runs in the release `.so`). Ports 39710/39711/39712.
- corpus: staged real DBpedia `ec_real_10k` (`data/staged-current/`), 1536-dim,
  10000 corpus rows + 200 held-out queries. Corpus sha256 recorded in
  `data/staged-current/ec_real_10k_manifest.json`
  (`c67c5810b66d982d705974e48d4775479adfbd92a988f694091266e049a35e75`).
- storage surface: replicated global graph per node + partitioned-heap disjoint
  drill (NOT physically disjoint index storage).
- NON-STANDARD config rationale: this is a bespoke multinode lane (not the
  canonical per-lane sweep) because Task 172 requires the distributed 3-instance
  path, which the standard single-node lane configs do not exercise.

## Artifacts

- `distann-real-multinode.json` — the `ecaz bench suite` config (3 scales;
  10k run here, 50k/100k deferred per the reviewer's guidance).
- `distann-real-multinode-10k/distann-local-multinode.log` — the suite step
  fixture log (the source the result rows parse from).
- `distann-real-multinode-10k/distann-multinode-summary.log` — recall-only
  summary with honest measured provenance (rows=10000 dim=1536).
- `clean-10k/distann-multinode-summary.log` — clean fresh-dir recall-only run on
  the reverted (tie-break-deferred) release `.so`.
- `full-10k-fixture.log` — full fault-matrix run: 12 NFR-020 drills fail-closed,
  concurrency/retention/AC-5 pass, recovery clean, disjoint prune to ~1/3.
- `results.jsonl` — normalized rows: `distinct_recall_identity`,
  `suite_recall_gate`, `storage_node`×3, `storage_summation`, `drill_outcome`×2.
- `suite-manifest.json` — suite run manifest.

## Key result lines cited by request.md

- `RECALL_RESULT n_queries=200 identical=200 mismatched_ids=0`
- `suite_recall_gate single=0.9990 multi=0.9990 delta=0.0000 pass=true`
- `qual_correctness single_n=10 multi_n=10 mismatch=0 pass=true`
- `storage_summation nodes=3 cluster_index_bytes=347062272 corpus_rows=10000
  dim=1536 cluster_index_space_amplification=5.6488`
- `GATE PASS: recall identical; 12 faults fail-closed; recovery clean`

## Provenance notes

- This is development preflight, NOT promotable Task 172 gate evidence. Missing
  surfaces (latency, throughput, telemetry, capacity, release-verified end-to-end
  suite, 50k/100k) are enumerated in `request.md`.
- Node PostgreSQL operational logs are intentionally excluded (regenerable
  operational exhaust, per repo policy).
