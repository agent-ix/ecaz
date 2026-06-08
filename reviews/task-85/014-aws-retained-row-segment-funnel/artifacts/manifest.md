head_sha: 112317368431dd0c09622a0f1197f889a76f8d21
task_bucket: reviews/task-85
packet_path: reviews/task-85/014-aws-retained-row-segment-funnel
timestamp: 2026-06-07
lane: aws-1m
fixture: task67_1m_hnsw_m7g2xlarge
storage_format: rabitq
rerank_mode: heap rerank width 25
surface: retained block16/global1152
isolated_one_index_per_table: shared retained AWS table/index

# Artifacts

- `suite-aws-1m-retained-row-segment-funnel-q500.json`
  - command: `target/debug/ecaz bench suite audit --config reviews/task-85/014-aws-retained-row-segment-funnel/suite-aws-1m-retained-row-segment-funnel-q500.json`
  - result: audit passed, 3 steps.
- `artifacts/suite-audit.log`
  - command output for suite audit.
- `artifacts/cloud-resume-before-row-segment-funnel.log`
  - command: `target/debug/ecaz cloud resume --profile 1m --database postgres --log-file reviews/task-85/014-aws-retained-row-segment-funnel/artifacts/cloud-resume-before-row-segment-funnel.log`
  - result: AWS 1M resumed.
- `artifacts/cloud-install-row-segment-funnel.log`
  - command: `target/debug/ecaz cloud install --profile 1m --database postgres --git-ref task-85-spire-product-scale-pareto --skip-extension-recreate --timeout 3600 --log-file reviews/task-85/014-aws-retained-row-segment-funnel/artifacts/cloud-install-row-segment-funnel.log`
  - result: install succeeded.
- `artifacts/cloud-bench-row-segment-funnel.log`
  - command: `target/debug/ecaz cloud bench --profile 1m --database postgres --config reviews/task-85/014-aws-retained-row-segment-funnel/suite-aws-1m-retained-row-segment-funnel-q500.json --suite task85-aws-1m-retained-row-segment-funnel-q500 --ecaz-bin /usr/local/bin/ecaz --log-file reviews/task-85/014-aws-retained-row-segment-funnel/artifacts/cloud-bench-row-segment-funnel.log`
  - result: failed in the first pipeline step because the retained DB extension
    signature did not expose `leaf_row_segment_read_count`.
- `artifacts/cloud-pause-after-row-segment-funnel-fail.log`
  - command: `target/debug/ecaz cloud pause --profile 1m --database postgres --log-file reviews/task-85/014-aws-retained-row-segment-funnel/artifacts/cloud-pause-after-row-segment-funnel-fail.log`
  - result: pause requested after failed AWS run.
- `artifacts/aws-ec2-status-final-after-fail.log`
  - command: `aws ec2 describe-instances ...`
  - result: DB and loader were both `stopped`.
- `artifacts/cloud-resume-before-row-segment-funnel-rerun.log`
  - command: `target/debug/ecaz cloud resume --profile 1m --database postgres --log-file reviews/task-85/014-aws-retained-row-segment-funnel/artifacts/cloud-resume-before-row-segment-funnel-rerun.log`
  - result: AWS 1M resumed for fallback validation rerun.
- `artifacts/cloud-install-row-segment-funnel-rerun.log`
  - command: `target/debug/ecaz cloud install --profile 1m --database postgres --git-ref task-85-spire-product-scale-pareto --skip-extension-recreate --timeout 3600 --log-file reviews/task-85/014-aws-retained-row-segment-funnel/artifacts/cloud-install-row-segment-funnel-rerun.log`
  - result: install succeeded after fallback commit `0fd494def`.
- `artifacts/cloud-bench-row-segment-funnel-rerun.log`
  - command: `target/debug/ecaz cloud bench --profile 1m --database postgres --config reviews/task-85/014-aws-retained-row-segment-funnel/suite-aws-1m-retained-row-segment-funnel-q500.json --suite task85-aws-1m-retained-row-segment-funnel-q500 --ecaz-bin /usr/local/bin/ecaz --log-file reviews/task-85/014-aws-retained-row-segment-funnel/artifacts/cloud-bench-row-segment-funnel-rerun.log`
  - result: failed with the same missing-column error; the first fallback guard
    matched only `err.to_string()`, not the structured Postgres DB error
    message.
