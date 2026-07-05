# Review Request: Task 111h Final Closeout Decision

Task: 111h

Head SHA: `537c16ca82ed7e7808d19bff91151b0acb6e6465`

Status update under review:

- `537c16ca8 task111h: mark persisted rerank sweep complete`

This packet requests final review of the Task 111h closeout. It does not add new benchmarks; it audits current code and the committed packet-local evidence, then records the final product decisions.

Primary artifact:

- `artifacts/final-closeout-audit.md`

Final decisions:

| Placement / format | Decision |
| --- | --- |
| `source/f32` | Promote as default/reference. |
| `table/*` | Abandon as a 111h product path; keep reserved for a future DDL/MVCC storage design. |
| `index/f16` | Iterate only; do not promote current layout. |
| `index/rabitq4` | Abandon current 111h candidate. |
| `index/rabitq8` | Iterate only; do not promote current layout. |
| `index/turboquant` | Abandon current 111h high-recall candidate. |

Why this is closeable:

- Every named format was implemented through the common persisted rerank payload architecture or evidence-rejected for the current product path.
- The misleading 111g query-time "table-side" interpretation is removed: `source` means existing f32 source vector, `index` means persisted compact payloads, and `table` is reserved/rejected.
- The packed scorer-width group/segment layout is implemented and covered by PG18 lifecycle fixtures.
- The warm matrix exists at 10k/50k/100k/1M through `ecaz bench suite`, with matched-recall analysis.
- The later RaBitQ8 clip finding is incorporated: clip=4 is an iteration candidate, not a default promotion.
- A local 50k cold diagnostic was added and does not change the source/f32 default decision.

Review focus:

1. Does `artifacts/final-closeout-audit.md` correctly map every Task 111h acceptance criterion and formerly open checklist row to packet-local evidence?
2. Are the promote/iterate/abandon decisions stated narrowly enough for the actual evidence, especially RaBitQ8 clip4 and the single-query cold-cache packet?
3. Is it acceptable to mark the task file and task index complete while leaving future work as new tasks rather than open 111h blockers?
