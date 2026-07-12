# Packet 022 — Token-fenced generation reclaim

Task bucket: `reviews/task-179/`; packet
`022-token-fenced-generation-reclaim/`.
Head SHA: `ce84a8bc270d28dc48838702e5846081ec29e56d`.
Lane: PG18, single local participant, two immutable physical generations,
normal zero-pin retirement. No benchmark measurements are claimed.

| File | Command | Key result |
|---|---|---|
| `validation.log` | commands listed in the artifact | Live PG18 retention fence/decision/reclaim/replay and strict clippy pass |

The fixture uses one logical control index with one local physical participant
per epoch. It is neither a shared-table benchmark surface nor closeout evidence
for the required multi-owner 10k/50k/100k matrix.
