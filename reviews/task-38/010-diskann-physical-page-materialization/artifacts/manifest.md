# Task 38 Packet 010 Artifact Manifest

- OOM activity-oracle checkpoint:
  `2100e73101fdab4fc4fa268cb7415ddaf4813f2f`
- DiskANN candidate checkpoint:
  `50b7690d84024ec58bc570bc506cfa6719c23c9e`
- Canonical unused-slot regression checkpoint:
  `73a3a7849b17c8a8488357dcd8f5fff76f8a377c`
- DiskANN baseline checkpoint:
  `147d44d05f0fe2d95a590a5730c6af55091d11c1`
- Task bucket: `reviews/task-38/`
- Packet: `010-diskann-physical-page-materialization`
- Host: local Apple M5, macOS, `arm64`
- PostgreSQL target: PG18 at Unix socket `/Users/peter/.pgrx`, port `28818`
- Benchmark timestamps:
  - baseline manifest: `2026-07-28T17:34:52-0700`
  - candidate manifest: `2026-07-28T17:56:27-0700`
- Remote/AWS/CI/Docker/Intel/Linux execution: none
- Storage format: `ecvector`, four-bit encoding, seed `42`
- Rerank mode: profile default; no separate rerank variant
- Isolation: separate fresh `task38_p010_baseline` and
  `task38_p010_candidate` databases; one index per corpus table; no shared-table
  A/B surface

## Focused Validation

### `oom-activity-oracle-unit.log`

- Command:
  `cargo test -p ecaz-cli
  oom_kill_activity_marker_distinguishes_workload_from_hold_query --color
  never`
- Exit: `0`
- SHA-256:
  `a94aef04e7f99964ff8424014c84e0181a41d4e7d9b77a96d75cd3d62f23b745`
- Key result: `1 passed; 0 failed; 472 filtered out`.
- Coverage: active marked workload accepted; hold query, idle state, missing
  activity, and mismatched query rejected.

### `diskann-page-materialization-unit.log`

- Command:
  `cargo test -p ecaz data_page_materialization_ --lib --color never`
- Exit: `0`
- SHA-256:
  `38c26101c05b3cc74ea292a654f0ee4b59dce5a160a2eafea26f01747ad79b35`
- Key result: `2 passed; 0 failed; 2523 filtered out`.
- Coverage: unused physical line-pointer offsets remain stable, and tuple
  insertion follows PostgreSQL `PageAddItem` accounting.

### `diskann-unused-line-pointer-pg18.log`

- Database preparation:
  `/opt/homebrew/Cellar/postgresql@18/18.3/bin/createdb -h
  /Users/peter/.pgrx -p 28818 task38_p010_pgtest`
- Command:
  `/Users/peter/.cargo/bin/ecaz dev sql --pg 18 --db task38_p010_pgtest
  --socket-dir /Users/peter/.pgrx --port 28818 --raw --sql
  "CREATE EXTENSION ecaz;
  SELECT tests.test_ec_diskann_unused_line_pointer_scan();" --log-output
  reviews/task-38/010-diskann-physical-page-materialization/artifacts/diskann-unused-line-pointer-pg18.log`
- Exit: `0`
- SHA-256:
  `589eb4886f1516641be5f87d073f7ba6951e514e5e476455cf5ab4be548411af`
- Key result: extension creation and the PG18 regression function both
  complete successfully.
- Coverage: real DiskANN index creation, an LP_UNUSED physical offset, a later
  occupied node, chain materialization, and forced nearest-neighbour scan.
- Status: superseded by the byte-faithful canonical-slot coverage in
  `finding11-canonical-unused-pg18.log`.

### `finding11-install.log`

- Source checkout: repository root.
- Source content: checkpoint
  `73a3a7849b17c8a8488357dcd8f5fff76f8a377c`; the install preceded the commit,
  with the two tracked source files containing the exact content subsequently
  committed at that checkpoint.
