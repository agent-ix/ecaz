# Task 84 Packet 005 Artifact Manifest

- Task: `plan/tasks/84-spire-1m-recall-recovery-without-candidate-inflation.md`
- Packet: `reviews/task-84/005-multi-index-knn-frontdoor-fix/`
- Branch: `task-84-spire-recall-recovery`
- Code commit under review: `07974586f`
- Remote branch head used by successful AWS install: `f0753852d`
- AWS profile: `1m`
- Database: `postgres`
- Corpus prefix: `task67_1m_hnsw_m7g2xlarge`
- Index under measurement: `aws_spire_1m_rabitq_t84_k3_block16_tg256_idx`
- Suite config:
  `reviews/task-84/004-aws-1m-k3-summary-build/suite-aws-1m-k3-summary-query-only-q500.json`
- Packet-local copied suite artifacts:
  `reviews/task-84/005-multi-index-knn-frontdoor-fix/artifacts/aws-1m-k3-summary-query-only-after-frontdoor-fix/`

## Local Validation

- `cargo pgrx test pg18 test_ec_spire_multi_index_knn_select_passes_through`
  - Result: passed.
  - Key line: `test tests::pg_test_ec_spire_multi_index_knn_select_passes_through ... ok`
- `cargo pgrx test pg18 test_ec_spire_dml_frontdoor_hook_fail_closed_context_error`
  - Result: passed.
  - Key line:
    `test tests::pg_test_ec_spire_dml_frontdoor_hook_fail_closed_context_error ... ok`
- `cargo build -p ecaz-cli`
  - Result: passed.
  - Note: existing warning for `LoadedDistributedPlacementConfig.path`.

## Cloud Lifecycle Artifacts

- `cloud-status-before-fix-query-only.log`
  - Command: `target/debug/ecaz cloud status --profile 1m --database postgres`
  - Result: initial state `paused`.
- `cloud-resume-fix-query-only.log`
  - Command: `target/debug/ecaz cloud resume --profile 1m --database postgres`
  - Result: `resume: profile=1m db=10.42.1.131 ready`.
- `cloud-install-frontdoor-fix.log`
  - Command: attempted unsupported `--commit 07974586f`.
  - Result: CLI usage error; corrected in later install.
- `cloud-install-frontdoor-fix-supported.log`
  - Command: `target/debug/ecaz cloud install --profile 1m --database postgres`
  - Result: interrupted by pause, SSM rc=143.
- `cloud-pause-after-install-stall.log`
  - Command: `target/debug/ecaz cloud pause --profile 1m --database postgres`
  - Result: pause requested after the premature install interruption.
- `cloud-status-final-paused-after-install-stall.log`
  - Command: `target/debug/ecaz cloud status --profile 1m --database postgres`
  - Result: state `paused`.
- `cloud-resume-fix-query-only-rerun.log`
  - Command: resume before install rerun.
  - Result: ready.
- `cloud-install-frontdoor-fix-rerun.log`
  - Command: install from default `main`.
  - Result: failed with SSM rc=1 because main was installed and
    `DROP EXTENSION` could not run while benchmark tables depended on
    extension-owned types.
- `ssm-1356d471-install-main-drop-extension-fail.json`
  - Command: `aws ssm get-command-invocation --command-id 1356d471-0a15-457e-83d2-45299c5ecd13 --instance-id i-06ace3e95ab942623`
  - Result: detailed install failure; remote checkout used `main`, then
    `DROP EXTENSION IF EXISTS ecaz` failed due dependent benchmark tables.
- `cloud-pause-after-install-fail-rerun.log`
  - Command: pause after failed main install.
  - Result: pause requested.
- `cloud-status-final-paused-after-install-fail-rerun.log`
  - Command: final status after failed main install.
  - Result: state `paused`.
- `cloud-resume-branch-install.log`
  - Command: resume before branch install.
  - Result: ready.
