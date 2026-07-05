# Artifact Manifest

## Static Validation

- head SHA: `a2e59a42`
- packet/topic: `31143-standards-compliance-claim-fixes`
- lane / fixture / storage format / rerank mode: docs/spec static validation
- command used: `git diff --check`
- timestamp: 2026-05-16T21:17:14Z
- isolated/shared surface: not applicable
- key result lines: command exited successfully with no output

## Spec Inventory

- head SHA: `a2e59a42`
- packet/topic: `31143-standards-compliance-claim-fixes`
- lane / fixture / storage format / rerank mode: docs/spec inventory
- command used: local Python inventory over `spec/**/*.md`
- timestamp: 2026-05-16T21:17:14Z
- isolated/shared surface: not applicable
- key result lines:
  - StR count 7, missing IDs `[]`
  - US count 22, missing IDs `[]`
  - FR count 60, missing IDs `[]`
  - NFR count 15, missing IDs `[]`
  - no US shape failures after migration

## Namespace And User-Story Normative Scan

- head SHA: `a2e59a42`
- packet/topic: `31143-standards-compliance-claim-fixes`
- lane / fixture / storage format / rerank mode: docs/spec static scan
- command used:
  - `rg -n "\b(SHALL|SHALL NOT|MUST|MAY)\b" spec/usecase`
  - `rg -n "ix://agent-ix/tqvector" spec`
- timestamp: 2026-05-16T21:17:14Z
- isolated/shared surface: not applicable
- key result lines: both commands returned no matches