- Command:
  `/Users/peter/dev/tqvector/target/debug/ecaz --log-file
  /Users/peter/dev/tqvector/reviews/task-38/010-diskann-physical-page-materialization/artifacts/finding11-install.log
  dev install ecaz-pg-test --pg 18 --pgrx-home /Users/peter/.pgrx`
- Exit: `0`.
- Timestamp: `2026-07-28T23:35:27-0700`.
- Log SHA-256:
  `9ca049793587fb7f6d58247c0bd82d48e18ba0d2cc82f18d7efb4849ccc8ce71`.
- Installed release backend SHA-256:
  `37754507e17f11496e5a5e0123a0e8ac0a687b89aa00389ee2d64725d9025401`.

### `finding11-canonical-unused-pg18.log`

- Database: fresh `task38_p010_finding11`, dropped after validation.
- Command:
  `/Users/peter/.cargo/bin/ecaz dev sql --pg 18 --db
  task38_p010_finding11 --socket-dir /Users/peter/.pgrx --port 28818 --raw
  --sql "CREATE EXTENSION ecaz;
  SELECT tests.test_ec_diskann_unused_line_pointer_scan();
  SELECT tests.test_ec_diskann_persistent_unused_line_pointer_scan();"
  --log-output
  reviews/task-38/010-diskann-physical-page-materialization/artifacts/finding11-canonical-unused-pg18.log`
- Exit: `0`.
- Timestamp: `2026-07-28T23:35:46-0700`.
- SHA-256:
  `ad1341ed171b5fd6961ea959aae80ea5ebba0f2a12c3c1ba7fecd5b2ad502664`.
- Key result: extension creation and both focused PG18 regression functions
  complete successfully.
- Coverage:
  - canonical `(lp_off, lp_flags, lp_len) = (0, 0, 0)` plus
    `PD_HAS_FREE_LINES`;
  - normal `PageAddItemExtended` recycling and addressability at the recycled
    physical TID, followed by a forced DiskANN scan; and
  - a canonical persistent gap on a page too full for the next fixed-size
    node, with a later occupied tuple remaining byte-identical and addressable
    at its original physical TID.

## Suite Configuration

### `../diskann-ab-suite.json`

- SHA-256:
  `3d6f9e9aefc5e278b4e4c41a661771bd2831ff9d8afd58e1afbbfbc6da749996`
- Runner: `ecaz bench suite` only.
- Matrix: baseline/candidate × `ec_real_10k`/`ec_real_50k`/`ec_real_100k`
  × load/recall/latency/storage = 24 steps.
- Recall/latency configuration: 200 queries, `k=10`, list-size sweep
  `64,128,200,400,800`, forced index, latency concurrency 1, 200 iterations,
  post-recall warm cache state.
- Corpus data is not committed. The staged source SHA-256 values recorded by
  the load logs are:
  - 10k corpus `c67c5810b66d982d705974e48d4775479adfbd92a988f694091266e049a35e75`;
  - 10k queries `a2c191bb742017d849e73f6e6866e8e0f0bac1579ba212f7fc76b8eb09904ae8`;
  - 50k corpus `56023baaa7bc42f758272e8617603d538808e6290a8a70a3a84e057571240133`;
  - 50k queries `95ac7992578aa80bb193657f10fbcbf1ea3867e559739244bf5a467f7a5a9fa3`;
  - 100k corpus `07275cfd5a7a4b415ddf5eacc086de98294ac978532df46ffae30f9202323a95`;
  - 100k queries `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782`.

### `suite-dry-run.log`

- Command:
  `/Users/peter/.cargo/bin/ecaz --log-file
  reviews/task-38/010-diskann-physical-page-materialization/artifacts/suite-dry-run.log
  bench suite run --config
  reviews/task-38/010-diskann-physical-page-materialization/diskann-ab-suite.json
  --dry-run`
- Exit: `0`.
- Key result: all 24 configured commands expand through the canonical suite
  runner before execution.

