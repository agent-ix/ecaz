# Review Feedback: Task 42 Completion Audit (`9c4e5bb5`)

Reviewer: Claude (2026-05-17)
Scope: the three commits since the qemu cross-build fix that the audit
relies on — `a4be6a55` (pg_upgrade smoke), `c63e8e5a` (WAL format
policy), `788a074a` (status → complete + audit doc).

## Verdict

**Audit framing is honest. Two of the three new commits cleanly close
their §Approach items; the third closes its §Approach item in a defensible
but scope-narrow way that the audit should call out explicitly.**

The "Complete" status on the task body is justified given the explicit
distinction in the audit between current Task 42 requirements and
conditional future work. The audit gets the discipline right.

## Per-commit notes

### `a4be6a55` — pg_upgrade smoke lane (§Approach 6)

A 205-line bash script (`scripts/run_pg_upgrade_smoke_pg18.sh`) plus an
`ecaz dev pg-upgrade-smoke` CLI wrapper plus a `make pg-upgrade-smoke`
lane. The script:

1. Initializes two PG18 data dirs (`old`, `new`).
2. Starts `old`, installs ECAZ, creates a 4-row `ec_hnsw` index over a
   4-dim corpus, captures pre-state (top-2 IDs, index count, heap
   count).
3. Stops `old`, runs `pg_upgrade --old-bindir=... --new-bindir=...`
   (same-binary upgrade).
4. Starts `new`, captures post-state, asserts pre==post on all three
   probes, runs `pg_amcheck`.

**Strengths**
- Clean RAII discipline: `trap cleanup EXIT` stops both clusters even
  on failure.
- Socket dir derived from `cksum $RUN_DIR` keeps multiple parallel
  invocations from colliding.
- `--retain` on pg_upgrade keeps logs for postmortem.
- Smoke-log tee via re-exec is a clean pattern.
- Pre/post top-2 ID equality is the right invariant: it proves the
  index bytes survived the upgrade *and* the post-upgrade reader can
  still rank them.

**Scope concerns**
1. **Single AM coverage.** Only `ec_hnsw` is exercised. `ec_ivf`,
   `ec_diskann`, and `ec_spire` indexes are not built or verified.
   For a smoke that's defensible — the upgrade-side bytes that matter
   are PG catalog and tablespace layout, which `pg_upgrade` handles
   identically for any AM — but the audit could be more explicit
   that the lane is HNSW-only today.

2. **"Recall floor" language in §Approach 6 is overshot.** §Validation
   says the smoke "verifies recall floor and `pg_amcheck` parity
   post-upgrade." The current 4-row corpus doesn't have a meaningful
   recall floor; what it verifies is *top-2 ID equality*, which is
   recall@2 in the trivial limit. That's the right thing to verify in
   a smoke, but the audit should flag that "recall floor" in the
   original Exit Criteria is satisfied vacuously, not by measurement.
   When a richer pg_upgrade corpus lands later, the recall-floor probe
   becomes substantive.

3. **`pg_amcheck` adds limited additional signal.** It validates the
   post-upgrade cluster's internal consistency, not that the upgrade
   migrated correctly. The pre/post equality check is what proves the
   migration; `pg_amcheck` is icing. Not a defect — just worth noting
   in the audit's evidence chain so a future reader doesn't
   over-weight it.

4. **Same-binary upgrade only.** `--old-bindir == --new-bindir`. The
   script genuinely cannot do PG18→PG19 until PG19 exists; same-binary
   is the right "infrastructure works" smoke. When PG19 ships, this
   script extends naturally by passing different bindirs.

### `c63e8e5a` — WAL format policy (§Approach 5)

- `src/storage/wal.rs`: defines
  `ECAZ_CUSTOM_WAL_RECORD_FORMAT_VERSION = 1`,
  `ECAZ_CUSTOM_WAL_RECORD_FORMAT_VERSION_OFFSET = 0`,
  `ECAZ_CUSTOM_WAL_RECORDS_ENABLED = false`, and
  `validate_custom_wal_record_format_version(record: &[u8])`.
- `tests/wal_policy.rs`: 2 tests — one pins the policy state, one
  asserts the validator rejects empty bytes and unknown version tags.
- `docs/on-disk-format.md` gains a "WAL Format Policy" section.

**Framing this honestly**

Today ECAZ does not emit extension-owned WAL record payloads. It
only emits `GenericXLog` records, which carry page-image deltas —
the bytes inside those deltas are owned by the page-format ADRs
(ADR-070 + the per-page FRs), not by a WAL-record format. So §Approach
5's literal text ("each ECAZ WAL record carries a version byte; replay
rejects unknown versions cleanly") is currently **vacuously
satisfied** — there are no extension-owned WAL records to version-tag.

What `c63e8e5a` ships is the *policy scaffold* for the day Task 37
adds extension-owned WAL records: the version constant is allocated,
the byte offset is fixed, the validator exists, and the explicit
`ECAZ_CUSTOM_WAL_RECORDS_ENABLED = false` constant is a tripwire that
flips when Task 37 lands real records.

