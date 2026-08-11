# Task 167 checkpoint: physical benchmark suite configuration

Packet-local `SuiteConfig` is prepared for the required PG18 physical
10k/50k/100k matrix. Each step enables the physical benchmark, recall,
latency, storage, concurrent latency sweep, physical mid-insert drill,
physical concurrent insert/query drill, and the physical-vs-local insert
throughput A/B emitted by the fixture.

The configuration was syntax-validated with `jq`. It has not been executed:
this host has neither the installed `ecaz` operator binary nor
`data/staged-current`. Therefore this packet contains no fabricated benchmark
numbers and is not a closeout request.
