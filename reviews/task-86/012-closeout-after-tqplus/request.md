# Review Request: Closeout After TQ+ Measurement

Task: 86 - TurboVec-Derived TurboQuant Improvements

Head SHA: `c7e85e8ac542a20c3934d8c24c0a875d5a935fc2`

Packet: `reviews/task-86/012-closeout-after-tqplus/`

## Why This Packet Exists

The prior closeout accepted the SPIRE TurboQuant LUT optimization but explicitly
left TQ+ unmeasured. The user correctly rejected that as incomplete for Task 86.

Packet 011 now supplies the missing TQ+ evidence:

- IVF TurboQuant baseline vs IVF TQ+ on real10k/50k/100k;
- recall@10, p50/p95/p99 latency, and storage;
- `ecaz bench suite` configs and raw logs;
- PG18 validation logs;
- a task-local format plan for the new IVF `turboquant_tqplus` storage tag.

## Request

Please review `artifacts/completion-audit.md` against the Task 86 requirements
and packet 011 evidence.

Coder verdict: **coder-complete pending reviewer acceptance**.

## Key Point

TQ+ is no longer skipped. It is measured on real corpora against our own
TurboQuant baseline, and it improves recall and latency at every measured IVF
point with effectively unchanged index bytes per row.

The audit still avoids overclaiming: TQ+ is proven as an IVF candidate, not as
completed HNSW/DISKANN/SPIRE support.