- `cloud-install-branch-frontdoor-fix.log`
  - Command:
    `target/debug/ecaz cloud install --profile 1m --database postgres --git-ref task-84-spire-recall-recovery --skip-extension-recreate`
  - Result: `install: profile=1m db=10.42.1.131 ref=task-84-spire-recall-recovery ok`.
- `cloud-bench-k3-query-only-after-frontdoor-fix.log`
  - Command:
    `target/debug/ecaz cloud bench --profile 1m --database postgres --config reviews/task-84/004-aws-1m-k3-summary-build/suite-aws-1m-k3-summary-query-only-q500.json`
  - Result: synced successful suite artifacts from
    `s3://ecaz-cloud-1m-b62eb804/bench-artifacts/smoke/20260606T195440Z/`.
- `cloud-pause-after-branch-query-only.log`
  - Command: pause after successful branch query-only run.
  - Result: pause requested.
- `cloud-status-final-paused-after-branch-query-only.log`
  - Command: final status after successful run.
  - Result: state `paused`, cost `~$0.00/hr running`.

## Suite Artifacts

Packet-local copies are under
`artifacts/aws-1m-k3-summary-query-only-after-frontdoor-fix/`.

- `suite-run.log`
  - Result: all three pipeline rows and storage step executed.
- `suite-manifest.json`
  - Result: `pipeline-spire-1m-rabitq-k3-block16-global1024-q500`,
    `global1152`, `global1280`, and storage all `succeeded`.
- `results.jsonl`
  - Key rows:
    - `global1024`: `recall@k=0.9808`, `candidate_sum=8190090`,
      `heap_rerank_sum=12500`, `p50=264.202 ms`, `p95=340.013 ms`,
      `p99=2902.047 ms`.
    - `global1152`: `recall@k=0.9832`, `candidate_sum=9213742`,
      `heap_rerank_sum=12500`, `p50=252.186 ms`, `p95=311.879 ms`,
      `p99=324.200 ms`.
    - `global1280`: `recall@k=0.9846`, `candidate_sum=10237430`,
      `heap_rerank_sum=12500`, `p50=258.684 ms`, `p95=314.655 ms`,
      `p99=330.054 ms`.
- `pipeline-spire-1m-rabitq-k3-block16-global1024-q500.log`
  - Source pipeline log for the lower cap.
- `pipeline-spire-1m-rabitq-k3-block16-global1152-q500.log`
  - Source pipeline log for the retained-budget comparison.
- `pipeline-spire-1m-rabitq-k3-block16-global1280-q500.log`
  - Source pipeline log for the blanket-cap control neighborhood.
- `miss-attribution-spire-1m-k3-block16-global1152-q500.jsonl`
  - `5000` truth rows.
  - Miss split from `jq -r 'select(.hit == false) | .miss_stage' ... | sort | uniq -c`:
    `3 routing_miss`, `81 selected_leaf_block_pruning_or_candidate_cap`.
- `miss-attribution-spire-1m-k3-block16-global1280-q500.jsonl`
  - `5000` truth rows.
  - Miss split: `3 routing_miss`,
    `74 selected_leaf_block_pruning_or_candidate_cap`.
- `storage-spire-1m-rabitq-k3-block16-tg256.log`
  - Key rows:
    - `aws_spire_1m_rabitq_t84_k3_block16_tg256_idx`: `936.4 MiB`,
      `991.9 B/row`.
    - `aws_spire_1m_rabitq_t80_block16_tg256_idx`: `872.1 MiB`,
      `923.7 B/row`.

## Result

The front-door fix removes packet 004's relation-context blocker and preserves
ADR-069 fail-closed behavior for PK/DML shapes. The AWS measurement is valid
and negative for k=3 summary count as a Task 84 recovery policy:

- It does not beat retained `recall@10=0.9832` at the direct `global1152`
  comparison.
- It preserves the retained candidate surface (`9,213,742` vs baseline
  `9,213,846`) but recovers no selected-leaf misses.
- Wider caps match the prior blanket-cap behavior rather than producing a new
  Pareto point.
