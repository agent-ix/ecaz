# Artifact Manifest — SPIRE AWS Prep Local Verification

Packet: `reviews/task-30/957-spire-aws-prep-local-verification/`
Owner: coder B (SPIRE AWS lane)
Branch: `task-30-phase13-spire-aws-prep`

## Cluster Identity

| Field       | Value                                                 |
| ----------- | ----------------------------------------------------- |
| Role        | SPIRE-only verification (separate from IVF work)      |
| PG version  | 18.3 (pgrx-built `~/.pgrx/18.3/pgrx-install`)         |
| PGDATA      | `/home/peter/pg18-spire/data/`                        |
| Port        | `28919`                                               |
| Socket dir  | `/home/peter/pg18-spire/sock/`                        |
| Database    | `spire_aws_prep`                                      |
| Extensions  | ecaz 0.1.1 (pg_test feature build, head SHA below)    |
| Head SHA    | branch `task-30-phase13-spire-aws-prep` @ first commit |
| ecaz.so sha256 | `979ce28838c5c702124bba3c92d7b91b30c1834890c22130584b4660b3723ff8` |

## Artifacts

| # | Artifact | Scale | Surface | Command | Timestamp | Key result |
|---|----------|-------|---------|---------|-----------|-----------|
| 1 | `logs/10k-generate.log`, `logs/10k-generate-queries.log` | 10k | corpus generate | `ecaz corpus generate --n 10000 --dim 1536 --seed 42` (+ 100 queries seed 4242) | 2026-05-22T20:03 PDT | wrote 10000 × dim 1536 + 100 queries |
| 2 | `logs/10k-load.log` | 10k | corpus load (ec_spire + bits=4 RaBitQ) | `ecaz corpus load --prefix spire10k --profile ec_spire --bits 4` | 2026-05-22T20:13 PDT | built spire10k_idx in 14.79s; 169 MB table / 9104 kB index |
| 3 | `logs/10k-knn-smoke.log` | 10k | single-node kNN smoke (`embedding <#> source`) | `psql` direct, top-10 kNN | 2026-05-22T20:14 PDT | top-10 ids returned; planner falls through to seq scan (expected: no remotes, no CustomScan injection) |
| 4 | `logs/install-pg-test.log` | n/a | install pg_test feature build | `ecaz dev install ecaz-pg-test --pg 18` | 2026-05-22T20:21 PDT | backend artifact assertion passed; ecaz.so sha256 above |
| 5 | `multicluster-smoke-baseline/{multicluster-smoke-success.log,coord-postgres.log,remote-postgres.log}` | n/a | `ecaz dev spire-multicluster smoke-pg18` | (fixture-internal CREATE INDEX) | 2026-05-22T20:22 PDT | **FAIL** — `ec_spire remote search executor endpoint_status requires_rabitq_storage_format is not ready` (see Findings) |

## Findings

### F1: Multicluster smoke fixture missing `storage_format='rabitq'` (Phase 13 entry-gate blocker)

`scripts/run_spire_multicluster_pg18_smoke.sh:150-163` creates the smoke
indexes with `WITH (nlists = 1)` only. SPIRE remote-search now requires
the index to be built with `storage_format = 'rabitq'`
(`src/am/ec_spire/coordinator/remote_candidates/endpoint_identity.rs:476`).
The fixture errors before it can exercise any remote round-trip:

    ERROR:  ec_spire remote search executor endpoint_status
            requires_rabitq_storage_format is not ready

The recommendation surfaced in the SPIRE code is literally:
"create or reindex the remote-serving SPIRE index with
`storage_format = 'rabitq'`". The fixture does neither.

This breaks the Phase 13 entry-gate requirement "Final local
production-readiness bundle passes from clean setup" — the canonical
SPIRE multicluster smoke does not pass on a freshly installed
extension. Every downstream multicluster fixture
(`customscan-read-pg18`, `insert-read-after-customscan-pg18`,
`transport-overlap-pg18`, `fault-pg18`, `lifecycle-pg18`) likely
inherits the same defect and would fail the same way. AWS spend
gated on this gate item should not start until the fixture set is
updated and re-validated locally.

A secondary surface: the fixture also surfaces a `NOTICE` that the
auto-spun remote has `max_prepared_transactions = 0`, which would
block coordinator-routed SPIRE writes. The runbook calls out a
non-zero `max_prepared_transactions` as required GUC — the local
fixture's remote bootstrap (`scripts/bootstrap-node.sh` for AWS,
or the equivalent bootstrap in the multicluster fixture) does not
set it. This may need to be fixed alongside F1.

## Snapshots

Filesystem snapshots of `$PGDATA` taken with `pg_basebackup -Ft -z`
after the cluster reaches a clean checkpoint at each scale. Stored
outside the packet when size makes inclusion impractical; the table
below records absolute path + sha256 + size.

| Scale | Snapshot path | sha256 | Size | Created |
|-------|--------------|--------|------|---------|
| 10k   | `/home/peter/pg18-spire/snapshots/10k/base.tar.gz` + `pg_wal.tar.gz` + `backup_manifest` | base.tar.gz `b6f6b4047af4ab7890b056434d5588c558e69c41041194bf18ec9289603eff68`; pg_wal.tar.gz `8004890399b9a764d6c2025c96fd4760442c457bfe1719dc2758b4e1c7976c7a` (see `/home/peter/pg18-spire/snapshots/10k/sha256.txt`) | 121 MB | 2026-05-22T20:15 PDT |
| 50k   | _TBD_        |        |      |         |
| 100k  | _TBD_        |        |      |         |
| 1M    | _TBD_        |        |      |         |
