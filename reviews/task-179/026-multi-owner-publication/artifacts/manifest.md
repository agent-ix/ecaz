# Packet 026 — Multi-owner publication recovery

Task bucket: `reviews/task-179/`; packet `026-multi-owner-publication/`.
Head SHA: `e4e6d9b1ad83c38b70a2ff9d00740ac477fe4890`.
Lane: PG18, one coordinator plus two remote participant shells over separate
pooled loopback sessions, first-epoch T3/T4a publication. No benchmark
measurements are claimed.

| File | Command | Key result |
|---|---|---|
| `validation.log` | commands listed in the artifact | Three exact Published acknowledgements before active-pointer movement; strict clippy |

This is isolated physical-generation storage per participant and validates
transaction/transport ordering. It is not the required real three-instance or
10k/50k/100k closeout evidence.
