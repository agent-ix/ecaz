# Artifact manifest

- Head SHA: `c9e64a9de383132abd1af2c55c0b15ecfe215f34`
- Task bucket: `reviews/task-199`
- Packet: `reviews/task-199/002-operations-lifecycle-and-isolation`
- Timestamp: `2026-07-24T19:52:19-07:00`
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
- Run interval: manifest timestamps `1784947517974` through `1784947817098`
  Unix milliseconds; duration `299123 ms`

The extension, CLI, and all three PostgreSQL nodes reported the same normal
release SHA above. Corpus/query TSV data is intentionally not committed.

## Commands and artifacts

| Artifact | SHA-256 | Command | Key result |
|---|---|---|---|
| `task199-normal-lifecycle-10k.json` | `d233cf4c103c57a120c34e176079769d50000894028b6932c36a779f17aed89f` | checked-in `SuiteConfig` | one normal-release PG18 `distann-local-multinode` step using `ec_real_10k` |
| `normal-release-install-r10.log` | `2996c2e3407ae89e55c228793ae53a8e2b5e27963c2881369c8ca2b9751c0741` | `PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features --features pg18` | normal release extension installed; 310 SQL entities |
| `ecaz-cli-release-build-r10.log` | `8b7893f9e780d9951b56cbe9ef242e4e764313b82c62b39d217fa4e1cba1929d` | `PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo build --release -p ecaz-cli` | release CLI finished; one pre-existing corpus-loader dead-code warning |
| `suite-run-r10.log` | `c515038e81086908f6f99bb9c9024ecde744aab7e506c01b57abafcbc31c761b` | `target/release/ecaz bench suite run --config reviews/task-199/002-operations-lifecycle-and-isolation/artifacts/task199-normal-lifecycle-10k.json --log-file reviews/task-199/002-operations-lifecycle-and-isolation/artifacts/suite-run-r10.log` | step succeeded; manifest and structured results written |
| `run/suite-manifest.json` | `807dbca364173f807b21ebe24c1af9ea48aba845be6becdf91304ac17a5c0dfb` | emitted by the suite command | `status=succeeded`, exit 0, duration 299123 ms, runner SHA exact |
| `run/results.jsonl` | `a7985a94145684f26dc125904f113e51822533df89e08024a941cc81f5a310ac` | emitted by the suite command | normalized topology, lifecycle, recall, latency, storage, and semantic rows |
| `run/normal-replica-lifecycle-10k/distann-multinode-summary.log` | `95abee35a1bc4e4ef357e072b40fbda8ad383205ea572c64cc7f496005b96817` | suite step summary | every cited Task 199 scenario has `pass=true`; exact-SHA provenance |
| `run/normal-replica-lifecycle-10k/distann-local-multinode.log` | `978b3448fe7a79aed818e3b6c8f368033340ab6ccbb1dfe232ad0cf981ffe085` | suite step log | release preflight, 10,000-row topology, lifecycle drills, semantic checks, and topology gate passed |
| `suite-audit-r10.log` | `474c702b085e7e0b191462c3c47f912251d560978e49a7700d81d62a7d2d2e5b` | `target/release/ecaz bench suite audit --config reviews/task-199/002-operations-lifecycle-and-isolation/artifacts/task199-normal-lifecycle-10k.json --log-file reviews/task-199/002-operations-lifecycle-and-isolation/artifacts/suite-audit-r10.log` | audit passed, one step |
| `suite-status-r10.log` | `f2141ffdffba6a85ab79b76cdca93058959563611b80668c477b7561d587efff` | `target/release/ecaz bench suite status --manifest reviews/task-199/002-operations-lifecycle-and-isolation/artifacts/run/suite-manifest.json --log-file reviews/task-199/002-operations-lifecycle-and-isolation/artifacts/suite-status-r10.log` | completed 1, failed 0, missing 0, stale 0 |
| `suite-report-r10.md` | `ee12c8207b0899292ff6f7a7c4c2d9dfe8ff21d79b8f83675e1cd0767045def0` | `target/release/ecaz bench suite report --manifest reviews/task-199/002-operations-lifecycle-and-isolation/artifacts/run/suite-manifest.json --log-file reviews/task-199/002-operations-lifecycle-and-isolation/artifacts/suite-report-r10.md` | compact report; parsed lifecycle outcomes all pass |