- `artifacts/cloud-pause-after-row-segment-funnel-rerun-fail.log`
  - command: `target/debug/ecaz cloud pause --profile 1m --database postgres --log-file reviews/task-85/014-aws-retained-row-segment-funnel/artifacts/cloud-pause-after-row-segment-funnel-rerun-fail.log`
  - result: pause requested after failed rerun.
- `artifacts/aws-ec2-status-final-after-rerun-fail.log`
  - command: `aws ec2 describe-instances ...`
  - result: DB and loader were both `stopped`.
- `artifacts/cloud-resume-before-structured-fallback-rerun.log`
  - command: `target/debug/ecaz cloud resume --profile 1m --database postgres --log-file reviews/task-85/014-aws-retained-row-segment-funnel/artifacts/cloud-resume-before-structured-fallback-rerun.log`
  - result: AWS 1M resumed for structured fallback rerun.
- `artifacts/cloud-cli-install-structured-fallback.log`
  - command: AWS SSM CLI-only install of `ecaz-cli` at `f17af966c`.
  - result: succeeded; `/usr/local/bin/ecaz --version` returned `ecaz 0.1.0`.
- `artifacts/cloud-bench-row-segment-funnel-structured-rerun.log`
  - command: `target/debug/ecaz cloud bench --profile 1m --database postgres --config reviews/task-85/014-aws-retained-row-segment-funnel/suite-aws-1m-retained-row-segment-funnel-q500.json --suite task85-aws-1m-retained-row-segment-funnel-q500 --ecaz-bin /usr/local/bin/ecaz --log-file reviews/task-85/014-aws-retained-row-segment-funnel/artifacts/cloud-bench-row-segment-funnel-structured-rerun.log`
  - result: succeeded and synced suite artifacts.
- `artifacts/cloud-pause-after-structured-fallback-success.log`
  - command: `target/debug/ecaz cloud pause --profile 1m --database postgres --log-file reviews/task-85/014-aws-retained-row-segment-funnel/artifacts/cloud-pause-after-structured-fallback-success.log`
  - result: pause requested after successful suite.
- `artifacts/aws-ec2-status-final-after-structured-success.log`
  - command: `aws ec2 describe-instances ...`
  - result: DB and loader were both `stopped`.

# Key Result Lines

- Suite audit: `audit passed: 3 steps`.
- Install: `install: profile=1m db=10.42.1.131 ref=task-85-spire-product-scale-pareto ok`.
- Precheck succeeded and showed the retained DB still returned the legacy
  `ec_spire_index_scan_leaf_candidate_snapshot` column list.
- Bench failure: `ERROR: column "leaf_row_segment_read_count" does not exist`.
- Fallback rerun failure after `0fd494def`: same missing-column error because
  the guard did not inspect `err.as_db_error().message()`.
- Structured fallback success after `f17af966c`: warm repeat
  `recall@10=0.9876`, `candidate_sum=9,213,846`, `heap_rerank_sum=12,500`,
  `p50=225.805 ms`, `p95=285.171 ms`, `p99=296.588 ms`.
- Final AWS state: `ecaz-cloud-1m-loader stopped`, `ecaz-cloud-1m-db stopped`.

# Interpretation

This was not a retained-surface benchmark result. It proved that
`--skip-extension-recreate` preserves the 1M corpus/index data but also keeps
the old pgrx SQL return signature for the snapshot function. The next
checkpoint must either update the extension signature without dropping data or
make `ecaz bench spire-pipeline` compatible with the legacy signature.

The final structured fallback rerun is a valid retained recall/latency row, but
not valid evidence for actual selected row-segment bytes. Because the first
row-segment implementation inserted columns in the middle of the table-returning
function, the retained DB's old SQL signature can mislabel subsequent returned
tuple positions after loading the new shared library. The next code checkpoint
must make new columns append-only before using legacy-signature funnel split
fields for physical-layout decisions.
