# Task 194 suite attempt record

- Head: `23a8615f3` plus the feature-gated PG18 extension installed from that head.
- Config: `reviews/task-194/002-nine-way-attribution/artifacts/suite/task194-suite.json`.
- Protocol: 100k, 50 timed iterations, 10 warmups, 200 queries, physical benchmark, stage counters enabled.
- Initial attempt: sandbox denied TCP socket creation; no measurement emitted.
- Elevated attempt: fixture startup succeeded, but the runtime loaded the prior 28-row extension, so the updated CLI rejected the emitted rows.
- After rebuilding/installing the corrected extension, the next elevated attempt stalled before topology output while the nearly-full host spent minutes in PostgreSQL checkpoints. It was terminated cleanly; no benchmark result is claimed.
- A third attempt after reclaiming 20 GB reproduced the same pre-topology futex stall after approximately 13 minutes; it was also terminated cleanly. The repeated condition is recorded for runtime follow-up, not treated as benchmark evidence.

This is an environment/provenance blocker for the required fresh A/A run, not evidence for a candidate or STOP disposition.
