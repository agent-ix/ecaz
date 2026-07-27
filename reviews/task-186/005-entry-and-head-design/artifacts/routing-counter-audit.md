# Task 186 routing-counter audit

This is a source-and-artifact audit of the historical 100k hierarchy run. It
does not invent measurements that the run did not emit.

| required bound/counter | historical value or status | provenance |
| --- | --- | --- |
| representatives scored | 256 representatives allocated per query | hierarchy implementation/packet 002 summary |
| groups opened | 16 groups cap | hierarchy implementation/packet 002 summary |
| landmarks scored | at most 512 second-level members | hierarchy implementation/packet 002 summary |
| returned seeds | 32 | packet 002 manifest and result log |
| remote requests | not emitted by the hierarchy result rows; topology only reports 2 remote owners | packet 002 `results.jsonl` |
| region assignment work | 16,384 region computations per query | reviewer source inspection; query-time rebuild |
| representative policy | arbitrary `nodes.first()` / lowest-index member | reviewer source inspection |
| builder peak memory | not measured | absent from packet 002 results |
| builder spill | not measured | absent from packet 002 results |
| build time / digest / head bytes | build time and head bytes are present; deterministic digest is present in head/storage rows | packet 002 structured results |

The historical result is consequently a latency/recall observation for one
query-time strawman, not a bounded-work proof for the entire two-level family.
Any rescreen must emit the first five counters per query (or aggregate with an
explicit denominator), plus builder peak memory and spill status, and must use
build-time region assignment with a declared representative policy.
