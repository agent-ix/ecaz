# Task 167 packet 057 artifact manifest — measured negative

- Preregistration head: `2f305031a50828f93a788311d5a789da9827a999`.
- Candidate code checkpoints:
  `5e32a1dfb2e5d35ffe365c8bb013f43cc3bdbb34` and
  `3da6df06cd8f2428212e492535987e993a4658cf`.
- Owning packet:
  `reviews/task-167/057-pruned-backlink-noop-50k-gate/`.
- Suite config: `task167-pruned-backlink-noop-50k-suite.json`.
- Suite config SHA-256:
  `a4dab32c4387468bcc0c7b3a52a4013a13548811d3c3b1301d63d1fce7c99e1c`.
- Timestamp: `2026-08-22`.
- Lane: production physical distributed DistANN on PG18, three owners,
  RabitQ neighbor storage, exact fp32 truth, no rerank variant.
- Fixture: isolated `ec_real_50k`, one index per table, external run directory
  `/home/peter/.ecaz/clusters/task167-pruned-backlink-noop-50k-20260822`.
- Search regime: beam width 4, candidate heap 32, hop rounds 100, top-k 10,
  200 heldout queries plus 48 inserted-neighborhood queries.
- Insert regime at the quality checkpoint: 160 candidate-default inserts with
  ID base `2000000`. The append-when-room control ID base `3000000` is excluded
  until after the quality gate passes.
- Hard gates remain fixed from packet 045: inserted-neighborhood deficit at
  most `0.015`, heldout deficit at most `0.007`.
- Before comparators:
  - packet 047 robust-prune-all heldout deficit `0.008611`;
  - packet 051 append-when-room heldout deficit `0.010611`;
  - packet 054 conservative-admission heldout deficit `0.009611`.
- Runtime output is packet-local under `artifacts/final-suite/`. Corpus data,
  truth caches, PGDATA, PostgreSQL operational logs, and polling output are not
  committed. The three generated node PostgreSQL logs were pruned, and the
  stopped 5.0 GB external fixture was removed after cited results were
  captured.
- Run timestamp: `2026-08-22T22:41:03-07:00`; finish timestamp:
  `2026-08-22T23:16:57-07:00`; duration `2,158,191 ms` (`35m58.191s`).
  Suite status: failed at the preregistered heldout quality gate.

## Preregistered command

- Audit:
  `/home/peter/.cargo-target/release/ecaz bench suite audit --config reviews/task-167/057-pruned-backlink-noop-50k-gate/artifacts/task167-pruned-backlink-noop-50k-suite.json --log-file reviews/task-167/057-pruned-backlink-noop-50k-gate/artifacts/suite-audit-preregister.log`.
- Audit result: passed, 1 step. Log SHA-256:
  `cdc6118948c73fd5ccc4fde9b50ad02afee5868378a7adeb9210edd7cbeb4c44`.
- Run, only after exact-runtime PG18 release installation, release CLI build,
  and repeated audit:
  `/home/peter/.cargo-target/release/ecaz bench suite run --config reviews/task-167/057-pruned-backlink-noop-50k-gate/artifacts/task167-pruned-backlink-noop-50k-suite.json --log-file reviews/task-167/057-pruned-backlink-noop-50k-gate/artifacts/suite-run.log`.
- Report after completion or an expected quality-gate failed-step exit:
  `/home/peter/.cargo-target/release/ecaz bench suite report --manifest reviews/task-167/057-pruned-backlink-noop-50k-gate/artifacts/final-suite/suite-manifest.json --results-output reviews/task-167/057-pruned-backlink-noop-50k-gate/artifacts/final-suite/results.jsonl --log-file reviews/task-167/057-pruned-backlink-noop-50k-gate/artifacts/suite-report.log`.

## Exact runtime

- Runtime head: `13da0c545fe2600d330640e2c476f63b420bffb4`; both the
  installed PG18 extension and release CLI were built from this clean head.
- PG18 extension install command:
  `cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features pg18 --no-default-features`.
- Install result: passed. The optimized library build took `5m11s`, followed
  by the required `pgrx_embed` SQL-generation build (`1m00s`).
  `install-extension.log` LF-normalized SHA-256:
  `7f94b661e66aafa057b6fada66b34076dbd13dec3b1f1c0839567883e060d8c4`.
- Installed `ecaz.so` SHA-256:
  `aa4d96190b10a474e29c24bfab4d73958f14a2863b28bf81a41d25e8b664db52`.
