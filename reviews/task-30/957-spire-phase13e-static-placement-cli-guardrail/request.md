# Review Request: SPIRE Phase 13e Static Placement Task And CLI Guardrail

## Summary

This checkpoint opens Task 30 Phase 13e as the explicit AWS production gap
closure lane and adds the first executable guardrail for distributed SPIRE load
work.

Code/task commit: `94ecb3eae9a7faa174517f052520484141b71acf`

## Changes

- Added `plan/tasks/task30-phase13e-spire-aws-production-gap-closure.md`.
- Indexed Phase 13e from `plan/tasks/README.md`.
- Added `ecaz corpus load --distributed-placement-config <path>`.
- Added strict JSON validation for static SPIRE remote placement configs:
  version, coordinator index name, remote node IDs, secret/index names,
  hash-source-identity policy, complete shard coverage, duplicate shard
  rejection, and missing shard rejection.
- The new flag is only accepted for `--profile ec_spire`.
- A valid config currently fails closed with an explicit Task 30 Phase 13e.1
  message instead of silently building a local-only fixture.

## Validation

See `artifacts/manifest.md`.

- `cargo test -p ecaz-cli distributed_placement_config`
- Result: 5 passed, 0 failed

## Review Notes

The implementation deliberately does not claim distributed load is complete.
This is the first checkpoint in 13e.1: it defines the operator-facing static
placement config shape and blocks false AWS distributed runs until remote shard
materialization is implemented.
