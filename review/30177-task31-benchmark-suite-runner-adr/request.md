# Task 31 Benchmark Suite Runner ADR

Reviewer: please review ADR-050 for the proposed long-running configured
benchmark suite runner.

## Scope

This packet adds design rationale only. It does not implement the runner.

The ADR is based on the Task 31 M5 IVF measurement sequence from packets
`30169` through `30176`, plus the expected need to onboard and tune indexes on
other architectures such as RDS Graviton.

## Change

Added:

- `spec/adr/ADR-050-configured-benchmark-suite-runner.md`

Updated:

- `spec/adr/index.md`

## Decision Summary

ADR-050 proposes an `ecaz bench suite --config <path>` command that consumes a
versioned structured JSON config and runs existing `ecaz` primitives:

- `corpus load`
- `bench recall`
- `bench latency`
- `bench storage`
- `dev sql` for EXPLAIN/counters
- raw `ecaz` escape-hatch steps for commands not yet modeled

The ADR requires:

- long-running unattended operation
- raw artifact preservation
- machine-readable suite manifests
- optional Markdown summaries
- support for full onboarding suites and narrow tuning suites
- architecture/cloud metadata for targets such as RDS Graviton
- dry-run, named-step selection, tags, resume, and continue-on-error behavior

## Validation

No cargo, pgrx, or benchmark tests were run. This is a documentation-only ADR
checkpoint; validation was static review of the ADR and `git diff --check`.

## Open Review Questions

- Is JSON acceptable as the initial config format, or should TOML/YAML be
  chosen before implementation?
- Should review-packet generation be part of v1, or should v1 stop at
  `suite-manifest.json` plus raw artifacts?
- Which metadata should be mandatory for cloud runs versus optional for local
  development runs?
