---
id: NFR-009
title: CLI Drift and Artifact Discipline
type: NFR
quality_attribute: maintainability
status: APPROVED
relationships:
  - target: "ix://agent-ix/ecaz/US-016"
    type: "constrains"
    cardinality: "N:1"
---
# NFR-009: CLI Drift and Artifact Discipline

## Statement

Ecaz SHALL keep the operator CLI aligned with the implemented extension surface and make CLI-produced evidence reproducible from packet-local artifacts.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|---|---|---|---|
| CLI command-tree drift | operator README command tree matches the implemented Clap command tree | no undocumented drift | CLI drift audit (README vs Clap tree, `profiles.rs` watch point) |
| CLI profile metadata drift | profile metadata matches extension access-method names, opclasses, reloptions, and scan GUCs | re-audited whenever those surfaces change | profile metadata audit against the extension surface |
| Packet-local CLI evidence compliance | 100% of review packets citing CLI measurements store raw logs and the command used under packet `artifacts/` | no exceptions | review packet audit |

## Policy

1. The operator README command tree SHALL match the implemented Clap command tree.
2. CLI profile metadata SHALL be audited against extension access-method names, opclasses, reloptions, and scan GUCs whenever those surfaces change.
3. Evidence-producing CLI runs SHOULD use `--log-file reviews/task-{id}/001-<topic>/artifacts/<run>.log`.
4. Review packets that cite CLI measurements SHALL store raw logs under the packet `artifacts/` directory and cite the command used.
5. Long benchmark sequences SHOULD use `ecaz bench suite` configs instead of shell scripts so dry-run manifests, status checks, and packet-local artifact paths remain auditable.
6. Until shared constants are extracted into a common crate, `profiles.rs` is the accepted drift watch point between the CLI and extension.

## Verification

Compliance is checked by auditing the operator README command tree against the
implemented Clap command tree (with `profiles.rs` as the accepted drift watch
point until shared constants are extracted), auditing CLI profile metadata
against extension access-method names, opclasses, reloptions, and scan GUCs
whenever those surfaces change, and reviewing packets that cite CLI
measurements for packet-local raw logs (`--log-file` into `artifacts/`) and
recorded commands. The test matrix is checked to trace the CLI user story,
functional requirement, and drift discipline to a validation case.

## Acceptance Criteria

### NFR-009-AC-1

Docs expose the current CLI command tree and link from the README, usage guide, getting-started guide, and benchmark docs.

### NFR-009-AC-2

The test matrix traces the CLI user story, functional requirement, and drift discipline to a validation case.

### NFR-009-AC-3

Benchmark docs instruct operators to use packet-local CLI logs for review evidence.
