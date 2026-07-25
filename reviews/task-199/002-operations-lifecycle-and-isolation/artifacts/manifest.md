# Artifact manifest

- Head SHA: `7c27a9916b810b86a5bfc296a41d8d5e0a8d18c2`
- Task bucket: `reviews/task-199`
- Packet: `reviews/task-199/002-operations-lifecycle-and-isolation`
- Timestamp: `2026-07-25T10:35:00-07:00`
- Lane: local WSL, managed PG18 `18.3`, three isolated local nodes
- Fixture: staged `ec_real_10k`; query SHA-256
  `a2c191bb742017d849e73f6e6866e8e0f0bac1579ba212f7fc76b8eb09904ae8`
- Storage / rerank mode: physical `ec_distann`, degree 32, trained exact
  cap-4,096 head, ordered 32 persisted-head seeds, BW4/H100, RaBitQ
  neighbor scoring, exact final scoring, lazy10 materialization
- Variant boundary: owner-control with no Ready replica versus the normal
  automatically selected coordinator Ready replica
- Isolation surface: one control index per physical corpus table; no
  multi-index shared-table surface
- Runner: checked-in `ecaz bench suite`; suite config SHA-256
  `bf3510fbdeacb98edcfa9db306cb8b9848d6d774d07c52f27505058d35a003e6`
- Run interval: manifest timestamps `1785000520089` through `1785000871990`
  Unix milliseconds; duration `357885 ms`

The extension, CLI, and all three PostgreSQL nodes reported the same normal
release SHA above. Corpus/query TSV data is intentionally not committed.

## Commands and artifacts

| Artifact | SHA-256 | Command | Key result |
|---|---|---|---|
| `task199-normal-lifecycle-10k.json` | `bf3510fbdeacb98edcfa9db306cb8b9848d6d774d07c52f27505058d35a003e6` | checked-in `SuiteConfig` | one normal-release PG18 `distann-local-multinode` step using `ec_real_10k`, including the armed ENOSPC drill |
| `normal-release-install-r19.log` | `60197737ffd217a88ea38d1c4db9c36cb5de28d37a26df6796f4b9f72cb35152` | `PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features --features pg18` | normal release extension installed; 311 SQL entities including two triggers |
| `ecaz-cli-release-build-r19.log` | `5b5e5f74f4a215176e7670ff350d001367b1611bacc2344b980461b53cc62cdc` | `PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo build --release -p ecaz-cli` | release CLI finished; one pre-existing corpus-loader dead-code warning |
| `suite-run-r19.log` | `915bca198ecd9a3ea7127c3f4ae0c35d95139168de1ef9c81bc41447dc82f907` | `target/release/ecaz bench suite run --config reviews/task-199/002-operations-lifecycle-and-isolation/artifacts/task199-normal-lifecycle-10k.json --log-file reviews/task-199/002-operations-lifecycle-and-isolation/artifacts/suite-run-r19.log` | step succeeded; manifest and structured results written |
| `run/suite-manifest.json` | `75222ade7f6fd310b882b5eb5573e47c77242c52aa8c0208af04906005b09960` | emitted by the suite command | `status=succeeded`, exit 0, duration 357885 ms, runner SHA exact |
| `run/results.jsonl` | `8b8b4b99523c084fa01b93925c17d97194ecbd572c5cbbdc74e2daafb5d4088a` | emitted by the suite command | normalized topology, lifecycle, recall, latency, storage, and semantic rows |
| `run/normal-replica-lifecycle-10k/distann-multinode-summary.log` | `7c07513e67a533a606b90500641eef9f6e3d9d73bb95f8f86004c468f8bc262d` | suite step summary | every cited Task 199 scenario has `pass=true`; exact-SHA provenance |
| `run/normal-replica-lifecycle-10k/distann-local-multinode.log` | `6a13a9961336aec5baa06daaf9e5276495ace97702e556f42c249367e24427b9` | suite step log | release preflight, 10,000-row topology, lifecycle drills, semantic checks, and topology gate passed |
| `suite-audit-r19.log` | `474c702b085e7e0b191462c3c47f912251d560978e49a7700d81d62a7d2d2e5b` | `target/release/ecaz bench suite audit --config reviews/task-199/002-operations-lifecycle-and-isolation/artifacts/task199-normal-lifecycle-10k.json --log-file reviews/task-199/002-operations-lifecycle-and-isolation/artifacts/suite-audit-r19.log` | audit passed, one step |
| `suite-status-r19.log` | `87e8175baa38364b7919137794045ef21cf2c1c4668041d6bbc25445743ca5c7` | `target/release/ecaz bench suite status --manifest reviews/task-199/002-operations-lifecycle-and-isolation/artifacts/run/suite-manifest.json --log-file reviews/task-199/002-operations-lifecycle-and-isolation/artifacts/suite-status-r19.log` | completed 1, failed 0, missing 0, stale 0 |
| `suite-report-r19.md` | `88725be703094136f749235653b93574e3cc775d860f36d7f17faae50fa37e2c` | `target/release/ecaz bench suite report --manifest reviews/task-199/002-operations-lifecycle-and-isolation/artifacts/run/suite-manifest.json --log-file reviews/task-199/002-operations-lifecycle-and-isolation/artifacts/suite-report-r19.md` | compact report; parsed lifecycle outcomes all pass |

