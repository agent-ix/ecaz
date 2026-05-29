# Review Request: SPIRE Phase 13e AWS Preserve and Smoke Truth Harness

## Summary

This fixes two harness problems exposed by packet `1060`:

- The pass watchdog tore down the loaded AWS cluster after a benchmark/harness failure. It now preserves resources on ordinary pass failure by default and only tears down on success unless `SPIRE_AWS_TEARDOWN_ON_EXIT=always` is set. The timeout watchdog still tears down after the configured timeout for cost safety.
- Representative smoke used `bench spire-pipeline --include-recall` without a truth corpus file, so it could still fetch the full coordinator corpus table over the tunnel. Smoke now passes the staged representative TSV via `--truth-corpus-file`.

The representative performance pass now also builds the local release `ecaz` CLI before provisioning, so the tunnel-side benchmark runner cannot use a stale `target/release/ecaz`.

## Evidence

See `artifacts/manifest.md`.

Key local validation:

- Watchdog local check passed and verifies failing passes preserve AWS resources by default.
- Representative performance preflight passed.
- Local release `ecaz` build completed.
- Current `target/release/ecaz bench spire-pipeline --help` and `bench recall --help` both expose `--truth-corpus-file`.

## Scope

Harness-only. This does not change SPIRE placement, remote execution, tuple transport, or pooling behavior. It prevents teardown churn for unrelated failures and closes the remaining full-corpus SQL fetch path in representative smoke.
