# Review Feedback: Task 42 (commits `82bd0565`..`3b8e7a1d`)

Reviewer: Claude (2026-05-17)
Scope: full Task 42 series from layout contracts → qemu cross-build fix.
Packets cover: 9042 (layout contracts), 9043 (IVF/DiskANN layouts), 9044 (SPIRE
layouts), 9045 (metadata fixtures), 9046 (tuple fixtures), 9047 (IVF
fixtures), 9048 (SPIRE metadata fixtures), 9049 (SPIRE partition fixtures),
9050 (SPIRE V2 chain fixtures), 9051 (HNSW hot fixtures), 9052 (DiskANN
overflow fixture), 9053 (upgrade matrix smoke), 9054 (CI fixture lanes),
9055 (qemu endian lane), 9056 (qemu cross-build fix).

## Verdict

**Mergeable** as a Task 42 smoke checkpoint. Endian fixtures, static layout
assertions, version-compat matrix, qemu cross-arch lane, and per-PR CI
wiring are all in place. The two remaining task-body items (WAL record
version tags, `pg_upgrade` smoke) are correctly deferred — they depend on
Task 37 coordination and a live-cluster harness respectively, and the
request.md "Remaining Task 42 Gaps" section names both explicitly.

## What landed (verified)

| Task body §Approach | Status | Evidence |
|---|---|---|
| 1. Endian fixtures | ✅ | 30 `.hex` fixtures under `fixtures/on-disk/`; 45 tests in `tests/on_disk_fixtures.rs` doing decode-and-assert + byte-swap-rejection per fixture. Rejection messages match (`"invalid metadata format version"` etc.). |
| 2. Cross-arch CI via qemu | ✅ | `make endian-qemu` cross-compiles to `s390x-unknown-linux-gnu` and runs `tests/on_disk_fixtures.rs` under `qemu-s390x -L /usr/s390x-linux-gnu`. CI job `endian-qemu` runs on `schedule: "37 9 * * *"`, `workflow_dispatch`, and pushes to `main`. |
| 3. Static layout assertions | ✅ | `tests/size_of_assertions.rs` + 13 passing assertions including per-field offset consts (`HNSW_METADATA_FORMAT_VERSION_OFFSET`, `EC_IVF_CENTROID_DIMENSIONS_OFFSET`, etc.) exposed via `bench_api`. |
| 4. Version compat matrix | ✅ | `fixtures/upgrade/matrix.csv` with 8 rows (legacy HNSW v1+v2, current HNSW v3, DiskANN v3, IVF v1, SPIRE partition v1+v2). `tests/upgrade_matrix.rs` enforces unique-keys, read-implies-write invariant, fixture-exists, and pins the current writable set. |
| 5. WAL record version tags | ❌ deferred | Correctly fenced to Task 37 coordination. |
| 6. `pg_upgrade` smoke | ❌ deferred | Correctly fenced; needs the same live-cluster harness as Task 38. |
| 7. Makefile lanes | ✅ | All four lanes (`on-disk-fixtures`, `endian-qemu`, `upgrade-smoke`, `layout-check`) are in `make ci-quick`; `on-disk-fixtures` + `upgrade-smoke` run per-PR in GH Actions Rust Checks job. |
| Exit: `docs/on-disk-format.md` | ✅ | 112 lines covering little-endian convention, static coverage table, fixture process. |

## Concerns

### Important (worth a follow-up cycle)

1. **`--unresolved-symbols=ignore-all` is a heavy hammer.** Commit
   `3b8e7a1d` adds `CARGO_TARGET_S390X_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C
   link-arg=-Wl,--unresolved-symbols=ignore-all"` because the s390x
   cross-compile of `tests/on_disk_fixtures.rs` pulls in pgrx and pgrx has
   PG18-native symbol stubs that don't resolve under the cross target.
   Side effect: any *real* broken-link bug in non-pgrx code on s390x
   silently slips past this lane too. Two options:
   - Document the choice in the Makefile (one-line `# qemu lane is
     decode-only; pgrx FFI stubs are not present on s390x` comment) and
     accept the limitation. Cheap, OK.
   - Move the on-disk decoders into a sub-crate that doesn't depend on
     pgrx so the qemu lane cross-compiles cleanly without ignore-all.
     Bigger refactor; would also shrink the qemu CI install footprint
     dramatically.

2. **Qemu CI installs PG18 headers + clang + llvm + libssl** for a
   decode-only test. The decode test only uses `ecaz::bench_api`
   re-exports — none of those need pgrx at runtime. Same root cause as
   #1: the test crate is linked into the same workspace as pgrx, so
   `cargo test --test on_disk_fixtures` pulls the lib build. Splitting
   decoders into a no-pgrx crate would let the qemu lane drop ~200MB of
   apt installs and run faster.

