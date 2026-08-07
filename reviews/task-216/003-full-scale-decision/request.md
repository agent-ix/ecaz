---
task: 216
packet: 003-full-scale-decision
agent: coder
role: coder
model: gpt-5
date: 2026-08-06
seq: 01
---

# Task 216 — full-scale decision

## Decision: STOP

Packet 002's preregistered 100k isolated A/B gate rejects MAT-15. The durable
reason is its measured addressable ceiling: coordinator decode is only 0.076
ms against a 40.60 ms control scan (0.19%), and response bytes are unchanged.
The candidate's physical mean latency is 86.10 ms versus 40.60 ms for control;
p95 is 113.70 ms versus 54.30 ms; and p99 is 127.00 ms versus 57.20 ms. That
regression is secondary evidence about the owner-SQL implementation, not a
claim that every packed-buffer implementation has that cost.

The candidate physical prediction artifact differs from control in 2 of 200
ordered query rows. The single-surface predictions match byte-for-byte. This
is explained by the lane defect that each arm rebuilt a fresh generation, so
the seed digests/indexes differ; it is not attributed to MAT-15. Future MAT-21
or successor A/B work must build once and swap only the extension binary, or
pin the drifting generation input. Candidate screening must also calculate a
maximum-addressable-win ceiling before advancing a stage-local hypothesis.
Topology and remote-owner materialization passed for the candidate. Fault
drills were explicitly skipped (`skip_fault_drills: true`); NFR-021 was
unavailable at this diagnostic scale in both runs; and malformed-payload
coverage is supplied by the packed-range unit tests, not by the benchmark.

Under packet 002's preregistered rule, a candidate that does not improve
end-to-end latency or tails does not authorize packet 003's normal 10k/50k/
100k matrix. Therefore no full-scale measurement was run, and MAT-15 is not
promoted or productionized. MAT-21 remains a separate follow-up candidate and
is not part of Task 216's completion.

Evidence is in
`reviews/task-216/002-isolated-candidate/`; the packet-local manifest below
records the two suite manifests, result hashes, and cited lines.
