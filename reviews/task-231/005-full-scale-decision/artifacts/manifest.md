# Task 231 Packet 005 preregistration artifact manifest

- Head SHA: `1f88b553e140628e5d70f72632599f15705b1f25`.
- Task bucket and packet: `reviews/task-231/005-full-scale-decision/`.
- Lane: local Intel development host, PostgreSQL 18 target.
- Fixture/storage formats: isolated fresh current-heap control versus
  fixed-stride candidate; one index/table surface per suite step.
- Rerank/search mode: production RaBitQ neighbor scoring with exact-vector
  materialization, BW4/H100 primary and BW16/H25 transfer pair.
- Measurement state: frozen host precheck passed; no A/B arm has run yet.
- Suite config:
  `crates/ecaz-cli/suites/task231-fixed-stride-10k-50k-100k.json`, SHA-256
  `48dbcbf38383d99418e99b6f246149c5fb7b552b696444ed6cd8e9379da1d211`.

## `preregistration-audit.log`

- Timestamp: `2026-08-30T00:26:58-07:00`.
- Head SHA: `bf4b78ed2ad462e6c15816fa6544dfd46ee7414c`.
- Command: `/home/peter/.cargo-target/debug/ecaz bench suite audit --config
  crates/ecaz-cli/suites/task231-fixed-stride-10k-50k-100k.json --log-file
  reviews/task-231/005-full-scale-decision/artifacts/preregistration-audit.log`.
- SHA-256: `1297d037ee0c2d1c86607585e52c35cabad8ee9c8f63163abac022136568c989`.
- Key result: `audit passed: 27 steps`; exit code 0.

## `preregistration-cli-tests.log`

- Timestamp: `2026-08-30T00:25:16-07:00`.
- Head SHA: `9bb9e0f1b15996389b41e3b872bf1ba68ebd97f2`.
- Command: `cargo test -p ecaz-cli task231` (captured through
  `script -q -e -c`).
- SHA-256: `eb0373b61cf1a86b5fb8b04fd34c5666c58fbcedb57ac1e74efbd29b50e2f1d3`.
- Key result: `2 passed; 0 failed`; exit code 0. The tests cover fixed-stride
  suite expansion/cold-profile validation and structured parsing of the
  checksum plus DML raw-store-growth metrics.

## `preregistration-cold-residency-tests.log`

- Timestamp: `2026-08-30T01:11:09-07:00`.
- Head SHA: `f432f0575b23471d792789df57e723e702c8cf25`.
- Command: `cargo test -p ecaz-cli task231` (captured through
  `script -q -e -c`).
- SHA-256: `84c32921fa59c86187887412120050d6b7d6ffb1856c76fcdf186729e159fb7b`.
- Key result: `2 passed; 0 failed`; exit code 0. The result parser test now
  also proves that `physical_benchmark_residency_control` persists measured
  `resident_buffers_after` rather than falling through as a generic drill.
- Suite audit at the same source checkpoint remains `audit passed: 27 steps`.

## Decision-run build and host precheck

- Timestamp: `2026-08-30T01:50:48-07:00`.
- Head SHA: `1f88b553e140628e5d70f72632599f15705b1f25`.
- The custom PG18 toolchain's matching source-tree `contrib/pg_buffercache`
  was installed into `/home/peter/.ecaz/toolchains/pg18-ssl` before the run;
  the controlled-cold steps remain responsible for creating and measuring the
  extension independently on every fixture node.
- `cargo-pgrx-install-release.log` records the release extension installation
  into the custom multinode PG18 toolchain. Command: `cargo pgrx install
  --release --pg-config /home/peter/.ecaz/toolchains/pg18-ssl/bin/pg_config
  --no-default-features --features 'pg18 distann-head-attribution-benchmark'`.
  SHA-256: `03940300acfeb460bbdf27d8299fee6bf227042eb25af3530a83d74b1f9cd413`;
  exit code 0.
- `cargo-pgrx-install-release-precheck-host.log` records the same release
  extension installation into the pgrx PG18 scratch prefix used only by the
  suite host precheck. SHA-256:
  `6a17de7f8baa2626b9e0dd9fd59461f62ed13cd479992b59603fbbb032753712`;
  exit code 0.
- `cargo-build-cli-debug-runner.log` records the CLI runner build. Command:
  `cargo build -p ecaz-cli`. SHA-256:
  `1d1771e09081166bc5a3dd380154c3fce4b57751b26c40896c1ee486769f950f`;
  exit code 0. The single dead-code warning is pre-existing and does not
  affect the release extension backend used by measured fixtures.
- `run/precheck-runtime-identity.log` is the post-restart direct identity
  check. SHA-256:
  `1c902bc5448d9e8ae14775e5eaa98bb7760968888c0484a9588f97ddf2ddd704`.
  Key result: exact SHA `1f88b553e140628e5d70f72632599f15705b1f25`,
  profile `release`, checksums `on`, shared buffers `128MB`.
