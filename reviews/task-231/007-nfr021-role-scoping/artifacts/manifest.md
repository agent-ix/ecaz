# Task 231 Packet 007 artifact manifest

- Head SHA: `795af9616a304f2bf276d57c2c151270198f9bd4`.
- Task bucket and packet:
  `reviews/task-231/007-nfr021-role-scoping/`.
- Scope: benchmark-runner derived NFR-021 evidence aggregation only; no
  extension or fixture behavior changed.
- Measurement source: Packet 005's all-succeeded 27-step manifest at accepted
  extension SHA `66b53998a955b583ca43c0e967806aa29e0a4404`.

## `focused-test.log`

- Timestamp: `2026-08-30T12:07:39-07:00`.
- Command: `cargo test -p ecaz-cli
  distann_nfr_021_same_variant_does_not_mix_decision_roles` (captured through
  `script -q -e -c`).
- SHA-256:
  `232ee18345c292d2bfb2a6d4f8d6fcf35377cb183acb6e14d058bbf9a271c1ba`.
- Key result: `1 passed; 0 failed`; exit code 0.

## Unchanged-fixture derived re-extraction

- Runner head: `6bd18e2d1d9d5fe154b92e2dd1f6fc316132f82d`; role-scoping code checkpoint:
  `795af9616a304f2bf276d57c2c151270198f9bd4`.
- Command: `/home/peter/.cargo-target/debug/ecaz bench suite run --config
  crates/ecaz-cli/suites/task231-fixed-stride-10k-50k-100k.json --resume-from
  reviews/task-231/005-full-scale-decision/artifacts/run/suite-manifest.json
  --manifest-output
  reviews/task-231/005-full-scale-decision/artifacts/run/suite-manifest.json
  --results-output
  reviews/task-231/005-full-scale-decision/artifacts/run/results.jsonl
  --log-file
  reviews/task-231/005-full-scale-decision/artifacts/run/suite-run.log`.
- Packet 005 `run/suite-run.log` SHA-256:
  `0d51932121514900716e6c833dca51c4ca1e0071187cad628461c1d11107c60c`;
  key result: all 27 succeeded fixture steps reused, results regenerated,
  task-owned run directories cleaned, exit code 0.
- Packet 005 `run/results.jsonl` SHA-256:
  `94f0fb9928c9e16ad8f613987d7bd15c4be65aa9869fa6f8dbea1f9a57b78029`;
  key result: all registrations conforming and decision-eligible, candidate
  maximum normalized per-owner growth `0.998937`, control `1.095044`, bound
  `2.0`; zero non-owned records, orphans, unsharded derived bytes, or
  coordinator-resident unsharded bytes, and constant head capacity.
- The pre-correction source is preserved in Packet 005 as
  `run/results-pre-role-scope.jsonl`, SHA-256
  `3f9c87ec156aa473b7cb7d5e2f84c2c087d3112f90fe2e40ba3a92862b5a277f`.
  Its false `2.171204` row is the exact cross-role contamination this patch
  corrects; no fixture measurement changed.
