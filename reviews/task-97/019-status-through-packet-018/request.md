# Task 97 Packet 019: Status Through Packet 018

This status-only packet refreshes Task 97 after packet 018.

Changed files:

- `plan/tasks/97-tq-qjl-block-kernel-family.md`
- `plan/tasks/README.md`

Code checkpoint: `1bcf9bf58800def05033d0d25577e5c0640204db`

No kernel code changed. No tests, GitHub CI, or AWS runs were used.

## Status Update

Task 97 remains in review. The latest evidence is now:

- `reviews/task-97/015-old-scorer-diagnostic/`
  - approved; closes the old-loop diagnostic and confirms the faster old
    scorer violated ADR-076 tolerance.
- `reviews/task-97/016-qjl32-avx2-transpose/`
  - approved for the AVX2 transpose slice;
  - reviewer requested one bounded octet-tail follow-up before accepting the
    stop-condition path.
- `reviews/task-97/018-qjl32-octet-batch/`
  - implements qjl32 AVX2 8-candidate octet scoring after full block32 chunks;
  - keeps true `n % 8` tails scalar;
  - adds the reviewer-requested HNSW under-octet scalar bypass after the first
    local suite pass still showed HNSW below parity;
  - local final suite reports SPIRE direct scoring `2.52x` / `2.53x` and HNSW
    `1.70 ms` kernel-on vs `1.73 ms` kernel-off.

Packet 018 still needs reviewer disposition. Graviton 4 runtime
dispatch/vector-length/counter evidence and the final closeout matrix remain
pending approval.

## Request

Please review this as a status-only packet confirming the task files now point
at packet 018 as the latest Task 97 evidence.
