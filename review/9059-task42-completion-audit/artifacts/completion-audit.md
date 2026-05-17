# Task 42 Partial Closeout Audit

Objective: record the Task 42 smoke checkpoint so the on-disk format is
endian-explicit, version-tagged, size-stable, cross-arch checked,
cross-version registered, and covered by a narrow PG18 `pg_upgrade` smoke with
ECAZ data. Broader CI stabilization and richer live-upgrade coverage remain
deferred.

Audit timestamp: `2026-05-17T22:16:37Z`
Original audit head SHA: `788a074a4f93b5771b21df6d720db1eb857f7066`
Reviewer-feedback update: `2026-05-17`, after NFR-016 and ADR-070 landed on
`main`.

## Prompt-to-Artifact Checklist

| Requirement | Evidence | Status |
| --- | --- | --- |
| Canonical task file updated | `plan/tasks/42-on-disk-format-invariants.md` has `Status: **partial smoke checkpoint**`. | Partial closeout |
| Endian fixtures under `fixtures/on-disk/` | `artifacts/on-disk-fixture-list.txt` lists 30 fixture files covering HNSW, DiskANN, IVF, SPIRE metadata/tuple/object surfaces. `review/9055` records `make on-disk-fixtures`: 45 tests passed. | Complete |
| Byte-swapped rejection tests | `tests/on_disk_fixtures.rs` mutates version/count/dimension fields and asserts decoder rejection; `review/9055` records the passing lane. | Complete |
| Cross-arch qemu lane | `Makefile` target `endian-qemu`; `.github/workflows/ci.yml` job `On-disk fixtures under qemu s390x`; `review/9056` records the cross-build fix. CI run `26003647665`, job `76431237004`, passed before this audit. | Wired as smoke coverage; broader CI stabilization deferred |
| Static layout assertions | `tests/size_of_assertions.rs`; `Makefile` target `layout-check`; `review/9055` records `make layout-check`: 13 tests passed. | Complete |
| Version compatibility matrix | `fixtures/upgrade/matrix.csv`; `tests/upgrade_matrix.rs`; `Makefile` target `upgrade-smoke`; `review/9055` records `make upgrade-smoke`: 2 tests passed. | Covered for the current single-writable-version registry. Live upgrade rehearsal activates when a second writable version ships per NFR-016-EV-3. |
| WAL record version policy | `src/storage/wal.rs` exposes custom WAL version `1`, byte offset `0`, and a validator rejecting missing/unknown tags; docs state current writes use GenericXLog and have no extension-owned payloads. `artifacts/cargo-test-wal-policy.log`: 2 tests passed. | Policy scaffold: current production emits only GenericXLog records whose contents are covered by page-format ADRs. Per-record contracts activate when Task 37 lands custom WAL records. |
| `pg_upgrade` smoke | `Makefile` target `pg-upgrade-smoke`; `ecaz dev pg-upgrade-smoke`; `scripts/run_pg_upgrade_smoke_pg18.sh`; `review/9057` records pre/post top-2 parity, index presence, and `pg_amcheck=passed`. | Narrow PG18 same-binary smoke: HNSW-only, four-row corpus, top-2 ID equality. The original recall-floor criterion is satisfied only trivially until a richer corpus lands. |
| Make lanes | `Makefile` contains `layout-check`, `on-disk-fixtures`, `endian-qemu`, `upgrade-smoke`, and `pg-upgrade-smoke`. | Complete |
| Per-PR/scheduled gates | `.github/workflows/ci.yml` runs host on-disk fixtures, upgrade matrix, and layout assertions in `Rust Checks`; qemu job runs on schedule, workflow dispatch, and pushes to `main`. | Wired; extensive CI burn-in deferred until CI is steadier |
| Documentation | `docs/on-disk-format.md` documents endian convention, fixture process, version policy, upgrade matrix, qemu lane, PG upgrade smoke, and WAL policy. | Complete |
| Review packets | `artifacts/task42-review-packets.txt` lists Task 42 packets from 9042 through 9059. | Complete |

## Conditional Future Work

The remaining bullets in `docs/on-disk-format.md` are explicitly conditional on
new durable byte contracts or a steadier CI baseline: raw generic page fixtures,
extra rejectable swapped fields, additional SPIRE object prefixes, future
incompatible format versions, richer `pg_upgrade` recall corpora, multi-AM
`pg_upgrade` coverage, and broader CI burn-in. They are not current blockers for
the Task 42 smoke checkpoint.

## Validation Notes

- `cargo fmt --all -- --check` passed; logs contain existing stable-toolchain
  warnings about unstable rustfmt options.
- `cargo test --features bench --test wal_policy` passed (`2 passed`), with
  the existing unused-import warning in `src/am/mod.rs`.