## Release Install Provenance

### `baseline-install.log`

- Source checkout:
  `.task-worktrees/task38-p010-baseline`
- Source checkpoint:
  `147d44d05f0fe2d95a590a5730c6af55091d11c1`
- Dirty state at install: clean (`git status --porcelain` empty).
- Command:
  `/Users/peter/dev/tqvector/target/debug/ecaz --log-file
  /Users/peter/dev/tqvector/reviews/task-38/010-diskann-physical-page-materialization/artifacts/baseline-install.log
  dev install ecaz-pg-test --pg 18 --pgrx-home /Users/peter/.pgrx`
- Exit: `0`.
- Log SHA-256:
  `3c14f1a2f853630c9f948e3cfe0c25febcc9588a65a0dfe5dcc3e901657a0365`
- Installed release backend SHA-256:
  `cc6a69fd4f8fad5c0ee01b07c629cf7571c9398a79028ec0c62cc4a373917630`.

### `candidate-install.log`

- Source checkout: repository root.
- Source checkpoint:
  `50b7690d84024ec58bc570bc506cfa6719c23c9e`
- Dirty state at install: tracked source clean
  (`git status --porcelain --untracked-files=no` empty); untracked files were
  packet artifacts and unrelated user-owned review artifacts only.
- Command:
  `/Users/peter/dev/tqvector/target/debug/ecaz --log-file
  /Users/peter/dev/tqvector/reviews/task-38/010-diskann-physical-page-materialization/artifacts/candidate-install.log
  dev install ecaz-pg-test --pg 18 --pgrx-home /Users/peter/.pgrx`
- Exit: `0`.
- Log SHA-256:
  `ab2ff68d81d0360e7443ccc8ee0f4f5d8193baf3f6091948b876576e7cc57b91`
- Installed release backend SHA-256:
  `c12df437ae355a5c06aa3aefcebfd659b8a587bed27699566261aaa61a739556`.

Each install was followed by
`ecaz dev fault provider-restore --pg 18 --pgrx-home /Users/peter/.pgrx
--port 28818` before creating that lane's fresh database.

## Baseline Benchmark Artifacts

### `bench/baseline/suite-manifest.json`

- Command:
  `/Users/peter/.cargo/bin/ecaz --log-file
  reviews/task-38/010-diskann-physical-page-materialization/artifacts/baseline-suite-run.log
  bench suite run --config
  reviews/task-38/010-diskann-physical-page-materialization/diskann-ab-suite.json
  --only-tag baseline --artifact-dir
  reviews/task-38/010-diskann-physical-page-materialization/artifacts/bench/baseline
  --database task38_p010_baseline --host /Users/peter/.pgrx --port 28818`
- Exit: `0`.
- SHA-256:
  `cdce37e39fb1e24f1f3f8b2c59fdd46a02d7b517d003ea89c49ca0e8cdb68682`
- Backend: release.
- Key result: 12 selected baseline steps succeeded; 12 candidate-tagged steps
  were intentionally skipped.

### `bench/baseline/results.jsonl`

- Lines: `81`.
- SHA-256:
  `da762baeff5012b7bfa4d04b2b3d9163d239445e05222ba3adc93b27d3ba70f4`
- Key result rows:
  - list-size-200 recall: 10k `1.0000`, 50k `0.9905`, 100k `0.9845`;
  - list-size-200 mean latency: 10k `0.78 ms`, 50k `1.19 ms`,
    100k `1.59 ms`;
  - DiskANN index bytes: 10k `4,299,162`, 50k `21,600,666`,
    100k `43,096,474`;
  - index build seconds: 10k `7.19`, 50k `96.13`, 100k `235.95`.

### Baseline per-step logs

`bench/baseline/` contains one packet-local log for every selected step:

