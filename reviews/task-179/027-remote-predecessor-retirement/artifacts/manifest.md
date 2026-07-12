# Packet 027 — Remote predecessor retirement

Task bucket: `reviews/task-179/`; packet
`027-remote-predecessor-retirement/`.
Head SHA: `b29366f27d9663cbce3e1468688d954541d5493f`.
Lane: PG18, one coordinator plus two remote participant shells over separate
pooled loopback sessions, two physical epochs, T4a/T4b. No benchmark
measurements are claimed.

| File | Command | Key result |
|---|---|---|
| `validation.log` | commands listed in the artifact | Three exact predecessor acknowledgements, Applied covering decision, strict clippy |

This validates participant/coordinator transaction ordering on isolated
physical generations. It is not the required real three-instance or
10k/50k/100k closeout evidence.
