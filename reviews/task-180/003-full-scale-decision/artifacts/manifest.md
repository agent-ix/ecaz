# Task 180 packet 003 artifact manifest

This manifest covers the Phase 2 10k/50k/100k confirmation and final NFR-017
decision. Measurements are pending.

## Provenance and fixed shape

- Owning task / packet: `task-180` / `reviews/task-180/003-full-scale-decision/`
- Selected bounded candidate: persisted head cap 4096, search width 64, 64
  returned seeds, RaBitQ neighbor scoring, BW4/H100, graph degree 32.
- Comparators: unchanged production persisted width 32 / seeds 32 and the
  benchmark-only O(N) owner-scan oracle.
- Physical topology: three PG18 hash-shard owners, one index per source table,
  exact/disjoint ownership and remote materialization required at every scale.
- Corpus prefixes: `ec_real_10k`, `ec_real_50k`, `ec_real_100k` from
  `/home/peter/dev/ecaz/data/staged-current`.
- Measurement protocol: 200 held-out queries / 2,000 top-10 trials; 50 warm
  latency measurements after 10 warmups; concurrency 1.
- Installed extension: clean release build at SHA
  `53b62bbea7ce4be1bd8053daf504801f09b36352`; unanimity is enforced per step.

## Checked-in suite

- Config: `confirmation-suite.json`.
- Command template: `target/release/ecaz bench suite run --config reviews/task-180/003-full-scale-decision/artifacts/confirmation-suite.json --only <step> ...`.
- Disk-safe execution order: `confirm-10k`, prune stopped run directory;
  `confirm-50k`, prune; `confirm-100k`, prune. Each selected step gets its own
  suite manifest/results/report/status and all use the same checked-in config.
- Status: audit and dry-run expansion pass; 10k succeeded; 50k/100k pending. Durable
  outputs: `confirmation-audit.log` and `confirmation-dry-run.log`.

## 10k confirmation

- Timestamp: 2026-07-15 01:20-01:24 PDT.
- Status: one succeeded selected step, no failures/missing/stale artifacts; all
  five step-local thresholds pass.
- Query SHA-256: `a2c191bb742017d849e73f6e6866e8e0f0bac1579ba212f7fc76b8eb09904ae8`.
- Topology: ready/published owner rows 3323/3391/3286 = 10,000 exactly;
  zero non-owned rows/orphans; two remote materialization probes pass.

| Variant | Distinct recall@10 (95% CI) | Warm p50 / p95 / p99 | Head cache |
| --- | ---: | ---: | ---: |
| production width32/seeds32 | 0.9990 (0.9964-0.9997) | 33.50 / 38.40 / 45.00 ms | 25,794,572 B |
| owner oracle | 0.9995 (0.9972-0.9999) | 252.00 / 262.60 / 272.70 ms | shared |
| bounded width64/seeds64 | 0.9990 (0.9964-0.9997) | 33.50 / 39.10 / 42.80 ms | shared |

The bounded candidate passes the 0.9990 recall floor at 10k. Its p95/p50 ratio
is 1.167. Physical generation / control / coordinator source / same-run single
index bytes are 242,745,344 / 24,576 / 166,699,008 / 115,687,424.

Durable sources: `confirmation/suite-manifest-10k.json`,
`confirmation/results-10k.jsonl`, `confirmation/10k/distann-multinode-summary.log`,
the cited per-arm recall/latency tables, `confirmation-10k-report.md`,
`confirmation-10k-status.log`, and `confirmation-10k-suite.log`. Node logs,
duplicate full fixture log, and stopped run directory were pruned. Checksums are
in `checksums.sha256`.

Corpus TSVs, truth caches, node PostgreSQL logs, duplicate full fixture logs,
and regenerable run directories will not be committed.
