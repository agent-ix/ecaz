# Task 107 packet 003 - AWS benchmark evidence

Status: partial benchmark evidence packet. Coder lane.

This packet packages the Task 107 AWS benchmark evidence gathered so far and
records the corrected scope boundary.

Completed distributed evidence:

- RaBitQ 100k, `bits=4`, `local_store_count=1`, one coordinator plus two
  remote nodes.
- Production remote read path is ready and uses
  `result_source=remote_heap_candidates`.
- At nprobe 64, recall is `0.9560` for k10 and `0.9331` for k100.
- Production read profile total p50/p95 is `48/51 ms` for k10 and `49/52 ms`
  for k100.

Important limits:

- The Task 107 topology measurements here are one setting only: `bits=4`.
  They are not a 2/4/8 bit-depth sweep.
- Task 106 already owns single-node SPIRE RaBitQ AWS coverage. I am not
  starting more single-node runs from this packet.
- TurboQuant 100k and RaBitQ 100k `local_store_count=4` only produced
  single-node or failed-planning evidence; neither is a completed distributed
  result.
- The RaBitQ 1m attempt was cancelled after the scope correction, cleaned up,
  and verified to leave zero `task107_rabitq_1m_l1%` relations.

See `artifacts/manifest.md` for commands, artifact paths, key result lines,
cleanup evidence, and remaining gaps.
