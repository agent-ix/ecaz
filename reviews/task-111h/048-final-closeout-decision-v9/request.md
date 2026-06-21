# Task 111h / Packet 048 Review Request: Corrected Final Closeout Decision v9

This packet requests final review of the corrected Task 111h closeout after the
packet 041 rejection.

Head SHA under review:
`b088c07536c2e7001ab259efc0b925c33c70471b`

Status update under review:

- `b088c0753 task111h: mark corrected rerank sweep complete`

## Scope

The commit marks Task 111h complete in:

- `plan/tasks/111h-ivf-persisted-rerank-format-sweep.md`
- `plan/tasks/README.md`

It does not add code or new benchmark runs. It records the corrected closeout
decision after packets 043-047 supplied the evidence missing from packet 041.

## Evidence

Primary packet-local artifact:

- `artifacts/final-closeout-audit-v9.md`

Supporting benchmark/code evidence:

- `reviews/task-111h/043-exact-dequant-score-mode/`
- `reviews/task-111h/044-corrected-compact-10k-v9/`
- `reviews/task-111h/045-corrected-compact-50k-v9/`
- `reviews/task-111h/046-corrected-compact-100k-v9/`
- `reviews/task-111h/047-corrected-compact-1m-locked-v9/`

Validation for this packet:

- `git diff --check` passed before the task tracker commit.
- No tests or benchmarks were run for this documentation/status-only commit.

## Final Decisions

| Placement / format | Decision |
| --- | --- |
| `source/f32` | Promote as default/reference. |
| `table/*` | Reserve; not a Task 111h product path. |
| `index/f16` | Do not promote; iterate only if storage/scoring architecture changes materially. |
| `index/rabitq4` | Abandon current 111h high-recall candidate. |
| `index/rabitq8` | Iterate only; do not promote in 111h. |
| `index/turboquant` | Abandon current 111h high-recall candidate. |

## Corrected Closeout Readout

- RaBitQ-4 clip `{2,3,4}` was swept at 10k/50k/100k. Best RQ4 is clip 3, but
  it does not reach recall@10 >= 0.97 at 50k, 100k, or 1M.
- Exact-dequant was implemented and measured. It does not improve RQ8 recall in
  the corrected 1M run and does not improve TQ recall in the corrected sweeps.
- TurboQuant fidelity coverage exists through exact-dequant, but TQ still does
  not hit recall@10 >= 0.97 at 50k, 100k, or 1M.
- RQ8 clip 4 is the only compact quantized format that reaches recall@10 >=
  0.97 at 1M/w64. It does not reach recall@10 >= 0.99, and exact-dequant does
  not improve recall.
- Index f16 matches source f32 recall at 1M, but is slower and far larger in
  the measured path.

## Review Focus

1. Does `artifacts/final-closeout-audit-v9.md` correctly map every reopened
   packet 041 follow-up gate to packet-local evidence?
2. Are the final promote/reserve/iterate/abandon decisions supported by the
   corrected 10k/50k/100k and locked 1M artifacts?
3. Is it acceptable to mark Task 111h complete while moving the remaining compact
   ideas into future tasks rather than keeping 111h open?
