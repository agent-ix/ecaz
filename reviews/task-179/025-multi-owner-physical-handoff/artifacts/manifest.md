# Packet 025 — Multi-owner physical handoff

Task bucket: `reviews/task-179/`; packet
`025-multi-owner-physical-handoff/`.
Head SHA: `c2929fcb59ddf2ef94a0737bc8a2cc42e5e38fea`.
Lane: PG18, one coordinator plus two remote participant shells over separate
pooled loopback sessions, RaBitQ, 30 synthetic rows. No benchmark measurements
are claimed.

| File | Command | Key result |
|---|---|---|
| `validation.log` | commands listed in the artifact | Three nonempty disjoint Ready physical owners, exact union, remote abort cleanup, strict clippy |

Each owner has its own generation row-tier heap, graph heap, and directory
B-tree. The fixture is isolated one-generation-per-participant storage, not a
shared-table surface. It validates transport/storage topology but is not the
required real three-instance or 10k/50k/100k closeout evidence.