- Frozen precheck command: `/home/peter/.cargo-target/debug/ecaz bench suite
  run --config crates/ecaz-cli/suites/task231-fixed-stride-10k-50k-100k.json
  --only precheck-host --manifest-output
  reviews/task-231/005-full-scale-decision/artifacts/run/suite-manifest-precheck.json
  --results-output
  reviews/task-231/005-full-scale-decision/artifacts/run/results-precheck.jsonl
  --log-file
  reviews/task-231/005-full-scale-decision/artifacts/run/suite-precheck.log`.
- `run/suite-precheck.log` SHA-256:
  `3283998e1309dc211f3801dfad4fe0c56a1bbf72971428917daa7ca784c15d7f`;
  `run/suite-manifest-precheck.json` SHA-256:
  `b5dd097905cc48a6a97a08cc0485b5956c26006b5fe57172098357a0fa8e13f4`;
  `run/results-precheck.jsonl` SHA-256:
  `bcc1fd93cc804bf3a3ea1d4e26035ed1207425d5e3613021e35edf5945f5e566`.
- Key result: one selected step completed, zero failed, zero missing or stale
  artifacts. PostgreSQL 18.3 reported checksums `on`, shared buffers `128MB`,
  extension profile `release`, and the exact run head SHA above. The remaining
  26 suite steps were selection-skipped and are not claimed as measurements.

## Invalid decision attempt 001: prepared-lock self-deadlock

- Timestamp: started `2026-08-30T01:51:00-07:00`; interrupted after diagnosis
  at approximately `2026-08-30T02:21:00-07:00`.
- Measurement extension SHA:
  `1f88b553e140628e5d70f72632599f15705b1f25`; profile `release` on every
  fixture node. Suite receipt commit at launch: `641901239`.
- Command: `/home/peter/.cargo-target/debug/ecaz bench suite run --config
  crates/ecaz-cli/suites/task231-fixed-stride-10k-50k-100k.json --resume-from
  reviews/task-231/005-full-scale-decision/artifacts/run/suite-manifest-precheck.json
  --manifest-output
  reviews/task-231/005-full-scale-decision/artifacts/run/suite-manifest.json
  --results-output
  reviews/task-231/005-full-scale-decision/artifacts/run/results.jsonl
  --log-file
  reviews/task-231/005-full-scale-decision/artifacts/run/suite-run.log`.
- Isolation: both attempted arms used fresh one-index-per-table, three-owner
  fixtures in distinct directories under `/home/peter/.ecaz/clusters/`. The
  runner cleaned the completed control fixture. The interrupted fixed fixture
  was stopped with `pg_ctl -m fast` and removed after its diagnostic was copied
  packet-locally; neither cluster is review evidence.
- `run/suite-manifest.json` SHA-256:
  `62b7181aaf1435d27ee944d5112e5af632a6fdb49fc4fce5f16eb71a16805774`.
  It correctly records only the host precheck and
  `task231-warm-10k-a-control-first` as succeeded; the fixed arm and all later
  steps remain pending. `run/suite-run.log` SHA-256:
  `b60b12bd6677d60ea4619f671fedaaa78283c8d5dc72d83900e7d410d9bc92cc`.
- The completed control summary is
  `run/warm/10k/pair-a/control-first/distann-multinode-summary.log`, SHA-256
  `24c52a1e11d35f79532c2a930ee9ac6be3fff95c8c7778fec6d4ee84940be738`.
  Key lines: distinct recall `0.9990`, warm concurrency-1 mean latency
  `7.56 ms`, physical generation bytes `242860032`, and cluster raw node-store
  DML growth `0`; every topology and workload gate passed. This arm is retained
  as diagnostic evidence only, not as a final A/B control, because the decision
  matrix will restart on one post-fix extension SHA.
- The fixed arm passed build, checksum, topology, serving, recall, latency,
  graph diagnostics, and the exact raw-store size relation before stalling in
  routed DML. Its partial log is
  `run/warm/10k/pair-a/fixed-second/distann-local-multinode.log`, SHA-256
  `6355c22373d787b9e61b58c31bcf9ec965d71a3ac50225c5286dcdeff1e2dfed`.
  Those partial measurements are invalid and are not used for an A/B claim.
- `run/fixed-stride-10k-a-stall-diagnostic.md` SHA-256:
  `86cdeb31b4cbdb374fd723e06749dea9babdd6ccaabbf9b5fc9404088acba417`.
  It proves a prepared remote transaction held the fixed node store's
  self-conflicting `ShareRowExclusiveLock` while a later backlink transaction
  to the same owner waited for that lock. The coordinator could therefore
  never reach the callback that resolved the prepared transaction.
- Disposition: the attempt is invalid, not STOP/PROMOTE evidence. Code
  checkpoint `fc4a4292681715d899a80d7df251955b5de6f711` releases the raw-tail
  lock when the owner-local mutation context ends, before remote prepare. The
  full matrix will restart from a fresh precheck and fresh fixtures only after
  Packet 006 review closes that correction.
