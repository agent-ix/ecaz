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
| Head SHA    | branch `task-30-phase13-spire-aws-prep` @ `3ae638b7` (script fixes for F1) |
| ecaz.so sha256 | `979ce28838c5c702124bba3c92d7b91b30c1834890c22130584b4660b3723ff8` (unchanged across packet — script-only fixes do not rebuild the extension) |
| Tablespaces | `spire_ts1..ts4` at `/home/peter/pg18-spire/tablespaces/ts{1,2,3,4}/` (sibling dirs, same physical disk — functional multi-disk only; perf is the AWS lane) |
| terraform   | `1.9.8` installed at `/home/peter/.local/bin/terraform` for preflight target |

## Artifacts

| # | Artifact | Scale | Surface | Command | Timestamp | Key result |
|---|----------|-------|---------|---------|-----------|-----------|
| 1 | `logs/10k-generate.log`, `logs/10k-generate-queries.log` | 10k | corpus generate | `ecaz corpus generate --n 10000 --dim 1536 --seed 42` (+ 100 queries seed 4242) | 2026-05-22T20:03 PDT | wrote 10000 × dim 1536 + 100 queries |
| 2 | `logs/10k-load.log` | 10k | corpus load (ec_spire + bits=4 RaBitQ) | `ecaz corpus load --prefix spire10k --profile ec_spire --bits 4` | 2026-05-22T20:13 PDT | built spire10k_idx in 14.79s; 169 MB table / 9104 kB index |
| 3 | `logs/10k-knn-smoke.log` | 10k | single-node kNN smoke (`embedding <#> source`) | `psql` direct, top-10 kNN | 2026-05-22T20:14 PDT | top-10 ids returned; planner falls through to seq scan (expected: no remotes, no CustomScan injection) |
| 4 | `logs/install-pg-test.log` | n/a | install pg_test feature build | `ecaz dev install ecaz-pg-test --pg 18` | 2026-05-22T20:21 PDT | backend artifact assertion passed; ecaz.so sha256 above |
| 5 | `multicluster-smoke-baseline/{multicluster-smoke-success.log,coord-postgres.log,remote-postgres.log}` | n/a | `ecaz dev spire-multicluster smoke-pg18` (pre-fix) | (fixture-internal CREATE INDEX) | 2026-05-22T20:22 PDT | **FAIL** — `requires_rabitq_storage_format is not ready` (F1, **superseded** by row 6 after script patch in commit `3ae638b7`) |
| 6 | `multicluster-smoke-pg18-10k/multicluster-smoke-success.log` + `multicluster-sweep-10k.log` | n/a | `ecaz dev spire-multicluster smoke-pg18 --pg 18 --skip-install --artifact-dir ...` | (fixture-internal CREATE INDEX, post-patch) | 2026-05-22T21:53 PDT | **PASS** — `SPIRE multicluster PG18 smoke passed`; F1 resolved by adding `storage_format='rabitq'`, real `profile_fingerprint` from `ec_spire_remote_search_endpoint_identity`, and `max_prepared_transactions=10` (commit `3ae638b7`) |
| 7 | `multicluster-customscan-read-pg18-10k/multicluster-smoke-success.log` | n/a | `ecaz dev spire-multicluster customscan-read-pg18 --pg 18 --skip-install --artifact-dir ...` | (fixture-internal CREATE INDEX, already-correct) | 2026-05-22T21:47 PDT | **PASS** — `SPIRE multicluster CustomScan read passed`; CustomScan plan injected, remote heap candidates returned via libpq |
| 8 | `multicluster-insert-read-after-customscan-pg18-10k/multicluster-smoke-success.log` | n/a | `ecaz dev spire-multicluster insert-read-after-customscan-pg18 --pg 18 --skip-install --artifact-dir ...` | (fixture-internal CREATE INDEX, already-correct) | 2026-05-22T21:48 PDT | **PASS** — `SPIRE multicluster coordinator insert read-after-CustomScan passed`; remote_insert_prepared_pending_local_commit → ready → CustomScan reads new row |
| 9 | `multicluster-transport-overlap-pg18-10k/multicluster-smoke-success.log` | n/a | `ecaz dev spire-multicluster transport-overlap-pg18 --pg 18 --skip-install --artifact-dir ...` | (no CREATE INDEX; uses `ec_spire_test_production_transport_probe`) | 2026-05-22T21:48 PDT | **PASS** |
| 10 | `multicluster-fault-pg18-10k/multicluster-smoke-success.log` | n/a | `ecaz dev spire-multicluster fault-pg18 --pg 18 --skip-install --case local_cancel --artifact-dir ...` | Stage E fault matrix, case `local_cancel` (representative) | 2026-05-22T21:53 PDT | **PASS** — representative case only; exhaustive 11-case matrix is its own validation lane |
| 11 | `multicluster-lifecycle-pg18-10k/multicluster-smoke-success.log` | n/a | `ecaz dev spire-multicluster lifecycle-pg18 --pg 18 --skip-install --case drop_remote_index_before_fanout --artifact-dir ...` | Stage E lifecycle matrix, case `drop_remote_index_before_fanout` (representative) | 2026-05-22T21:53 PDT | **PASS** — representative case only |
| 12 | `logs/50k-generate.log`, `logs/50k-generate-queries.log` | 50k | corpus generate | `ecaz corpus generate --n 50000 --dim 1536 --seed 42` (+ 100 queries seed 4242) | 2026-05-22T21:54 PDT | wrote 50000 × dim 1536 + 100 queries to `/home/peter/pg18-spire/corpora/spire50k_*.tsv` |
| 13 | `logs/50k-load.log` | 50k | corpus load (ec_spire + RaBitQ + nlists=128) | `ecaz corpus load --prefix spire50k --profile ec_spire --bits 4 --storage-format rabitq --reloption nlists=128` | 2026-05-22T21:55 PDT | built `spire50k_rabitq_idx` in 40.09s; table 6680 kB / index 41 MB; load completed in 78.32s (see F2 for page-overflow at nlists=1) |
| 14 | `logs/50k-knn-smoke.log` | 50k | single-node kNN smoke (`embedding <#> encode_to_ecvector(source,4,42)`) | `psql` direct, top-5 kNN | 2026-05-22T21:56 PDT | top-5 ids returned `{46153, 22968, 2962, 38695, 38001}`; planner falls through to seq scan (expected: no remotes, no CustomScan injection) |
| 15 | `logs/50k-snapshot.log` | 50k | `pg_basebackup -Ft -z` | `pg_basebackup -h ... -p 28919 -D /home/peter/pg18-spire/snapshots/50k -Ft -z -P --manifest-checksums=SHA256` | 2026-05-22T21:57 PDT | base.tar.gz 692 MB, sha256 `86ceb9df…`; see Snapshots table |
| 16 | `logs/100k-generate.log`, `logs/100k-generate-queries.log` | 100k | corpus generate | `ecaz corpus generate --n 100000 --dim 1536 --seed 42` (+ 100 queries seed 4242) | 2026-05-22T22:01 PDT | wrote 100000 × dim 1536 + 100 queries to `/home/peter/pg18-spire/corpora/spire100k_*.tsv` |
| 17 | `logs/100k-load.log` | 100k | corpus load (ec_spire + RaBitQ + nlists=128 + 4 tablespaces) | `ecaz corpus load --prefix spire100k --profile ec_spire --bits 4 --storage-format rabitq --reloption nlists=128 --reloption local_store_count=4 --reloption local_store_tablespaces=spire_ts1,spire_ts2,spire_ts3,spire_ts4` | 2026-05-22T22:06 PDT | built `spire100k_rabitq_idx` in 74.37s; load completed in 150.71s; index main relation 32 kB, corpus table 13 MB (rest of storage lives in tablespaces — see row 18) |
| 18 | `logs/100k-multidisk-verify.log` | 100k | multi-disk file placement audit + kNN smoke | `find tablespaces/ts{1..4} -type f`, `du -sb`, kNN query | 2026-05-22T22:07 PDT | **PASS** — each tablespace holds a SPIRE local-store relfile pair (`<oid>` + `<oid>_fsm`): ts1=19 MB, ts2=23 MB, ts3=18 MB, ts4=20 MB (≈80 MB total split fairly evenly across 4 tablespace locations). kNN top-5 `{46153, 79286, 64711, 71065, 22968}`. **`local_store_tablespaces` reloption confirmed end-to-end functional.** |
| 19 | `logs/100k-snapshot.log` | 100k | `pg_basebackup -Ft -z` including tablespaces | same as row 15 (different output dir) | 2026-05-22T22:08 PDT | 5 tarballs (base + 4 tablespaces) + pg_wal.tar.gz, total 1.9 GB; per-tablespace sha256 in Snapshots table |
| 20 | `logs/preflight.log` | n/a | AWS preflight (local-only, no spend) | `make -C infra/spire-aws preflight` after installing `terraform 1.9.8` to `~/.local/bin/` | 2026-05-22T21:58 PDT | **PASS** — `terraform fmt -check` ✓, `terraform init -backend=false` ✓ (downloads providers aws v5.100.0, random v3.9.0 from public registry to local .terraform/), `terraform validate` ✓ (`Success! The configuration is valid.`), `bash -n scripts/spire-aws/*.sh` ✓, `jq empty` over three suite JSONs ✓; `shellcheck` not installed locally, skipped |