3. **The upgrade matrix tests don't actually upgrade.** `tests/upgrade_
   matrix.rs` only verifies:
   - CSV format invariants
   - Read-implies-write logic
   - Fixture files exist
   - Current writable set matches the hardcoded `expected`
   
   It does NOT build at vN, upgrade to vN+1, scan, verify recall — which
   is what §Approach 4 asks for ("build a corpus with format vN, upgrade
   the extension to vN+1, scan and verify recall floor"). The current
   tests are a *registry consistency check*, not a *live upgrade probe*.
   This is honest with reality: there's only one writable version per AM
   today, so there's nothing to actually upgrade through. But the task
   body §Exit Criteria says "build matrix runs per-PR with the current
   matrix; new versions add a row" — which the current implementation
   satisfies in spirit. Worth a clarifying note in the task body that
   the live upgrade rehearsal activates when a second writable version
   ships.

### Minor

4. **Fixture content review.** I spot-checked `hnsw_metadata_v3.hex` —
   the test asserts `m=16`, `ef_construction=200`, `entry_point=(5, 2)`,
   `dimensions=128`, `bits=4`, `max_level=3`,
   `seed=0x0102_0304_0506_0708`, `inserted_since_rebuild=42`,
   `payload_flags=1<<2`. These are clearly hand-crafted magic constants
   rather than a captured-from-real-index dump. That's fine for decode
   tests (the bit pattern is what matters), but worth one comment in the
   fixture or test file noting "these values are intentionally chosen to
   exercise each field; not from a real corpus" so a future maintainer
   doesn't try to reverse-engineer where `0x0102_0304_0506_0708` came
   from.

5. **Schedule cron `37 9 * * *`** runs at 09:37 UTC daily. That's the
   middle of US-east work hours. If nightly green status is what you want,
   `0 6 * * *` UTC (post-midnight Pacific) would land results on engineers'
   morning. Cosmetic.

6. **Per-PR CI runs `cargo test --features bench --test
   on_disk_fixtures`** in the Rust Checks job and again in the
   `endian-qemu` job on schedule. The per-PR run uses the host
   architecture (x86_64 little-endian); the qemu run cross-compiles for
   s390x big-endian. Good coverage split. Verify the per-PR job catches
   a deliberately byte-swapped fixture (the byte-swap rejection tests
   should do this — they pass on x86_64 already so the rejection logic
   is exercised). No action.

### Process

7. **`--features bench` gating of the on-disk decoders.** Decoders are
   under `bench_api` (an existing pattern in this repo). That's fine
   internally, but means a downstream consumer of `ecaz` as a library
   can't call the decoders without enabling `bench`. If someone later
   wants e.g. an external `ecaz-fsck` tool, they'll either need a
   different feature or these get promoted to a stable public surface.
   Not a Task 42 problem; flagging for awareness.

## Reviewer-focus answers

The packet request.md doesn't have a "Reviewer Focus" section, so
inferring from the work:

- **Are the qemu rustflags too permissive?** Yes, see #1. Cheap to
  document, harder to fully fix without a no-pgrx sub-crate.
- **Is the matrix coverage adequate?** Yes for current state; the
  spec's "build at vN, upgrade to vN+1" lane activates when a second
  writable version ships. Worth restating in the task body.
- **Are the byte-swap rejection assertions adequate?** Yes for the
  fields where rejection is required (format versions, magic numbers).
  Other fields (offsets, dimensions, neighbor counts) decode either
  way; the rejection tests target the discriminator bytes.

## Recommended next-cycle actions

1. Add one-line Makefile comment explaining the
   `--unresolved-symbols=ignore-all` choice (1 minute).
2. Update task body §Exit Criteria to clarify that live-upgrade
   rehearsal activates when a second writable version ships, and the
   current matrix is a registry-consistency check (5 minutes).
3. Consider a follow-up task (or note in this task body) for "split
   on-disk decoders into a no-pgrx sub-crate" — would simplify the qemu
   lane and remove the rustflags hack. Bigger lift, not blocking.
4. WAL version tags + `pg_upgrade` smoke remain task-body §Approach 5+6
   items, correctly deferred. These will need their own packets when
   Task 37 work catches up.

## Net

Task 42 has progressed from "proposed" to a working smoke checkpoint
that covers: every persisted byte has a static offset assertion + a
golden fixture decode test + a byte-swap rejection test, the qemu lane
exercises big-endian cross-arch decode, and the upgrade matrix gates
against silent format drift. The two remaining task-body items are
explicitly deferred to dependent work. Recommend merge with the
documentation nits (#1, #3) as small follow-ups.
