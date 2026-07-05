# Task 67 Packet 039 Artifact Manifest

- head SHA: `de498f9541b14dbb90332e296214d20d62e0cf80`
- task bucket: `reviews/task-67/039-closeout-audit-refresh/`
- timestamp: `2026-05-30T16:50:36Z`
- lane: Task 67 closeout audit refresh
- fixture / storage format / rerank mode: documentation audit only; no new benchmark run
- isolated one-index-per-table or shared-table surfaces: not applicable

## Purpose

This packet refreshes packet 035 after:

- packets 029, 033, and 034 received outside reviewer approval;
- packet 036's 100k scalar-vs-auto evidence was rejected as invalid;
- packet 037 fixed cloud `--simd-mode` propagation into the remote CLI process;
- packet 038 reran the corrected 100k AWS Intel scalar-vs-auto benchmark.

## Audit Commands

```bash
find reviews/task-67 -maxdepth 3 -type f | sort | tail -120
git status --short --branch
sed -n '1,260p' plan/tasks/67-rabitq-intel-avx-optimization.md
sed -n '1,220p' reviews/task-67/035-closeout-audit/feedback/2026-05-30-01-reviewer.md
sed -n '1,220p' reviews/task-67/036-scale-benchmark-100k-1m/feedback/2026-05-30-01-reviewer.md
sed -n '1,220p' reviews/task-67/038-corrected-100k-simd-benchmark/artifacts/manifest.md
find reviews/task-67/037-cloud-bench-simd-env reviews/task-67/038-corrected-100k-simd-benchmark -path '*/feedback/*' -type f -print
```

## Key Source Artifacts

- `reviews/task-67/035-closeout-audit/feedback/2026-05-30-01-reviewer.md`
  - verdict: conditionally approve as a handoff packet;
  - remaining gap: procedural review disposition for packets 029, 033, and 034.
- `reviews/task-67/029-bf16-decision/feedback/2026-05-30-01-reviewer.md`
  - verdict: approve.
- `reviews/task-67/033-measured-closeout-amendment/feedback/2026-05-30-01-reviewer.md`
  - verdict: approve.
- `reviews/task-67/034-ivf-adaptive-test-fixture/feedback/2026-05-30-01-reviewer.md`
  - verdict: approve.
- `reviews/task-67/036-scale-benchmark-100k-1m/feedback/2026-05-30-01-reviewer.md`
  - finding: packet 036's scalar-vs-auto pair was not a real comparison;
  - finding: 1m coverage was not delivered.
- `reviews/task-67/037-cloud-bench-simd-env/request.md`
  - fixes cloud runner `ECAZ_SIMD` propagation into the remote suite process.
- `reviews/task-67/038-corrected-100k-simd-benchmark/artifacts/manifest.md`
  - corrected 100k AWS Intel result after the runner fix.

## Corrected 100k Key Results

From `reviews/task-67/038-corrected-100k-simd-benchmark/artifacts/100k-comparison.tsv`:

- sidecar score p50:
  - scalar: `0.107-0.111 ms`
  - auto: `0.019-0.022 ms`
  - speedup: `4.864-5.842x`
- total bound p50:
  - scalar: `13.433-24.287 ms`
  - auto: `11.167-19.136 ms`
  - speedup: `1.197-1.271x`
- recall@10 range: `0.9470-0.9940`

Host attestation from packet 038:

- AWS profile: `10k-intel`
- DB instance: `i-02811174cc6ded75c`
- instance type: `m7i.2xlarge`
- architecture: `x86_64`
- processor info: Intel, sustained clock 3.2 GHz

## 1m Status

No 1m HNSW or DiskANN benchmark result is claimed.

Packet 036 contains staged configs and failure/blocker evidence:

- dedicated `1m` profile Terraform apply failed with a VPC quota blocker;
- fallback HNSW-on-`10k-intel` did not produce a successful benchmark result;
- DiskANN 1m was not executed successfully.

## Current Open Items

- Packet 037 has no feedback file yet.
- Packet 038 has no feedback file yet.
- Packet 039 is a new documentation-only review request.
