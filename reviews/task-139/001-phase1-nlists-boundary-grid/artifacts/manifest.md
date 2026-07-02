# Task 139 Phase 1: nlists x boundary Grid (local multi-instance)

- Pre-registration head SHA: TBD (filled at run start)
- Result head SHA: TBD
- Task bucket: `reviews/task-139/001-phase1-nlists-boundary-grid`
- Suite configs: `artifacts/task139-phase1-50k-suite.json`,
  `artifacts/task139-phase1-100k-suite.json`
- Status: pre-registered; gated on Task 137 packet 001 proving the
  identity-on distributed surface returns distinct results, and on Task 138
  packet 001 (distinct_recall@k metric) — both consumed by this branch.

## Matrix

- Runner: `ecaz bench suite` (`target/debug/ecaz`, task-139 branch =
  task-138 metric + task-137 loader wiring merged).
- Fixture: `spire-local-multinode`, fresh per cell, `rabitq`, k=10,
  200 prepared queries, fault drills skipped (measurement-only packet).
- Grid: nlists {128, 316, 512, 1024, 2048} x boundary_replica_count {0, 1, 2}
  at 50k and 100k (30 cells). All cells run `source_identity=include`
  (ADR-083): recall claims use `distinct_recall@k` per Task 138, and the
  distributed surface must return distinct rows for those claims to be
  meaningful.
- Sweep normalization (Phase 0): per-cell nprobe points are fractions of
  nlists — {3%, 6%, 12.5%, 25%} everywhere, plus {50%} for nlists <= 512 and
  the historical 75% anchor (nprobe=96) for nlists=128. Corpus-fraction
  scanned is additionally computed per cell from the production scan-profile
  row counters (rows available per query / total row-instances) in the
  readout table, alongside raw nprobe.
- Bespoke config reason (required by the standard-sweep convention): the
  canonical lane configs contain no multi-instance steps and the registered
  ec_spire default sweep is not comparable across nlists values; this task's
  Phase 0 explicitly requires fraction-normalized sweeps.
- training_sample_rows=50000 at both scales (Phase 2 owns saturation).

## Pre-Registered Decision Frame

Target to beat (task file): distinct_recall@10 >= 0.999 at <= 10-15% of
corpus row-instances scanned per query, with p50 at or under the current
n1024/b2 numbers (50k p50 663.809 ms per Task 131 packet 024; 100k p50
0.73-0.78 s per Task 123 packets 019/020 — both duplicate-tolerant-era
numbers, so they are ceilings, not matched-metric baselines) and storage
accounted. Frontier shapes feed Phase 2 lever saturation; promote / iterate /
shelve is decided in the Phase 4 closeout packet, not here.

## Artifacts

- `artifacts/task139-phase1-{50k,100k}-suite.json`
- `artifacts/dryrun-{50k,100k}-manifest.json`
- Per cell (`{scale}-n{nlists}-b{replicas}/bench-suite/`): `results.jsonl`,
  recall/latency logs, `production-read-k10-default-identity.jsonl`,
  `storage.log`

## Key Results

TBD.
