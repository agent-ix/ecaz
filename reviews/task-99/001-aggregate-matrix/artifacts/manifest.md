# Manifest — Task 99 packet 001 (aggregate matrix)

- Task bucket / packet: `reviews/task-99/001-aggregate-matrix/`
- Head SHA at compilation: `df2765e32` (branch `task-99-closeout`,
  rebased onto `origin/main` `239f27887`, post Task-104 merge)
- Artifact: `cross-am-quant-isa-matrix.md` — the Task 99 AC1 aggregate
  matrix with the AC5 structural-exclusion audit folded in.
- **No new measurement was run for this packet.** Every number is an
  aggregation citation; the authoritative copy of each measurement lives
  in the source packet listed next to it, with that packet's own
  manifest recording head SHA, fixture, command, and isolation mode.
- Source packets cited: task-87/{016,021,022,023,024},
  task-92/017, task-93/{002,003,004,006,007}, task-94/{001,024,025,026,
  027,028}, task-95/{001,002,003}, task-96/001, task-97/{022,026},
  task-98/003, task-101/{001,002,003,004}, task-102/{001,002},
  task-103/{001,002,003}, task-104/008.
- Spot-verification performed at compilation time (grep against source
  artifacts on this checkout): task-103/002 (88.6 ns/c, 10.4×),
  task-103/003 (81.1/80.4 ns/c), task-102/001 (235–237 ns/c, 4.5×,
  e2e table), task-104/008 (full file read).
- Timestamp: 2026-06-11 (PDT)
