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

Packet 002's preregistered 100k isolated A/B gate rejects MAT-15. The
candidate's physical mean latency is 86.10 ms versus 40.60 ms for control;
p95 is 113.70 ms versus 54.30 ms; and p99 is 127.00 ms versus 57.20 ms.
Recall is 0.9295 versus 0.9275 and physical storage amplification is
1.351160 versus 1.351173, so the small recall/storage movement does not
offset the end-to-end regression.

The candidate physical prediction artifact differs from control in 2 of 200
ordered query rows. The single-surface predictions match byte-for-byte. The
physical seed digests also differ between the two fresh generations despite
the same nominal suite seed policy, so the identity difference is recorded as
a hard reproducibility gate and is not claimed as a MAT-15-caused regression.
Topology, remote-owner materialization, and malformed-payload hard-error
coverage passed for the candidate. NFR-021 was unavailable at this diagnostic
scale in both runs; this is not treated as conformance evidence.

Under packet 002's preregistered rule, a candidate that does not improve
end-to-end latency or tails does not authorize packet 003's normal 10k/50k/
100k matrix. Therefore no full-scale measurement was run, and MAT-15 is not
promoted or productionized. MAT-21 remains a separate follow-up candidate and
is not part of Task 216's completion.

Evidence is in
`reviews/task-216/002-isolated-candidate/`; the packet-local manifest below
records the two suite manifests, result hashes, and cited lines.