## Raw recall and latency logs

These are diagnostic 10-query / two-timed-sample measurements for the
operations packet, not the Task 199 release gate.

| Artifact | SHA-256 | Key result |
|---|---|---|
| `run/normal-replica-lifecycle-10k/physical-owner-control-recall.log` | `5004643657d4d74323bf3137cf80d272dc50743d49a39ceca4d7f479ea988c19` | 10 queries / 100 trials; recall `0.9900` |
| `run/normal-replica-lifecycle-10k/physical-owner-control-latency.log` | `99edc840060187d6375e97815d536c690e3cda3e94168139d85d57eb85032146` | warm mean `19.50 ms`, two timed samples |
| `run/normal-replica-lifecycle-10k/physical-coordinator-replica-recall.log` | `745b5a6de93946585e9f4bd8d17debc0e14e0b5caf85860541561b2cfd3efe6c` | 10 queries / 100 trials; recall `0.9900` |
| `run/normal-replica-lifecycle-10k/physical-coordinator-replica-latency.log` | `86add4cc15f1b080bed7c3fd407a60fe7f9d05b0b62fe51f0dc162fd52e62a3e` | warm mean `16.10 ms`, two timed samples |
| `run/normal-replica-lifecycle-10k/single-single-recall.log` | `bddfafba1cc17a7b7686089eb07b2ae6b49aba665fd209ca501954f6aee9da99` | 10 queries / 100 trials; recall `1.0000` |
| `run/normal-replica-lifecycle-10k/single-single-latency.log` | `0451aa1ad1e64880682584b4448bc085f85828ff300472ec75b6856001456c82` | warm mean `2.73 ms`, two timed samples |

## Cited lifecycle results

- Three-node release preflight was unanimous at
  `7c27a9916b810b86a5bfc296a41d8d5e0a8d18c2`.
- Ready/Published topology contained exactly 10,000 owned records, zero
  non-owned records, and zero orphans; two remote owners materialized the
  expected rows.
- A Ready replica copied 10,000 records / 131,520,000 bytes into 158,326,784
  relation bytes; the representative build took 5,067 ms and emitted
  137,659,336 WAL bytes.
- Owner and replica arms both returned `0.9900` recall. The two-sample warm
  latency diagnostic was 19.50 ms owner versus 16.10 ms replica.
- READ UNCOMMITTED selected normally. REPEATABLE READ and SERIALIZABLE
  read-only scans returned owner-ordered identity without demoting Ready;
  writes at both levels reached the ordinary mutation guard, returned
  `40001 EC_REPLICA_INVALIDATED`, inserted zero rows, and rebuilt between
  cases.
- The post-control-commit crash terminated the mutating backend, durably left
  Stale, inserted zero rows, and returned owner-ordered identity from a fresh
  backend.
- Armed tablespace relation creation returned SQLSTATE `53100` with one
  provider `errno=28` event, zero eligible partial images, zero catalog and
  hidden-relation residue, a healthy cluster, owner-fallback identity, and a
  successful Ready recovery build.
- Actual INSERT produced one `40001 EC_REPLICA_INVALIDATED`, durably left
  `Stale`, then reached the existing Task 167 fail-closed
  `EC_GENERATION_MISSING` retry posture with zero inserted rows; its Stale
  rebuild error named the retire/reclaim recovery sequence.
- Actual DELETE returned one `40001`, deleted zero rows, and left Stale. The
  participant tombstone endpoint likewise returned one `40001`, preserved the
  physical graph digest, tombstoned zero rows, and left Stale.
- VACUUM processed a real pre-build dead tuple, durably changed Ready to Stale
  with reason `vacuum`, and completed without an ERROR.
- A blocking real INSERT first held an ungranted `RowExclusiveLock` behind the
  build's `ShareRowExclusiveLock`; after Ready committed it returned `40001`,
  inserted zero rows, left Stale, and the image rebuilt with the same digest.
- An in-flight 40-row cursor retained one active pin, completed with immutable
  ordered identity, while the real INSERT returned one `40001`, inserted zero
  rows, and left the image Stale.
- Broken control authentication failed closed without mutation. With both side
  demotion and a read-only local catalog write unavailable, a locked replica
  still returned owner identity and left Ready; the same backend suppressed a
  repeated control attempt for that exact build, and operator recovery changed
  it to Stale after authentication was restored.
- A locked replica relation caused nonblocking full owner fallback, exact
  identity, durable Stale, and `EC_REPLICA_RELATION_RACE`.
- With a queued control-index `AccessExclusiveLock`, invalidation completed in
  45 ms with `40001`; the canceled DROP left the index present and the image
  Stale.
- Successor epoch 2 automatically moved the old image to Retiring with
  `epoch_superseded`, preserved owner identity, reclaimed both old relations
  with zero residue, built the successor image, and survived a fresh
  coordinator backend reconnect with ordered identity and Ready state.
- Owner restart transport recovery, owner-outage partial-build cleanup,
  corrupt-image diagnosis/fallback, stronger-isolation owner fallback,
  explicit retire/reclaim, removed-image fallback, normal-build feature
  isolation, and seven materialization semantic cases also passed.
