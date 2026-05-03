---
id: NFR-009
title: CLI Drift and Artifact Discipline
type: non-functional-requirement
artifact_type: NFR
status: APPROVED
relationships:
  - target: "ix://agent-ix/tqvector/US-016"
    type: "constrains"
    cardinality: "N:1"
---
# NFR-009: CLI Drift and Artifact Discipline

## Requirement

Ecaz SHALL keep the operator CLI aligned with the implemented extension surface and make CLI-produced evidence reproducible from packet-local artifacts.

## Policy

1. The operator README command tree SHALL match the implemented Clap command tree.
2. CLI profile metadata SHALL be audited against extension access-method names, opclasses, reloptions, and scan GUCs whenever those surfaces change.
3. Evidence-producing CLI runs SHOULD use `--log-file review/<topic>/artifacts/<run>.log`.
4. Review packets that cite CLI measurements SHALL store raw logs under the packet `artifacts/` directory and cite the command used.
5. Long benchmark sequences SHOULD use `ecaz bench suite` configs instead of shell scripts so dry-run manifests, status checks, and packet-local artifact paths remain auditable.
6. Until shared constants are extracted into a common crate, `profiles.rs` is the accepted drift watch point between the CLI and extension.

## Acceptance Criteria

### NFR-009-AC-1

Docs expose the current CLI command tree and link from the README, usage guide, getting-started guide, and benchmark docs.

### NFR-009-AC-2

The test matrix traces the CLI user story, functional requirement, and drift discipline to a validation case.

### NFR-009-AC-3

Benchmark docs instruct operators to use packet-local CLI logs for review evidence.
