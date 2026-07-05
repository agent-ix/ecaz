# Review Request: SPIRE Trigger Live Fixture

- coder: coder1
- code/evidence commit: `8ef53a636a96bf1a5ed699624fb1f7dbab900272`
- tracker row: Phase 12.9 local readiness blocker follow-up

## Scope

This packet addresses the trigger-mode blocker found in packet `30978`.

The live `insert-read-after-customscan-pg18 --insert-mode trigger` fixture was
failing because its coordinator heap assertion used `WHERE id = 303`. That is
now a distributed PK-select shape, so the assertion could observe the remote
row through CustomScan and report `coordinator_row_count=1` even when the
BEFORE trigger suppressed the local heap row.

The fixture now counts the coordinator heap with `WHERE id + 0 = 303` for that
one assertion. The normal readback assertion still uses the CustomScan KNN path
and proves the row is returned from the distributed read.

## Evidence

Packet-local metadata is in `artifacts/manifest.md`.

Key passing lines from `artifacts/insert-read-trigger-v2.log`:

- `insert_result=trigger_insert_committed`
- `coordinator_row_count=0`
- `remote_row=303,remote inserted via coordinator`
- `plan=Limit -> Custom Scan (EcSpireDistributedScan)`
- `read_row=303,remote inserted via coordinator`
- `SPIRE multicluster coordinator insert read-after-CustomScan passed`

## Validation

- `target/debug/ecaz dev spire-multicluster insert-read-after-customscan-pg18 --insert-mode trigger ...`
- `git diff --check`

## Reviewer Focus

- Confirm `id + 0 = 303` is an acceptable heap-only assertion for the fixture.
- Confirm the normal readback still proves the distributed CustomScan read path.
- Confirm this resolves the trigger-mode blocker from packet `30978`.
