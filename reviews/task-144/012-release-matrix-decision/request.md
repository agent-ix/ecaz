# Task 144 Packet 012: Release Matrix Decision

## Request

Review the Task 144 release-matrix decision after completing the approved r2 10k / 50k / 100k matrix.

Decision: **do not promote**. Continue with **iterate / escalate**.

## Evidence

- 10k evidence: `reviews/task-144/009-release-matrix-10k-r2/`
- 50k evidence: `reviews/task-144/010-release-matrix-50k-r2/`
- 100k evidence: `reviews/task-144/011-release-matrix-100k-r2/`
- This packet manifest: `artifacts/manifest.md`

## Decision Basis

The 10k result was promising but did not scale:

| scale | row | nprobe | recall | candidate rows | production p50 |
| --- | --- | ---: | ---: | ---: | ---: |
| 10k | closure_e050_b8-ratio400 | 16 | 0.9900 | 2.57% | 7.670 ms |
| 10k | closure_e025_b8-adaptive | 32 | 0.9935 | 4.36% | 7.332 ms |
| 50k | fixed_b2-adaptive | 96 | 0.9900 | 35.6834% | 20.434 ms |
| 100k | closure_e050_b8-ratio200 | 96 | 0.9925 | 78.6594% | 73.132 ms |

At 50k, no row satisfies recall >= 0.99 and candidate rows <= 5%. At 100k, the only 0.99-recall family is `closure_e050_b8`, and it requires roughly 79% candidate row scan plus 568.8 MiB index size and 7.1315 mean replicas/vector.

Ratio pruning should be retired as a headline lever. Across the matrix, tight ratio values reduce scan by losing recall, while looser ratios behave like fixed/no-pruning and do not create a cheap 0.99 operating point.

## Call

Task 144’s release matrix says **do not promote closure/ratio pruning**. The next program step should treat this as an iterate/escalate outcome and avoid building Tasks 145/146 on the assumption that the 10k closure shape is scalable.
