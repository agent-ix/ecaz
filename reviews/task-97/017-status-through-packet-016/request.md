# Task 97 Packet 017: Status Through Packet 016

This status-only packet refreshes Task 97 after packets 015 and 016.

Changed files:

- `plan/tasks/97-tq-qjl-block-kernel-family.md`
- `plan/tasks/README.md`

No code changed. No tests, GitHub CI, or AWS runs were used.

## Status Update

Task 97 remains in review. The latest evidence is now:

- `reviews/task-97/015-old-scorer-diagnostic/`
  - answers packet 004 F1's old-loop question;
  - old pre-`b0efa19d9` loop was faster but exceeded the ADR-076 tolerance pair (`max_ulp=5920`, `285/1000` violations).
- `reviews/task-97/016-qjl32-avx2-transpose/`
  - implements the packet 011 reviewer-required qjl32 AVX2 candidate-parallel transpose;
  - local qjl32 micro-bench improves dispatch to `8.0926 us` median vs scalar `35.383 us` (`4.37x`);
  - same-head local PG18 SPIRE direct scoring remains below the Task 97 `1.5x` AVX2 stop-condition floor once scalar tails are included (`1.36x` / `1.38x`).

Packets 015-016 still need reviewer disposition. Graviton 4 runtime dispatch/vector-length/counter evidence and the final closeout matrix remain pending approval.
