# Artifact manifest

- Head SHA: `1b3de943c75331cdc4ebc6c676d96fe878a78401`
- Task bucket: `reviews/task-199`
- Packet: `reviews/task-199/002-operations-lifecycle-and-isolation`
- Timestamp: `2026-07-24T21:39:55-07:00`
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
  `d233cf4c103c57a120c34e176079769d50000894028b6932c36a779f17aed89f`
- Run interval: manifest timestamps `1784954007482` through `1784954335622`
  Unix milliseconds; duration `328138 ms`

The extension, CLI, and all three PostgreSQL nodes reported the same normal
release SHA above. Corpus/query TSV data is intentionally not committed.

## Commands and artifacts

| Artifact | SHA-256 | Command | Key result |
|---|---|---|---|
| `task199-normal-lifecycle-10k.json` | `d233cf4c103c57a120c34e176079769d50000894028b6932c36a779f17aed89f` | checked-in `SuiteConfig` | one normal-release PG18 `distann-local-multinode` step using `ec_real_10k` |
| `normal-release-install-r14.log` | `7a3fd0915283ec22c065d8b645779066b164c5fe3d89c5055adc1332747e662c` | `PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features --features pg18` | normal release extension installed; 311 SQL entities including two triggers |
| `ecaz-cli-release-build-r14.log` | `a9978dc9817bf97845a2dbe0185ae2ce3e84ccf193ee65224f432ac946fa6b0c` | `PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo build --release -p ecaz-cli` | release CLI finished; one pre-existing corpus-loader dead-code warning |
| `suite-run-r14.log` | `c515038e81086908f6f99bb9c9024ecde744aab7e506c01b57abafcbc31c761b` | `target/release/ecaz bench suite run --config reviews/task-199/002-operations-lifecycle-and-isolation/artifacts/task199-normal-lifecycle-10k.json --log-file reviews/task-199/002-operations-lifecycle-and-isolation/artifacts/suite-run-r14.log` | step succeeded; manifest and structured results written |
| `run/suite-manifest.json` | `7fa9b404bb4160658732e4e6edd7f27b2f503d6858c066f1ee9d786e018184f6` | emitted by the suite command | `status=succeeded`, exit 0, duration 328138 ms, runner SHA exact |
| `run/results.jsonl` | `3f5caf386eb30007b2c001866c00feee6ebc457f853346f845b11128c33cedf0` | emitted by the suite command | normalized topology, lifecycle, recall, latency, storage, and semantic rows |
| `run/normal-replica-lifecycle-10k/distann-multinode-summary.log` | `03b991ae465d55dd454e324f80fa529b823799426efbad61a1ac8bfee942652d` | suite step summary | every cited Task 199 scenario has `pass=true`; exact-SHA provenance |
| `run/normal-replica-lifecycle-10k/distann-local-multinode.log` | `24313edc206df6ac76564a577202fefd141eb3e6d3eb50144935f46d4f323207` | suite step log | release preflight, 10,000-row topology, lifecycle drills, semantic checks, and topology gate passed |
| `suite-audit-r14.log` | `474c702b085e7e0b191462c3c47f912251d560978e49a7700d81d62a7d2d2e5b` | `target/release/ecaz bench suite audit --config reviews/task-199/002-operations-lifecycle-and-isolation/artifacts/task199-normal-lifecycle-10k.json --log-file reviews/task-199/002-operations-lifecycle-and-isolation/artifacts/suite-audit-r14.log` | audit passed, one step |
| `suite-status-r14.log` | `f2141ffdffba6a85ab79b76cdca93058959563611b80668c477b7561d587efff` | `target/release/ecaz bench suite status --manifest reviews/task-199/002-operations-lifecycle-and-isolation/artifacts/run/suite-manifest.json --log-file reviews/task-199/002-operations-lifecycle-and-isolation/artifacts/suite-status-r14.log` | completed 1, failed 0, missing 0, stale 0 |
| `suite-report-r14.md` | `efc461a48fd55285a20b0fd39537685f19dd55594b9dc5baab622f8984901b59` | `target/release/ecaz bench suite report --manifest reviews/task-199/002-operations-lifecycle-and-isolation/artifacts/run/suite-manifest.json --log-file reviews/task-199/002-operations-lifecycle-and-isolation/artifacts/suite-report-r14.md` | compact report; parsed lifecycle outcomes all pass |

