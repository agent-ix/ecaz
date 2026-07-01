# Task 121 Packet 002 - Phase 0 Local Multi-Node Suite Lane

## Review Request

Please review the Phase 0 tooling slice for Task 121.

This slice adds a first-class suite step, `kind: "spire-local-multinode"`, so a
Task 121 `SuiteConfig` can address the local distributed SPIRE lane directly:

- one coordinator PostgreSQL instance
- three worker PostgreSQL instances
- distinct local ports for all four instances
- packet-local artifact directory wiring
- representative/correctness tier knobs
- bench-suite knobs for the downstream local distributed `spire-pipeline` gate

The new suite step expands to `ecaz dev spire-multicluster local-multinode-pg18`,
which wraps the existing local four-instance runner from Task 120 under a
non-AWS CLI name. This is still local-only and does not authorize AWS work.

## What Changed

- `crates/ecaz-cli/src/commands/bench/suite.rs`
  - added `SuiteStep::SpireLocalMultinode`
  - added validation, artifact-dir templating, expected artifact tracking, input
    path tracking, and command expansion
- `crates/ecaz-cli/src/commands/dev/spire_multicluster.rs`
  - added `local-multinode-pg18` as the named local four-instance SPIRE command

## Evidence

See `artifacts/manifest.md`.

This packet is a Phase 0 tooling checkpoint only. No new benchmark results are
claimed here.