- `baseline-load-{10k,50k,100k}-diskann.log`;
- `baseline-recall-{10k,50k,100k}-diskann.log`;
- `baseline-latency-{10k,50k,100k}-diskann.log`; and
- `baseline-storage-{10k,50k,100k}-diskann.log`.

Each log records the lane, corpus scale, `ec_diskann` profile, command-specific
configuration, and raw result table used to construct `results.jsonl`.

## Candidate Benchmark Artifacts

### `bench/candidate/suite-manifest.json`

- Command:
  `/Users/peter/.cargo/bin/ecaz --log-file
  reviews/task-38/010-diskann-physical-page-materialization/artifacts/candidate-suite-run.log
  bench suite run --config
  reviews/task-38/010-diskann-physical-page-materialization/diskann-ab-suite.json
  --only-tag candidate --artifact-dir
  reviews/task-38/010-diskann-physical-page-materialization/artifacts/bench/candidate
  --database task38_p010_candidate --host /Users/peter/.pgrx --port 28818`
- Exit: `0`.
- SHA-256:
  `8869c74969eddbe952300132ea96d6685e861cb391aea0b8be8b480c8f5e5b69`
- Backend: release.
- Key result: 12 selected candidate steps succeeded; 12 baseline-tagged steps
  were intentionally skipped.

### `bench/candidate/results.jsonl`

- Lines: `81`.
- SHA-256:
  `f18650d2d62727fadfcd6d058237f82b6ba16dc9682fcd868b60cb68426871a8`
- Key result rows:
  - list-size-200 recall: 10k `1.0000`, 50k `0.9905`, 100k `0.9845`;
  - list-size-200 mean latency: 10k `0.74 ms`, 50k `1.25 ms`,
    100k `1.45 ms`;
  - DiskANN index bytes: 10k `4,299,162`, 50k `21,600,666`,
    100k `43,096,474`;
  - index build seconds: 10k `7.15`, 50k `98.60`, 100k `232.11`.

### Candidate per-step logs

`bench/candidate/` contains one packet-local log for every selected step:

- `candidate-load-{10k,50k,100k}-diskann.log`;
- `candidate-recall-{10k,50k,100k}-diskann.log`;
- `candidate-latency-{10k,50k,100k}-diskann.log`; and
- `candidate-storage-{10k,50k,100k}-diskann.log`.

Each log records the lane, corpus scale, `ec_diskann` profile, command-specific
configuration, and raw result table used to construct `results.jsonl`.

## A/B Findings

- All 15 pairs of `recall@k`, `recall_worst`, and `ndcg@k` values are
  identical between baseline and candidate.
- Freshly built benchmark indexes contain no unused line pointers, so the
  bit-identical recall is no-regression evidence for the unchanged path; the
  focused PG18 tests above exercise the changed LP_UNUSED branch.
- DiskANN `size_bytes` and `per_row_bytes` are identical at 10k, 50k, and
  100k.
- Candidate mean-latency deltas over the complete 15-point sweep range from
  `-17.4%` to `+7.2%`.
- At list size 200, candidate deltas are `-5.1%` at 10k, `+5.0%` at 50k,
  and `-8.8%` at 100k.
- At 50k, candidate mean latency is slower at every list size: `+1.4%` at 64,
  `+1.1%` at 128, `+5.0%` at 200, `+2.2%` at 400, and `+7.2%` at 800.
- Latency is mixed and inconclusive from this single sequential A/B. The
  packet does not attribute the differences to variance or claim latency
  neutrality.
- The correctness hardening is proven recall-neutral and storage-neutral; its
  acceptance retains the observed 50k latency slowdown explicitly.

## Evidence Boundary

This packet proves the OOM activity-oracle logic, the unused-line-pointer
DiskANN regression on local PG18, and release-backend DiskANN A/B behavior at
10k/50k/100k on Apple M5. It does not prove any Linux LD_PRELOAD, remote socket,
cgroup-v2, or Intel behavior. Those 67 Task 38 gates remain open for the
designated Intel/Linux host.