**That's the right move.** It satisfies the spirit of §Approach 5
("when ECAZ has its own WAL records, they will be version-tagged and
the policy will be enforced") without writing a contract for code
that doesn't exist. The pairing with ADR-070 §Cross-Cutting rule 4
(WAL defaults to Option A reject-unknown) means the per-record
contract has a posture defined before Task 37 starts writing it.

**Concern**: the audit's row for WAL records says simply "Complete".
That's literally true (the policy is in place) but a future reader
could read it as "WAL versioning is being enforced in production
today", which it isn't because there's nothing to enforce. Suggested
audit row update:

> Complete (policy scaffold; current production emits only
> `GenericXLog` records whose contents are covered by the page-format
> ADRs. Per-record contracts activate when Task 37 lands custom WAL
> records.)

### `788a074a` — Mark Task 42 complete

Two-line edit: status flip in the task body + docs line rename
("Remaining gaps" → "Conditional future extensions"). Both edits
correctly reflect that what remains is gated on new artifacts
(format versions, WAL records, larger corpora) rather than on
unshipped Task 42 work.

The audit (`completion-audit.md`) is well-structured:

| Strength | Note |
|---|---|
| Maps each requirement to evidence with packet IDs | Every "Complete" row is traceable to a packet that contains the verifying logs. |
| Conditional Future Work section is explicit | Names what's deferred (raw generic page fixtures, extra rejectable fields, additional SPIRE prefixes, future incompatible versions) and *why* (they activate when new durable byte contracts ship). |
| Validation Notes acknowledge ambient warnings | The `src/am/mod.rs` unused-import warning is named as pre-existing rather than waved away. |

**One nit**: the audit's "Version compatibility matrix" row reads
"Complete for current single-writable-version matrix". That's
honest about scope but doesn't link to the policy that makes it
sufficient. After NFR-016 + ADR-070 land on `main`, this row should
add: "Live upgrade rehearsal activates when a second writable
version ships per NFR-016-EV-3." Today it reads as if the matrix
is permanently complete, which it isn't — it's complete *for now*.

## Cross-cutting

- **The audit predates NFR-016 + ADR-070** (those are on the
  `iam-spire-aws-operator-policy` branch, not yet on `main`). When
  they merge, the audit could be updated in one pass to cite the
  governing spec artifacts, but that's a follow-up — not a defect in
  this packet.
- **Coverage of `ec_ivf`, `ec_diskann`, `ec_spire` in
  pg_upgrade-smoke is a genuine future expansion.** The CLI surface
  is parametric enough that adding `--am ec_ivf` etc. is a small
  change. File as a follow-up when those AMs hit a corpus large
  enough to make the upgrade smoke load-bearing.

## Recommended next moves

These are documentation refinements, not blockers:

1. Update the audit's "WAL record version policy" row to flag
   "policy scaffold; activates when Task 37 lands custom records"
   (~1 line).
2. Update the audit's "Version compatibility matrix" row to
   reference NFR-016-EV-3 once those artifacts merge to main
   (~1 line).
3. File a small follow-up to extend `pg_upgrade-smoke` to the
   other three AMs when corpus sizing supports it. Today's HNSW-only
   coverage is appropriate for a smoke.

## Net

Task 42 is honestly complete to its scope. The pg_upgrade smoke and
WAL policy commits both close their §Approach items in defensible
ways — pg_upgrade with a narrow but correct ID-equality probe, WAL
with a policy scaffold ready for Task 37 — and the audit clearly
distinguishes current closure from conditional future expansion. The
two small audit-row clarifications above would tighten the framing
but the closure is sound as-is.

---

## Cycle 24 follow-up — clarifications landed (`25a03880`)

Coder directly addressed every clarification asked for above plus the
"CI gates deferred" framing:

- **Status**: `complete` → `partial smoke checkpoint`.
- **Audit title**: `Completion Audit` → `Partial Closeout Audit`.
- **WAL row**: now reads "Policy scaffold; current production emits
  only GenericXLog records … Per-record contracts activate when Task
  37 lands custom WAL records." Matches the framing in my cycle-23
  note.
- **pg_upgrade row**: now reads "Narrow PG18 same-binary smoke:
  HNSW-only, four-row corpus, top-2 ID equality. The original
  recall-floor criterion is satisfied only trivially until a richer
  corpus lands." Honest about the recall-floor language.
- **Matrix row**: now cross-references NFR-016-EV-3 explicitly. The
  registry-vs-live distinction is now load-bearing on policy, not
  just on the audit text.
- **§Approach 4**: cross-references NFR-016-EV-3 for the live-upgrade
  trigger.
- **§Approach 5**: cross-references ADR-070's default reject-unknown
  posture.
- **§Exit Criteria**: gains a "Deferred closeout" bullet covering CI
  stabilization, richer `pg_upgrade` recall coverage, and multi-AM
  `pg_upgrade` smoke.
- **Conditional Future Work**: names "broader CI burn-in" as a
  deferral, aligning with the user's "CI gates deferred until later"
  direction.

The cross-references are now bidirectional: the task body points at
NFR-016 + ADR-070, and those artifacts (now on `main` via
`dcf947bf`/`daa009b5`) point back at the FRs / Tasks they govern.

**Closeout state**: Task 42 is now correctly framed as a deferred-CI
partial smoke checkpoint with the policy spine in place. No further
review action is needed on Task 42 itself; remaining work (richer
corpus, multi-AM smoke, CI gate promotion, WAL ADR after Task 37,
per-FR posture declarations) is fenced as future packets.

