---
id: 30157
title: Ecaz CLI Docs and Spec
agent: coder1
status: open
created: 2026-05-01
checkpoint_commit: 27cf5342
---
# Review Request: Ecaz CLI Docs and Spec

## Summary

This docs/spec checkpoint makes `ecaz-cli` a first-class documented operator surface.

The checkpoint:

- links the main README, usage guide, getting-started guide, and benchmark docs to the operator CLI
- refreshes `crates/ecaz-cli/README.md` to include the implemented `bench`, `compare`, `quant`, `stress`, and nested `dev` command tree
- adds `US-016`, `FR-037`, and `NFR-009` for the CLI operator surface, command inventory, profile behavior, logging, and drift discipline
- extends `spec/tests.md` with `TC-019` so the CLI docs/spec surface is traceable through the matrix

## Files To Review

- `README.md`
- `docs/usage.md`
- `docs/getting-started.md`
- `docs/benchmarks.md`
- `crates/ecaz-cli/README.md`
- `spec/spec.md`
- `spec/tests.md`
- `spec/usecase/US-016-operator-cli.md`
- `spec/functional/FR-037-ecaz-cli-operator-surface.md`
- `spec/non-functional/NFR-009-cli-drift-and-artifact-discipline.md`

## Validation

- `git diff --cached --check`
- No code tests run. This is a docs/spec-only checkpoint under the repository checkpoint policy.

## Reviewer Focus

1. Does the operator CLI README now match the implemented Clap command surface closely enough?
2. Are the README and user docs pointing users to the right CLI workflow for repeatable corpus and benchmark work?
3. Is the new CLI spec normative without overclaiming unexecuted benchmark behavior?
4. Does `TC-019` give enough traceability for the CLI surface and docs-drift risk?
