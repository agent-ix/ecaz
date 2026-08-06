# Task 215 release matrix manifest

- Decision: STOP; no BW64/H8 promotion
- Rollback checkpoint: `01384502f`
- Rollback validation: normal PG18 `cargo check` and release `cargo pgrx install`
  completed with BW4/H100 defaults restored
- Decision artifact: `artifacts/decision.md`
- Reconciliation artifact: `artifacts/reconciliation-206.md`
- Decision run: `artifacts/run-r2/`
- Decision run source SHA: `ea51a9c8bdce1f412652ac743ae0d055af8daa76`
- Decision run profile: normal PG18 release, attribution feature absent,
  three sharded owners, no coordinator full-graph replica
- Decision run result: six succeeded, zero failed, zero skipped
- Superseded attempt: `artifacts/run/` failed stale-schema preflight before
  benchmark evidence and is not cited

The checked-in suite config is `artifacts/task215-release-ab.json`. The
structured provenance is in `artifacts/run-r2/suite-manifest.json`, normalized
rows are in `artifacts/run-r2/results.jsonl`, and the generated report is in
`artifacts/run-r2/report.md`. Each arm's cited compact summary is its
`distann-multinode-summary.log` under `artifacts/run-r2/{control,candidate}-{10k,50k,100k}/`.

- Run command (durable arguments): `/home/peter/.cargo-target/release/ecaz bench suite run
  --config reviews/task-215/003-release-matrix-and-decision/artifacts/task215-release-ab.json
  --artifact-dir reviews/task-215/003-release-matrix-and-decision/artifacts/run-r2
  --manifest-output reviews/task-215/003-release-matrix-and-decision/artifacts/run-r2/suite-manifest.json
  --results-output reviews/task-215/003-release-matrix-and-decision/artifacts/run-r2/results.jsonl
  --continue-on-error`
- The transient runner log was not retained; the packet-local retained report
  log is `artifacts/run-r2/suite-report.log`.
- Entry-gate accounting: standalone Task 208/210 evidence was skipped. The
  matrix's topology/engagement/storage rows are cited as matrix evidence only,
  not as a replacement for those entry-gate packets.
- Run captured: `2026-08-06T17:16:49Z` manifest/report audit
- Config SHA256: `3c9e3bf558070ff08a83b1c8e95862fd647a5a92619aa8a205ad6e24ab1ae035`
- Suite manifest SHA256: `afa4cab65d37c1c48ffbfd9e6a98048d3376ef350e0f7e3346b60487cda31a38`
- Results SHA256: `23318c3570e6a9a683d36a0fb3b717645f0fdd406ee69ed30ff2dcfcc747d8cd`
- Report SHA256: `60b3496ceedb7744805b6b476db0c76c544eae5dfc0b78e8e2bda447610a657d`