- CLI build command: `cargo build -p ecaz-cli --release --no-default-features`.
- Build result: passed in `10m47s`, with the pre-existing unrelated dead-code
  warning at `commands/corpus/load.rs:190`. `build-cli.log` LF-normalized
  SHA-256:
  `1cf82367959bf7066a2ce6835e066f8d457a11fa13d6103211f4a4bd24806ad6`.
- Release CLI SHA-256:
  `d2a3a583408a52a67d99b4978a033e2c82137caba076538675521618f8efd2cb`.
- Exact-runtime audit result: passed, 1 step. Log SHA-256:
  `cdc6118948c73fd5ccc4fde9b50ad02afee5868378a7adeb9210edd7cbeb4c44`.
- The suite ran with worktree head
  `22d85408e7290db8c39dc863d7354e2ee01efec2`; the only commits between the
  built runtime head and this runner head add packet-local build/audit
  provenance and do not change product or runner code.

## Decision rule

- If either fixed quality band fails, reject the candidate, skip the
  append-when-room control and post-gate drills, and keep Task 167 open.
- If both fixed quality bands pass, complete the packet's remaining drills and
  then preregister the isolated 10k/50k/100k recall, latency, and storage
  matrix. A 50k pass alone is not closeout.

## Result and disposition

- Pre-insert recall instrumentation reconciled exactly: ordinary and exact
  distinct recall were both `0.954500` (`absolute_delta=0.000000`).
- Insert throughput: physical `0.222` rows/s versus single-instance control
  `0.423` rows/s (`physical_over_control=0.523832`). This unjudged measurement
  is not used to rescue or reject the candidate.
- The coordinator backend attempted 4,418 backlink amendments and preserved
  702 full targets whose exact prune rejected the proposed backlink.
- Inserted-neighborhood exact recall: physical `0.922082`, fresh `0.931052`,
  deficit `0.008970`; fixed allowance `0.015000`; pass.
- Heldout exact recall: physical `0.847722`, fresh `0.857333`, deficit
  `0.009611`; fixed allowance `0.007000`; fail by `0.002611`.
- The append-when-room arm was skipped with `control_mutation_excluded=true`,
  and post-gate drills were skipped.
- Against the same fixed query/truth surface, the candidate equals packet
  054's rejected conservative-admission heldout deficit (`0.009611`) and is
  `0.001000` worse than packet 047's retained robust-prune result (`0.008611`).
  It is rejected; no final 10k/50k/100k closeout matrix is authorized.
- Key decision lines are copied into `cited-results.log`.

## Committed result artifacts

- `cited-results.log`: SHA-256
  `cb1d59af1e3f5ab4fe3404ce4b1ece35f2125b4399a244817801d905039c2638`.
- `suite-run.log`: SHA-256
  `f630c48e1d75e3f1508aa4d986901d29f0387af5cd90953006eaee99133fbf17`.
- `suite-report.log`: SHA-256
  `e31c82b27ffc755209da1825943b4e9cbbae97378ed9c6a77418bc42a9c85dec`.
- `final-suite/suite-manifest.json`: SHA-256
  `1b98d6f81ef7d6a3bf47e199c4479e2005f64cb7570a462bbb7f9db09d1a2fd9`.
- `final-suite/results.jsonl`: 88 structured rows; SHA-256
  `c9d8d20164cf2efbf050178345e533dc779beb5a5a0aaa40152c20786a03671e`.
- `distann-local-multinode.log`: SHA-256
  `a95a3b1cbcd7bff9e08c7f75f86a444662475ec5467e8e594583ba0b46934e39`.
- `distann-multinode-summary.log`: SHA-256
  `3cc4c641cfbe85d184873082f4e3aa97495741b09cc795acc40b939d75c98837`.
- `physical-production-recall.log`: SHA-256
  `c3d19c8c70751161e1bfbf68dbac82a0d9fa1da8349bd15bd930a395d87b91b8`.
- `physical-production-latency.log`: SHA-256
  `9e460597aaebcb825904982728d034e1e80b34d009c0eb718331036afa7ad916`.
- `physical-production-predictions.json`: SHA-256
  `1abc27ffe21b97f0513c35721e4e354fd2b379dbd7d7a7031fc009ac5f219e22`.
- `physical-head-membership.json`: SHA-256
  `c76c218365faec135ad1a4e00009d3f4058d1fb8644614efb81159f138ccad45`.
