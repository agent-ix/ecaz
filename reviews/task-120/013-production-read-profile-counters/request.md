# Task 120 Production Read Profile Counters

Please review the CLI reporting checkpoint that exposes richer SPIRE production
read-profile counters for Phase 5 distributed measurement.

## Scope

The code checkpoint updates
`crates/ecaz-cli/src/commands/bench/spire_pipeline.rs` only.

`bench spire-pipeline --include-production-read-profile` already queried
`ec_spire_remote_search_production_read_profile`, whose rows include worker and
merge counters. This change carries more of those existing metrics into the
aggregate table and suite result rows:

- local, remote, and skipped PID sums
- compact candidate sum
- remote heap ready/failed dispatch sums
- remote and local heap candidate sums
- payload row and byte sums
- merge input, duplicate vec ID, and output sums
- strict fail sum

No scan behavior, SQL surface, benchmark suite config, Terraform config, or AWS
load logic changed.

## Evidence

- Artifact manifest:
  `reviews/task-120/013-production-read-profile-counters/artifacts/manifest.md`
- Formatting:
  `reviews/task-120/013-production-read-profile-counters/artifacts/cargo-fmt-check.log`
- Focused tests:
  `reviews/task-120/013-production-read-profile-counters/artifacts/cargo-test-ecaz-cli-spire-pipeline.log`

## Result

The focused CLI test target passed (`23 passed`). The upcoming Phase 5
distributed packet can now cite worker candidate generation, exact heap
candidate volume, payload rows/bytes, and coordinator merge/dedupe counters
from standard `ecaz bench suite` output.

This is not Task 120 closeout. It is measurement-surface enablement for the AWS
distributed near-data rerank run.