## Raw recall and latency logs

These are diagnostic 10-query / two-timed-sample measurements for the
operations packet, not the Task 199 release gate.

| Artifact | SHA-256 | Key result |
|---|---|---|
| `run/normal-replica-lifecycle-10k/physical-owner-control-recall.log` | `d789ef6c6ade0f0b9fdd7fed37b400c44466700224a5413545be4f390e209979` | 10 queries / 100 trials; recall `0.9900` |
| `run/normal-replica-lifecycle-10k/physical-owner-control-latency.log` | `1a7259ac8e2ac88ac267d65332df7cb1673118f69035e576d278d7f61447319d` | warm mean `19.70 ms`, two timed samples |
| `run/normal-replica-lifecycle-10k/physical-coordinator-replica-recall.log` | `52fc8b7b9f5820970156e7fac0b5a2da3a0dcdcfbac3ca7a52f2cce670e457bb` | 10 queries / 100 trials; recall `0.9900` |
| `run/normal-replica-lifecycle-10k/physical-coordinator-replica-latency.log` | `4b53dc014befd9281be6a3b3988f20e528058df34e2b2c44e20fccaf8dfcb169` | warm mean `15.30 ms`, two timed samples |
| `run/normal-replica-lifecycle-10k/single-single-recall.log` | `854448acfdc44112f10cef31c243587ae18f950e639b51377af3452b74070c40` | 10 queries / 100 trials; recall `1.0000` |
| `run/normal-replica-lifecycle-10k/single-single-latency.log` | `b39a97dbef6c88d3fbebc0c76507a682ed5c376795946a5134853ed5b68ae5c3` | warm mean `2.64 ms`, two timed samples |

## Cited lifecycle results

- Three-node release preflight was unanimous at
  `1b3de943c75331cdc4ebc6c676d96fe878a78401`.
- Ready/Published topology contained exactly 10,000 owned records, zero
  non-owned records, and zero orphans; two remote owners materialized the
  expected rows.
- A Ready replica copied 10,000 records / 131,520,000 bytes into 158,326,784
  relation bytes; the representative build took 4,718 ms and emitted
  137,460,656 WAL bytes.
- Owner and replica arms both returned `0.9900` recall. The two-sample warm
  latency diagnostic was 19.70 ms owner versus 15.30 ms replica.
- REPEATABLE READ and SERIALIZABLE read-only scans returned owner-ordered
  identity without demoting Ready. Writes at both levels returned SQLSTATE
  `25001` / `EC_TRANSACTION_ISOLATION`, inserted zero rows, and left Ready.
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
  still returned owner identity and left Ready; operator recovery changed it
  to Stale after authentication was restored.
- A locked replica relation caused nonblocking full owner fallback, exact
  identity, durable Stale, and `EC_REPLICA_RELATION_RACE`.
- With a queued control-index `AccessExclusiveLock`, invalidation completed in
  155 ms with `40001`; the canceled DROP left the index present and the image
  Stale.
- Successor epoch 2 automatically moved the old image to Retiring with
  `epoch_superseded`, preserved owner identity, reclaimed both old relations
  with zero residue, built the successor image, and survived a fresh
  coordinator backend reconnect with ordered identity and Ready state.
- Owner restart transport recovery, owner-outage partial-build cleanup,
  corrupt-image diagnosis/fallback, stronger-isolation owner fallback,
  explicit retire/reclaim, removed-image fallback, normal-build feature
  isolation, and seven materialization semantic cases also passed.