## Raw recall and latency logs

These are diagnostic 10-query / two-timed-sample measurements for the
operations packet, not the Task 199 release gate.

| Artifact | SHA-256 | Key result |
|---|---|---|
| `run/normal-replica-lifecycle-10k/physical-owner-control-recall.log` | `8703ed9c047faf6dd4f7a2b0e3771a0d07fa5e17dff761d22405b6f98dbdf2f2` | 10 queries / 100 trials; recall `0.9900` |
| `run/normal-replica-lifecycle-10k/physical-owner-control-latency.log` | `dfb39821a4a4582be3700fe3dcfda223f0bdcf5fe661cf9b383370b1f20483f4` | warm mean `19.00 ms`, two timed samples |
| `run/normal-replica-lifecycle-10k/physical-coordinator-replica-recall.log` | `870e2f94ccc20041a6a9c9b94699499d2a6adec299ee883fe00c4326b404f969` | 10 queries / 100 trials; recall `0.9900` |
| `run/normal-replica-lifecycle-10k/physical-coordinator-replica-latency.log` | `d72f618e0e127b46da6c443485f1001baba1ee3b00719e9651ee92a3d2356b7f` | warm mean `14.80 ms`, two timed samples |
| `run/normal-replica-lifecycle-10k/single-single-recall.log` | `458f0098ce38f49b466fa5eab1070109197c8d84c106bd85b0488665f2bc2547` | 10 queries / 100 trials; recall `1.0000` |
| `run/normal-replica-lifecycle-10k/single-single-latency.log` | `ee7f94ff31a6e6702c856829f0e76131258bf3842149eb78a9474e23b4a078c8` | warm mean `2.59 ms`, two timed samples |

## Cited lifecycle results

- Three-node release preflight was unanimous at
  `c9e64a9de383132abd1af2c55c0b15ecfe215f34`.
- Ready/Published topology contained exactly 10,000 owned records, zero
  non-owned records, and zero orphans; two remote owners materialized the
  expected rows.
- A Ready replica copied 10,000 records / 131,520,000 bytes into 158,326,784
  relation bytes; the representative build took 4,963 ms and emitted
  137,659,336 WAL bytes.
- Owner and replica arms both returned `0.9900` recall. The two-sample warm
  latency diagnostic was 19.00 ms owner versus 14.80 ms replica.
- Actual INSERT produced one `40001 EC_REPLICA_INVALIDATED`, durably left
  `Stale`, then reached the existing Task 167 fail-closed
  `EC_GENERATION_MISSING` retry posture with zero inserted rows.
- Concurrent build/mutation observed `ShareRowExclusiveLock`; the real INSERT
  was fenced with `55P03`, inserted zero rows, and the image became Ready.
- An in-flight 40-row cursor retained one active pin, completed with immutable
  ordered identity, while the real INSERT returned one `40001`, inserted zero
  rows, and left the image Stale.
- Broken control authentication failed closed without mutation; operator
  recovery changed Ready to Stale after authentication was restored.
- A locked replica relation caused nonblocking full owner fallback, exact
  identity, durable Stale, and `EC_REPLICA_RELATION_RACE`.
- With a queued control-index `AccessExclusiveLock`, invalidation completed in
  44 ms with `40001`; the canceled DROP left the index present and the image
  Stale.
- Successor epoch 2 automatically moved the old image to Retiring with
  `epoch_superseded`, preserved owner identity, reclaimed both old relations
  with zero residue, built the successor image, and survived a fresh
  coordinator backend reconnect with ordered identity and Ready state.
- Owner restart transport recovery, owner-outage partial-build cleanup,
  corrupt-image diagnosis/fallback, repeatable-read rejection, explicit
  retire/reclaim, removed-image fallback, normal-build feature isolation, and
  seven materialization semantic cases also passed.
