# Task 189 full-scale decision evidence

| evidence | decision use |
|---|---|
| Task 183 packet 002 | same-seed exact-neighbor arm is recall-worse and latency-worse |
| Task 183 packet 006 | prior conditional STOP and no production codec change |
| Task 188 packet 002 | search/graph attribution; owner oracle gap is budget/work, not proven codec error |
| Task 188 packet 003 | full-scale search candidate evidence belongs to Task 188; this task introduces no codec candidate |

Task 188 final head: `171b84898`. Its BW4/BW8 matrix held the RaBitQ payload
format and exact head seed digest fixed. BW8 gained recall at 50k and 100k
with zero storage delta, so the measured movement is attributable to search
budget; it does not satisfy Task 189's codec entry gate.

No `ecaz bench suite` codec matrix is emitted because the candidate screen had
no eligible arm. No recall, latency, or storage numbers are fabricated for
Task 189.
