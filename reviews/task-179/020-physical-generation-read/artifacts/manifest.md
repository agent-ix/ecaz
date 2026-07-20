# Packet 020 — Published physical-generation read

Task bucket: `reviews/task-179/`; packet `020-physical-generation-read/`.
Head SHA: `21fd1a6f1e0a56a95b976a6f72ff99c56ba17c4b`.
Lane: PG18, single local physical owner, RaBitQ fixture, Published generation,
CustomScan exact row-tier rerank. No benchmark measurements are claimed.

| File | Command | Key result |
|---|---|---|
| `validation.log` | commands listed in the artifact | Live PG18 physical plan/result and strict clippy pass |

The fixture has one control index and one immutable physical generation relation
set. It is not a shared-table benchmark surface and is not closeout evidence for
the required multi-owner 10k/50k/100k matrix.
