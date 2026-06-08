# Task 85 Funnel Read/Score Breakdown Manifest

- head SHA: `5c5def28aad09bfdf336d591984c0fd3295d9990`
- task bucket: `reviews/task-85/`
- packet path: `reviews/task-85/009-funnel-read-score-breakdown/`
- lane: Task 85 benchmark harness/evidence extension
- storage format: SPIRE RaBitQ evidence surface; no index format change
- rerank mode: unchanged
- timestamp: 2026-06-07
- isolated/shared surface: not applicable; this packet changes CLI artifact
  projection only and does not run a benchmark matrix

## Commands

Format check:

```text
script -q -c 'cargo fmt --check' reviews/task-85/009-funnel-read-score-breakdown/artifacts/cargo-fmt-check.log
```

Focused tests:

```text
script -q -c 'cargo test -p ecaz-cli spire_pipeline' reviews/task-85/009-funnel-read-score-breakdown/artifacts/cargo-test-ecaz-cli-spire-pipeline.log
```

AWS final status:

```text
aws ec2 describe-instances --instance-ids i-06ace3e95ab942623 --query 'Reservations[].Instances[].{InstanceId:InstanceId,State:State.Name,PrivateIp:PrivateIpAddress,PublicIp:PublicIpAddress,Name:Tags[?Key==`Name`]|[0].Value}' --output table
```

## Artifacts

- `cargo-fmt-check.log`: `cargo fmt --check` passed; log includes the repo's
  existing stable-rustfmt warnings about nightly-only import settings.
- `cargo-test-ecaz-cli-spire-pipeline.log`: focused CLI test run passed,
  `21 passed; 0 failed`.
- `aws-ec2-status-final.log`: direct AWS status shows
  `i-06ace3e95ab942623` / `ecaz-cloud-1m-db` is `stopped`.

## Key Result

`ecaz bench spire-pipeline --funnel-output` now records Task 85 read/score
breakdown fields in JSONL artifacts:

- object, summary, and row bytes;
- available, selected, and skipped leaf block counts;
- split summary-score and row-score nanoseconds.

This is an evidence-enabling checkpoint for the comprehensive Task 85 latency
program. It is not a product Pareto claim.
