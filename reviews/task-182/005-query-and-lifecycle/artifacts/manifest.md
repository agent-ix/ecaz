# Task 182 query/lifecycle manifest

- Head under validation: `d9411c692`
- Task bucket / packet: `reviews/task-182/005-query-and-lifecycle/`
- Lane: ecaz CLI and suite production-policy orchestration
- Fixture: local physical multinode driver; no measurement run
- Production policy: `training_landmarks_exact`, cap 4096, exact bounded head
  scoring, normal RaBitQ neighbor traversal
- Training relation: temporary, rows 201–400 from the declared TSV, exactly two
  columns; the corpus/query TSV itself is not committed
- Isolation: compile and focused unit parsing/expansion tests
- Timestamp: 2026-07-16 America/Los_Angeles
- Validation artifact: `validation.log`

This packet claims operator/suite reachability and structured attestation only.
Recall, latency, and storage evidence belongs to packet 006.
