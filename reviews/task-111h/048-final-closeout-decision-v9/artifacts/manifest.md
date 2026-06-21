# Artifact Manifest: Task 111h Packet 048

Head SHA: `b088c07536c2e7001ab259efc0b925c33c70471b`

Task bucket: `reviews/task-111h/`

Packet path: `reviews/task-111h/048-final-closeout-decision-v9/`

Timestamp: 2026-06-21 UTC

Lane / fixture / storage format / rerank mode: final corrected Task 111h
closeout audit over packet-local evidence from packets 043-047. This packet
does not run a new benchmark suite; it audits the corrected suites already
committed under the task bucket.

Surface isolation: this decision packet cites both isolated one-index-per-table
surfaces in packets 044-046 and the shared-table one-index-at-a-time final 1M
surface in packet 047.

## Commands

No tests or benchmarks were run for this packet. The code change under review is
the task tracker closeout:

```sh
git diff --check
git commit -m "task111h: mark corrected rerank sweep complete"
```

Benchmark evidence cited by this packet was produced by `ecaz bench suite` in:

- `reviews/task-111h/044-corrected-compact-10k-v9/artifacts/`
- `reviews/task-111h/045-corrected-compact-50k-v9/artifacts/`
- `reviews/task-111h/046-corrected-compact-100k-v9/artifacts/`
- `reviews/task-111h/047-corrected-compact-1m-locked-v9/artifacts/`

## Artifact Inventory

| Artifact | Purpose | Key result |
| --- | --- | --- |
| `final-closeout-audit-v9.md` | Requirement-by-requirement corrected closeout audit and final decision table. | All reopened follow-up gates mapped to packet-local evidence; final decision recorded for every placement/format. |
| `manifest.md` | Packet provenance. | This file. |

## Key Result Lines Cited

- Corrected 10k/50k/100k suites completed before final 1M:
  packets 044, 045, and 046 each report `completed=65 failed=0 skipped=0
  dry_run=0 missing_artifacts=0 stale=0`.
- Locked 1M suite completed:
  packet 047 reports `completed=44 failed=0 skipped=0 dry_run=0
  missing_artifacts=0 stale=0`.
- Locked 1M nprobe 64:
  source f32 recall@10 `0.9770`, mean `18.7 ms`, index `226.8 MiB`;
  index f16 recall@10 `0.9770`, mean `21.4 ms`, index `3.2 GiB`;
  RQ8 estimator c4 recall@10 `0.9730`, mean `18.1 ms`, index `1.8 GiB`;
  RQ4 c3 and TQ do not reach recall@10 `0.97`.

## Notes

- Packet 041 remains a rejected/stale closeout. Packet 048 is the corrected
  closeout request after the missing evidence was added.
- This packet intentionally does not commit generated truth caches from the
  source benchmark packets.
