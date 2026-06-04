# Suite Report: aws-spire-1m-rabitq-global1152-minimal

- config: `benchmarks/aws-spire-1m/001-run/suite-minimal.json`
- config_sha256: `f2cb2a168517f7ee352bc69598ff98a679e412d053577d5c0d64198920334674`
- dry_run: `false`
- steps: completed 3, failed 0, skipped 0, dry-run 0, missing artifacts 2, stale 0

| Step | Kind | Status | Duration ms | Artifacts |
| --- | --- | --- | ---: | --- |
| precheck-existing-spire-1m-index | raw | Succeeded | 40 | `benchmarks/aws-spire-1m/001-run/artifacts/minimal/precheck-existing-spire-1m-index.log` |
| pipeline-spire-1m-rabitq-global1152-minimal | spire-pipeline | Succeeded | 134092 | `${artifact_dir}/pipeline-spire-1m-rabitq-global1152-minimal.log` |
| storage-spire-1m-rabitq-global1152-minimal | storage | Succeeded | 39 | `${artifact_dir}/storage-spire-1m-rabitq-global1152-minimal.log` |
wrote benchmarks/aws-spire-1m/001-run/artifacts/minimal/results-report.jsonl
