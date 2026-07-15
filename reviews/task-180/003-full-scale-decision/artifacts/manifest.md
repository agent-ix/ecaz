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
- Status: audit and dry-run expansion pass; measurements pending. Durable
  outputs: `confirmation-audit.log` and `confirmation-dry-run.log`.

Corpus TSVs, truth caches, node PostgreSQL logs, duplicate full fixture logs,
and regenerable run directories will not be committed.
