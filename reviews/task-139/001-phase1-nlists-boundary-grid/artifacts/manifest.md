# Task 139 Phase 1: nlists x boundary Grid (local multi-instance)

- Pre-registration head SHA: c12ad4d52
- Result head SHA: 19e32e5ef
- Task bucket: `reviews/task-139/001-phase1-nlists-boundary-grid`
- Suite configs: `artifacts/task139-phase1-50k-suite.json`,
  `artifacts/task139-phase1-100k-suite.json`
- Status: superseded — debug-build substrate. Wind-down feedback:
  `reviews/task-139/001-phase1-nlists-boundary-grid/feedback/2026-07-04-01-agent-ix.md`.

## Measurement Substrate Warning

All completed cells in this packet ran through the local multi-instance fixture
that installed `ecaz.so` from the dev profile (`cargo pgrx install --test` /
`Finished 'dev' profile [unoptimized + debuginfo]`). The packet is therefore
not decision-grade for absolute latency claims, and no p50/p95/p99 number here
may be cited against release baselines.

Usable with this caveat: distinct recall, scan-profile counters, selected-PID
counts, storage shape, and failure modes observed on the debug substrate.
Superseded work: all remaining Task 139 phases and the unrun 100k grid are
deferred to the remediation program filed as Tasks 141-146, with honest Pareto
confirmation owned by Task 146 after the release-fixture and bench-guard fixes.

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
  the historical 75% anchor (nprobe=96) for nlists=128.
- Constraint-driven reloption rule: `top_graph_search_list_size =
  max(96, cell max nprobe)` (96/158/256/256/512 for n128/n316/n512/n1024/
  n2048), because the AM rejects route counts above the top-graph search
  list size. The completed n128 row already satisfies this rule (96). The
  first n316 attempt with the uniform tgsl=96 failed on exactly that
  constraint and was rerun; tgsl interaction with recall stays a Phase 2
  lever on the frontier shape. Corpus-fraction
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

- Completed 50k result files: 12/15 cells. Successful cells cover nlists
  128/316/512/1024 x boundary_replica_count 0/1/2.
- Failed 50k cells: n2048/b0 and n2048/b1 both reached production read and
  failed with `remote_candidate_receive_failed` from node_id 2. Failure logs are
  preserved under `artifacts/50k-n2048-b{0,1}/`.
- Halted and pruned: n2048/b2 was interrupted per wind-down feedback before a
  completed result; its non-decision partial fixture/artifacts were removed.
- Not run: 100k grid and Task 139 phases 2-4. Do not treat this packet as a
  promote/iterate/shelve decision for a default distributed read shape.
