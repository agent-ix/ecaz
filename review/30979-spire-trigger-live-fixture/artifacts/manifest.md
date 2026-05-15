# Artifact Manifest: SPIRE Trigger Live Fixture

- head SHA at run time: `f68a8540dadb245d6d8652e02878c886941cb4b7`
- packet/topic: `30979-spire-trigger-live-fixture`
- timestamp: `2026-05-13`
- fixture: PG18 one-coordinator/one-remote
  `insert-read-after-customscan-pg18` with `--insert-mode trigger`
- status: passed after the fixture assertion was narrowed to count the
  coordinator heap through a non-PK-select predicate.

## `insert-read-trigger-v2.log`

- command:
  `target/debug/ecaz dev spire-multicluster insert-read-after-customscan-pg18 --artifact-dir review/30979-spire-trigger-live-fixture/artifacts/insert-read-trigger-v2 --run-id p12trg2 --coord-port 39202 --remote-port 39203 --insert-mode trigger --skip-install --smoke-log review/30979-spire-trigger-live-fixture/artifacts/insert-read-trigger-v2.log --log-file review/30979-spire-trigger-live-fixture/artifacts/insert-read-trigger-v2-cli.log`
- key result lines:
  - `insert_result=trigger_insert_committed`
  - `coordinator_row_count=0`
  - `remote_row=303,remote inserted via coordinator`
  - `placement_row=2,3,2`
  - `plan=Limit -> Custom Scan (EcSpireDistributedScan)`
  - `read_row=303,remote inserted via coordinator`
  - `SPIRE multicluster coordinator insert read-after-CustomScan passed`

## Notes

The earlier `30978` trigger attempts counted `WHERE id = 303`, which is now a
distributed PK-select shape. That query can see the remote row through
CustomScan, so it is not a heap-only assertion. The updated fixture uses
`WHERE id + 0 = 303` only for the coordinator heap suppression assertion; the
readback assertion still uses the normal CustomScan KNN path.

Additional uncited local logs from the earlier trigger attempt are now
published for visibility: `insert-read-trigger-cli.log`,
`insert-read-trigger.log`, `insert-read-trigger/coord-postgres.log`, and
`insert-read-trigger/remote-postgres.log`.
