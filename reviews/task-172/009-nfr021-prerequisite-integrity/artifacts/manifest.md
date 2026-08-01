# Artifact manifest

- Task bucket / packet:
  `reviews/task-172/009-nfr021-prerequisite-integrity`
- Evidence type: immutable cross-packet requirement-integrity audit; no new
  measurement
- Review base SHA: `1e70ea2842176387a05a30cdfaa8df2170044e98`
- Branch: `task-203-ec-distann-conformance`
- Created: `2026-07-29` (America/Los_Angeles)
- Lane: Task 205 physical three-owner owner-traversal control
- Fixture: PG18, three physical hash owners, coordinator outside roster
- Storage format: RaBitQ graph neighbor codes with co-placed full-precision row
  tier
- Rerank mode: default exact co-located rerank
- Isolation surface: one physical fixture per Task 205 scale/arm

## Files

| Artifact | Purpose |
| --- | --- |
| `integrity-finding.md` | Normative contradiction, measured calculation, and downstream impact |

## Immutable sources

- `spec/non-functional/NFR-021-distann-distribution-invariant.md`
- `spec/non-functional/NFR-022-distann-control-validity.md`
- `plan/tasks/172-ec-distann-real-multinode-benchmark-gate.md`
- `plan/tasks/208-ec-distann-nfr-conformance-gates.md`
- `reviews/task-204/001-arm-fidelity/request.md`
- `reviews/task-204/001-arm-fidelity/artifacts/run-final/results.jsonl`
- `reviews/task-205/003-ab/request.md`
- `reviews/task-205/003-ab/artifacts/manifest.md`
- `reviews/task-205/003-ab/artifacts/nfr-021-growth.md`
- `reviews/task-205/003-ab/artifacts/run-candidate-stage2/results.jsonl`
- `reviews/task-205/003-ab/artifacts/run-candidate-stage2/suite-manifest.json`

## Calculation inputs

```text
10k max node graph-side bytes = 25,706,496
100k max node graph-side bytes = 277,372,928
100k cluster graph-side bytes = 830,144,512
10k corpus rows = 10,000
100k corpus rows = 100,000
roster size = 3
```

## Key results

```text
raw max-node growth = 10.7899936265
normalized bytes/global-row growth = 1.0789993627
100k max-node share of cluster graph-side bytes = 0.3341260756
100k max-owner record share = 0.33432
non-owner records = 0
orphan records = 0
coordinator O(N) resident bytes = 0
```

## Commands

Read-only inspection used `sed`, `rg`, `nl`, and a direct arithmetic
calculation over the cited committed values. No test, benchmark, corpus,
PostgreSQL, cluster, or installed-`ecaz` command was run.
