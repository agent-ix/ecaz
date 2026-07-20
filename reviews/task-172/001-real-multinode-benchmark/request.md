# Task 172 packet 001 — real multi-instance ec_distann (IN PROGRESS)

**Status: development preflight, NOT a promotable gate.** This packet extends the
local multi-instance fixture to real staged corpora and lands the correctness +
provenance fixes from the pre-checkpoint review
(`feedback/2026-07-09-01-reviewer.md`). It does NOT yet implement the full Task
172 measurement surface (latency/throughput/telemetry/capacity). Do not promote.

## What this packet delivers

### Real-corpus distributed lane (the fixture extension you called the right direction)

`ecaz dev distann-multicluster local-multinode-pg18 --corpus-prefix ec_real_10k`
loads the staged DBpedia TSVs into each node (`dm`/`dm_queries`, standard
`encode_to_ecvector(source, 4, 42)`), and the `distann-local-multinode` suite
step passes `corpus_prefix`/`staged_dir` through. Real 3×PG18, 1536-dim.

### Real-10k results (release `.so`, clean fresh artifact dir `clean-10k/`)

- distinct-recall identity single-vs-multinode: `n_queries=200 identical=200
  mismatched_ids=0`.
- absolute recall@10 (held-out real queries): `single=0.9990 multi=0.9990
  delta=0.0000` (vs the prior synthetic lane's meaningless 0.08).
- `qual_correctness mismatch=0` (WHERE on a non-projected column + LIMIT).
- FR-082 published-epoch read consumption pass.
- Full-fixture (fault matrix) also GATE PASS: all 12 NFR-020 fault drills
  fail-closed, concurrency + retention + AC-5 pass, recovery clean, disjoint
  drill prunes each node to ~1/3 (10000→~3300) with an identical signature.
- cluster storage summation: `cluster_index_bytes=347,062,272` over 3 nodes =
  **5.65× raw-vector space amplification** (replicated 3×). This is a real
  finding: the replicated lane exceeds NFR-018's 4.0× and is a
  replicated-index/partitioned-heap control, NOT disjoint index storage.
- `results.jsonl` now non-empty (027-P1 end-to-end) with structured
  `distinct_recall_identity`, `suite_recall_gate`, `storage_node`,
  `storage_summation`, and `drill_outcome` rows.

### Six real-data bugs found and fixed by running on real vectors

1. 011-P1 distributed CustomScan qual+LIMIT served the unproven tail → diverged
   from single-node (`mismatch=1`). Bounded to the proven prefix (`d4bd980dc`) +
   raw-boundary/cap hardening.
2. `remote_content_divergence` drill rebuilt at `graph_degree+8` (=40), which
   overflows the ec_distann node-record 8 KB page budget on 1536-dim. Diverge
   downward. (Also a real constraint: ec_distann caps graph_degree for high-dim
   corpora — see verdict.)
3. `co_placement_drift` recovery re-ran the synthetic setup → dim-16 index vs
   1536-dim query. Use the real-aware setup.
4. concurrency + AC-5 drills inserted synthetic dim-16 vectors into real `dm`.
   Reuse a real corpus vector.
5. 172-P1 false provenance (`rows=2000 dim=16` on a real run) → report measured
   rows/dim + corpus label.
6. 172-P2 COPY path not SQL-escaped → escape single quotes.

## What this packet does NOT deliver (open Task 172 requirements)

Per your P0/P1 findings, these remain and are the next work, before 50k/100k
host time:

- distributed **latency** sweep (p50/p95/p99) at every scale/sweep;
- **throughput/concurrency** curve (1,2,4,8,16) + saturation/bottleneck;
- **remote-engagement counters** (expand/materialize calls, owned rows, bytes)
  as first-class result rows — the current `drill_outcome` free-text label is
  intentionally shallow and is NOT sufficient telemetry;
- benchmark-mode vs full-metrics-mode + instrumentation-overhead audit;
- load/build timing rows; 1m/10m capacity model;
- a clean **release-verified** end-to-end suite run on every node (the current
  runs use `target/debug/ecaz` as the orchestrator with the release `.so`);
- 50k/100k gate runs (deferred until the metadata + measurement surfaces above
  are clean, per your guidance);
- true disjoint index storage (still replicated-index/partitioned-heap).

The lifecycle/DML semantics findings (mid-delete abort safety, real
Building/Published epoch generations, distributed DML routing) are NOT closed by
the dimension fixes and are tracked with Tasks 165/167.

## Artifacts

- `distann-real-multinode.json` — suite config (3 scales).
- `clean-10k/` — clean release-`.so` recall-only run (summary + log).
- `results.jsonl`, `suite-manifest.json` — 10k suite run.
- `manifest.md` — provenance.