## Findings

### F1: Multicluster smoke fixture missing `storage_format='rabitq'` — RESOLVED in `3ae638b7`

Status: **fixed and re-verified.** Three SPIRE multicluster fixtures
(`smoke-pg18`, `stage_e_predispatch_fault_pg18`,
`stage_e_network_partition_pg18`) built CREATE INDEX without
`storage_format = 'rabitq'`, so remote-search rejected the index at
`requires_rabitq_storage_format`. Commit `3ae638b7` adds the reloption
to every SPIRE CREATE INDEX in those three scripts, mirrors
`max_prepared_transactions=10` on `pg_ctl start` (as in
`insert-read-after-customscan-pg18`), and replaces the smoke fixture's
hardcoded `'01'` profile-fingerprint with the real one fetched from
`ec_spire_remote_search_endpoint_identity` (as in
`customscan-read-pg18`). All 6 multicluster fixtures now pass at 10k
against a fresh PG18 cluster pair (see rows 6–11).

### F2: ec_spire RaBitQ build page-overflow at small/large `nlists` — needs design follow-up

Status: **open, surfaced by 50k/100k watch lanes.** At dim 1536, the
top-level cluster-metadata tuple produced during `ec_spire` index build
scales linearly with `nlists`, while the per-cluster aggregated tuple
scales roughly with `N / nlists`. Both can independently exceed the
8 kB page limit (`ec_spire object tuple payload N exceeds page size
8192`).

