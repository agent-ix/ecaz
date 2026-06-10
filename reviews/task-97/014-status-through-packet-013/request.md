# Task 97 Packet 014: Status Through Packet 013

This status-only packet refreshes Task 97 after packet 013's current-head per-candidate scorer evidence.

Changed files:

- `plan/tasks/97-tq-qjl-block-kernel-family.md`
- `plan/tasks/README.md`

No code changed. No tests, GitHub CI, or AWS runs were used.

## Status Update

Task 97 is still in review. The latest packet is now `reviews/task-97/013-per-candidate-scorer-evidence/`, which adds a durable local Criterion row for the production per-candidate QJL scorer at `dim=1024,bits=4`:

- `quant/score_ip_from_parts/d1024_b4/1024`: `[874.53 ns 887.34 ns 904.33 ns]`

Packet 013 is partial F1 evidence. It does not include the old pre-`b0efa19d9` multi-accumulator comparison, so the old-vs-new F1 disposition remains open unless the reviewer accepts the current-row evidence plus packet 011's scoring-ladder evidence as sufficient for the Task 97 stop-condition / optimization decision.

The Graviton 4 runtime dispatch/vector-length/counter evidence and final closeout matrix remain pending approval.
