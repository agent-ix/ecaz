# Task 205 contract packet

- Head SHA: `d27e2fdde` (the contract and implementation were bundled into the
  Task 203 docs checkpoint; attribution is recorded in
  `reviews/task-203/001-decision-reaudit/artifacts/commit-bundling-note.md`).
- Follow-up ABI wiring checkpoint: `615fd72b2d6d31d7bec9020eabcfa8fa34d39a68`.
- Task bucket/packet: `reviews/task-205/001-contract/`.
- Contract files: `spec/functional/distann/read/FR-079-distann-remote-expansion-protocol.md`
  and `spec/functional/distann/read/FR-081-distann-query-orchestration.md`.
- Timestamp: 2026-07-29 America/Los_Angeles.

The contract adds `candidate_limit` to the expansion wire/API shape and defines
owner-side score-floor, deterministic sort, and truncate semantics. FR-081
records coordinator derivation of `t = peek_worst(H_C)` and the remaining
candidate limit `l` for each round.