Observed (50k corpus, dim 1536, RaBitQ, default nlists=1):

    ERROR: ec_spire populated ambuild failed:
    ec_spire object tuple payload 11270 exceeds page size 8192

Observed (100k corpus, same shape, nlists=256 then 512):

    ec_spire object tuple payload 8758 exceeds page size 8192   (nlists=256)
    ec_spire object tuple payload 17462 exceeds page size 8192  (nlists=512)

`nlists=128` worked at both 50k and 100k. Empirically the top-level
tuple was ~34 bytes per centroid in this corpus, so the practical upper
bound today is `nlists ≲ 200` at dim 1536 — anything larger overflows
regardless of `N`. Conversely the per-cluster aggregate forces a
practical lower bound that grows with `N`. The reachable nlists window
narrows with scale, and there is no compile-time / runtime guard that
surfaces this *before* the build starts — the operator only learns by
attempting the build.

Recommendation: capture this as a Phase 13 design follow-up: either
(a) page-spill the top-level cluster-metadata tuple so its size is no
longer bounded by `MaxHeapTupleSize`, (b) shard / chunk the
per-cluster aggregate so a single overflowing partition does not abort
the build, or (c) provide a build-time validator (`ecaz dev spire
validate-build-plan ...`) that returns the predicted top-level and
per-cluster tuple sizes for a given `(N, dim, storage_format, nlists)`
so operators know the safe range up front. Until then, packet manifests
and runbooks should record the `(N, nlists)` combination that
successfully builds.

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
| 50k   | `/home/peter/pg18-spire/snapshots/50k/base.tar.gz` + `pg_wal.tar.gz` + `backup_manifest` | base.tar.gz `86ceb9dfc1b646471de26c21c64a07985a16a9008755d56303bfc4175810a4c3`; pg_wal.tar.gz `57cedcb50ee24e25279ac0a3ac8db9600695ad0d2a14e7281a00a23bf1d7ff47`; backup_manifest `4a53d63932d489c4231001c3a117ae793ff17962e7ac3654f8aa6a2832ad8a3f` (see `/home/peter/pg18-spire/snapshots/50k/sha256.txt`) | 692 MB | 2026-05-22T21:57 PDT |
| 100k  | `/home/peter/pg18-spire/snapshots/100k/{base,299261,299262,299263,299264,pg_wal}.tar.gz` + `backup_manifest` (4 tablespace tarballs included) | base `84eef1e9fec21ca7265081713cfc60c3f36ea0ed254a97e133aa802a952c956f`; 299261 (ts1) `a8e640b2169e1afc8b452e364e6992390276e9c3ab8bb0bcc1a22ff02e0b346a`; 299262 (ts2) `5c48edc6136b7a3e6e9bca28b613cc9a0b860fd2993479fa61369f3f5355eb7c`; 299263 (ts3) `c2424a9b422b9d25a51b41207d71cc833097ee1f77c0ef9d0453bb76694c1277`; 299264 (ts4) `d6cb366e1b7541dce13eeb019df9a1aa7099ba070325adc5bb90d4dba4b77b3d`; pg_wal `736cae9ec696deff82e9d58ace2bffec12b4706178509620bae3a4d1bfe83676`; backup_manifest `416c4251dc143721d160a89c4fef24fd0c52ed0cc6808e8b36ba3979fea34b1b` (see `/home/peter/pg18-spire/snapshots/100k/sha256.txt`) | 1.9 GB | 2026-05-22T22:08 PDT |
| 1M    | skipped — user direction: 1M too big for this machine; deferred to AWS lane | — | — | — |
