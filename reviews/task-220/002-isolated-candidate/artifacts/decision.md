# Task 220 MAT-16 isolated decision

Decision: **STOP — no release matrix and no shipped-default change.**

The pre-registered candidate is recall-safe and semantically identical, but
it is a clear end-to-end regression:

| 100k arm | distinct recall | recall-step mean (200 queries) | warm physical latency mean / p95 | owner endpoint critical / work | owner payload SQL work | physical generation bytes | NFR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| control | 0.9285 | 58.28 ms | 23.00 / 28.20 ms | 6.19 / 9.75 ms | 9.36 ms | 3,188,072,448 | conforming |
| packed candidate | 0.9285 | 77.65 ms | 36.00 / 44.00 ms | 20.96 / 32.40 ms | 32.06 ms | 3,188,072,448 | conforming |

The candidate is approximately 33.2% slower on the 200-query recall-step mean
and 56.5% slower on the dedicated warm physical-latency mean. The measured
owner payload SQL stage is approximately 3.4x slower. The generation identity,
prediction identity, materialization correctness digests, storage bytes, and
NFR-021/NFR-022 conformance are unchanged. Because the pre-registered rule
requires an end-to-end useful result before expanding to 10k/50k/100k, packet
003 is intentionally not created.

The control retains the production lazy-10 path. Reviewer feedback found that
the first implementation accidentally left the packed SQL in the featureless
production path. Code checkpoint `c8b5fd9ee` restores `build_payload_sql` in
the featureless generation read, the non-profile production endpoint, and the
FR-079 endpoint (which flattens its legacy `bytea[]` result for the packed wire
ABI). The packed owner payload arm is not promoted or made a shipped default.
Any revised payload representation requires a new numbered task with a new
isolated measurement.
