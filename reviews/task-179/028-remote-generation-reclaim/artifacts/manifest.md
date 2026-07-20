# Packet 028 — Remote generation reclaim

Task bucket: `reviews/task-179/`; packet `028-remote-generation-reclaim/`.
Head SHA: `b7c123c47fde2c62829e4e34862dd3dd2503306c`.
Lane: PG18, one coordinator plus two remote participant shells over separate
pooled loopback sessions, normal zero-pin retirement. No benchmark measurements
are claimed.

| File | Command | Key result |
|---|---|---|
| `validation.log` | commands listed in the artifact | Three physical generations reclaimed, three tombstones retained, Applied decision, strict clippy |

This validates lifecycle transaction/transport ordering on isolated physical
generations. It is not the required real three-instance or 10k/50k/100k
closeout evidence.
