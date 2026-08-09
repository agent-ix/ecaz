# Task 220 MAT-16 isolated decision

Decision: **STOP — no release matrix and no shipped-default change.**

The pre-registered candidate is recall-safe and semantically identical, but
it is a clear end-to-end regression:

| 100k arm | distinct recall | warm mean latency | owner endpoint work | owner payload SQL work | physical generation bytes | NFR |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| control | 0.9285 | 58.28 ms | 6.19 ms | 9.36 ms | 3,188,072,448 | conforming |
| packed candidate | 0.9285 | 77.65 ms | 20.96 ms | 32.06 ms | 3,188,072,448 | conforming |

The candidate is approximately 33.2% slower end to end and approximately
3.4x slower in the measured owner payload SQL stage. The generation identity,
prediction identity, materialization correctness digests, storage bytes, and
NFR-021/NFR-022 conformance are unchanged. Because the pre-registered rule
requires an end-to-end useful result before expanding to 10k/50k/100k, packet
003 is intentionally not created.

The control retains the production lazy-10 path. The packed owner payload arm
is not promoted or made a shipped default. Any revised payload representation
requires a new numbered task with a new isolated measurement.
