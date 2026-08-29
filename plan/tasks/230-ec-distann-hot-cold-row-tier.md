# Task 230: ec_distann Hot/Cold Vertical Row Tier

Status: **planning packet 001 review-closed ACCEPT (seq-03); packet 002
descriptor foundation review-closed DONE (seq-02), Graph V2 review-closed DONE
(seq-04), descriptor V4/layout identity review-closed DONE (seq-05);
generation-owned hot/cold relation creation review-closed DONE (seq-06) at
`775174659`; hot/cold handoff + Graph V2 locator review-closed DONE (seq-07) at
`885b86be0`; receipt V3 / manifest V4 sealing review-closed DONE (seq-08) at
`5214b6d98`; production read admission review-closed DONE (seq-09) at
`f4c8fcedf`; NULL cold-only/mixed reconstruction review-closed DONE (seq-10) at
`03a4015a2` — **packet 002 REVIEW-CLOSED**, all four packet-001 §7 checkpoints
satisfied; packet 003 DML/lifecycle checkpoint 1 review-closed DONE (seq-01) at
`6d439e1e3` (verdict
`reviews/task-230/003-lifecycle-and-dml/feedback/2026-08-29-01-reviewer.md`),
packet 003 still open with remote retry/fault, restart/owner-failure,
publication/recovery, and retained-generation reads still owed; topology reporting
checkpoint 2 review-closed DONE (seq-02) at `760ed15a7` (verdict
`reviews/task-230/003-lifecycle-and-dml/feedback/2026-08-29-02-reviewer.md`),
closing the seq-01 topology carry-in with version-dispatched Graph V2
diagnostics, logical hot/cold reconstruction, and explicit cold
row/orphan/byte accounting; retained-history and destructive-lifecycle
checkpoint 3 review-closed DONE (seq-03) at `7ff55c0a3` (verdict
`reviews/task-230/003-lifecycle-and-dml/feedback/2026-08-29-03-reviewer.md`),
documenting raw orphan semantics and covering hot/cold DROP, REINDEX, and
aborted-REINDEX rollback;
multinode/suite harness checkpoint 4 review-closed DONE (seq-04) at
`41a016060` (verdict
`reviews/task-230/003-lifecycle-and-dml/feedback/2026-08-29-04-reviewer.md`),
adding the explicit hot/cold fixture selection, complete cold-tier
topology/storage accounting, reuse attestation, and the post-DML diagnostic
signal clarification;
row-tier I/O attribution checkpoint 5 review-closed DONE (seq-05) at
`50701c204` (verdict
`reviews/task-230/003-lifecycle-and-dml/feedback/2026-08-29-05-reviewer.md`),
closing the seq-04 must-land item with isolated typed shapes, same-session owner
pre/post/force-flush snapshots, all six heap/TOAST/tidx deltas, and aggregate
shared-buffer hit ratio; no packet-004 measurement has started; complete
six-shape attribution checkpoint 6 reviewed **NOT DONE** (seq-06) at `428ae9b94`
(verdict
`reviews/task-230/003-lifecycle-and-dml/feedback/2026-08-29-06-reviewer.md`) —
the hot-scalar and exact-vector arms are correct and close the four-versus-six
gap, but the `cold-only` shape's physical attribution reads the **hot** relation
(`SELECT a_4 FROM <hot>`) by construction, contradicting the packet-002 seq-09
counter assertion that a cold-only projection satisfies
`HotTierRelationOpens == 0`, and not measuring what §6 predicted; **closed by**
projection-mirror fix checkpoint 7, review-closed DONE (seq-07) at `95448fa75`
(verdict
`reviews/task-230/003-lifecycle-and-dml/feedback/2026-08-29-07-reviewer.md`),
which states one rule — the attribution query mirrors the end-to-end projection —
applies it to all six shapes in both arms, makes cold-only issue no hot query at
all, drops `embedding` from the control's cold-only so the A/B compares like with
like, pins every mapping in a table-driven test, and emits
`physical_projection_rule=mirrors_end_to_end_projection` in both log lines; no
packet-004 result has been read and nothing now blocks the matrix;
hot/cold lifecycle/fault harness checkpoint 8 review-closed DONE (seq-08) at
`c4b96fe89` (verdict
`reviews/task-230/003-lifecycle-and-dml/feedback/2026-08-29-08-reviewer.md`),
adding suite-driven read/write fault switches, four-relation lifecycle
accounting, hot/cold tuple-total parity across 2PC recovery
(`cold_pair_balanced=true`), and retained-predecessor materialization in the only
window where it is observable; the three actual runtime evidence groups remain
owed; **reviewer gate-coverage correction:** the mandatory clippy command runs
from the workspace root and therefore lints only the root `ecaz` package, never
`crates/ecaz-cli`, so seq-04..seq-08's ~1000 lines of harness code were ungated
by both coder and reviewer — the reviewer ran the missing gate and confirmed
Task 230's CLI code adds no warnings (the three `suite.rs` findings all blame to
commits already in `23fb9b7ba`), but
`cargo clippy -p ecaz-cli --all-targets` must join the packet gate set with a
recorded baseline; `-D warnings` is not yet usable there because `ecaz-cloud`
and the two known `ecaz` lib errors fail first;
payload-offset/lint-gate checkpoint 9 review-closed DONE (seq-09) at
`deb245711` (verdict
`reviews/task-230/003-lifecycle-and-dml/feedback/2026-08-29-09-reviewer.md`):
the reviewer settled the arity at the production encoder
(`remote_endpoint.rs:847-855` pushes one cumulative end offset per requested
attribute, no terminal N+1), confirming the packet-002 pgrx assertions were right
and the CLI's `= 3` was wrong; both root PG18 and `ecaz-cli` all-target clippy
receipts are now recorded. runtime checkpoint 10 review-closed DONE (seq-10) at `ef0134501` (verdict
`reviews/task-230/003-lifecycle-and-dml/feedback/2026-08-29-10-reviewer.md`) —
**PACKET 003 REVIEW-CLOSED**, all three packet-001 §7 packet-003 checkpoints plus
the carried packet-002 restart/owner-failure reads satisfied. The suite-driven
four-owner read matrix passed all 25 RPC/fault cells, the three-owner
write/lifecycle matrix passed 23 scenarios and 110 records with 12
`cold_pair_balanced=true` snapshots and three successful retained mixed-tier
predecessor reads, and both external cluster directories were removed after
capture. **Only packet 004's full-scale 10k/50k/100k A/B remains — it decides
PROMOTE or STOP.**
Packet 004 preregistration reviewed **NOT DONE** (seq-01) at `5837f4bec`
(verdict
`reviews/task-230/004-full-scale-decision/feedback/2026-08-29-01-reviewer.md`).
The 20-step suite shape and all five carried entry gates are accepted and not
reopened, and no step permits a debug extension. Two threshold items must be
fixed before the run, while changing them is still preregistration: (1) the
`physical_generation_bytes` ≤ 1.35× gate applies packet-001 §6's **hot-tier**
~30% figure to a **whole-generation** ratio — at 100k/1,536 dims the predicted
delta is ≈ +190 MB on ≈ 2,498 MB, a ratio near **1.08**, so the gate allows
about 4.6× more headroom than the hypothesis and cannot fail for the intended
design; restate it per-tier, or tighten the total to ~1.15×, with the arithmetic
shown; and (2) §6 pre-committed exact-vector to *improve*, but §4 only guards it
against regression, so a flat result would silently pass — state that a
pre-committed direction contradicted by the data is reported as a falsified
prediction regardless of its guardrail. Execution awaits these two fixes.
Packet 004 preregistration seq-02 **review-closed DONE** against unchanged
config `5837f4bec` (verdict
`reviews/task-230/004-full-scale-decision/feedback/2026-08-29-02-reviewer.md`) —
**THE FULL-SCALE RUN IS AUTHORIZED.** Both seq-01 findings closed: hot main-heap
bytes gated at 1.35× emitted raw-vector bytes (8192/6144 = 1.3333, so ~1.2%
headroom — a gate that can actually fail), total generation bytes at 1.15×
matched control with the 100k arithmetic stated (≈ +190 MB on 2,498 MB ≈ 1.08×),
and new §5 requiring every §6 timing direction to be reported supported or
falsified independent of its guardrail — including the symmetric case the
reviewer did not name, where an unexpectedly *faster* cold/mixed/all result is
also falsified rather than rewritten as success. Reviewer confirmed the suite
config is byte-unchanged, so only the interpretation rules moved and they moved
before any result existed. The first authorized suite attempt produced no valid
result: its release preflight passed at `35648e467`, then the CLI rejected the
extension's 61 server attribution rows plus one client row against a stale
52-row invariant. Standalone correction `0b15cf020` updates the expected total
to 62; packet 005 is **REVIEW-CLOSED DONE** (verdict
`reviews/task-230/005-suite-runner-attribution-count/feedback/2026-08-29-01-reviewer.md`)
— the reviewer verified the chain reconciles exactly (`ALL: [Self; 61]` at
`stage_counters.rs:260`, plus one `client_result_rows`, and 51 + the ten
packet-002 seq-09 hot/cold counters = 61), that the diff is two lines touching
only a comment and a constant, and that the failed attempt's preflight passed at
the preregistered SHA with `debug_override=false`. **Packet 004's unchanged
matrix is authorized to restart from step 1 with an empty results file and a
clean fixture.** Reviewer note, not blocking: `DISTANN_WORK_ROWS` remains a
hardcoded CLI literal, so the next counter added to `ALL` will break the runner
the same way, and the failure only surfaces once a full-scale arm is already
running — have the extension declare its own metric count, or at minimum comment
the coupling at the `ALL` definition site. The minimum source-site safeguard is
implemented comment-only at `1a927c22d`; packet 006 is **REVIEW-CLOSED DONE**
(verdict
`reviews/task-230/006-counter-coupling-note/feedback/2026-08-29-01-reviewer.md`)
— the comment sits immediately above `ALL`, which is exactly the uncovered path:
forgetting `ALL` itself is already a compile error via the `[Self; 61]` length,
so the failure mode that actually occurred was updating enum and `ALL` correctly
while forgetting the CLI literal in another crate. Packet 004's next clean
restart completed one 10k row-heap control, then the first hot/cold arm failed
closed before measurement because the remote-owner placement proof used logical
`source_id`/`source` names against compact hot columns. Correction `93615542d`
selects `source_id` versus `a_2` by physical layout and joins the sampled
internal identity back to `dm` only for the raw query vector; packet 007 is
**REVIEW-OPEN** at `reviews/task-230/007-hot-cold-owner-probe/request.md`. The
partial control is discarded so the accepted rerun uses one CLI head from step
1. No complete benchmark result exists yet.
packet-001 §7
checkpoint 4 restart and owner-failure coverage explicitly moved to packet 003's
lifecycle matrix and expressly not waived; seq-07
format/clippy artifact debt closed immediately after verdict; persisted-format
implementation authorized; entry condition 3 satisfied**
(updated 2026-08-29; Task 229 is review-closed STOP; request
`reviews/task-230/001-plan/request.md` at seq-04; verdict
`reviews/task-230/001-plan/feedback/2026-08-28-03-reviewer.md`; prior verdicts
`.../2026-08-28-01-reviewer.md` and `.../2026-08-28-02-reviewer.md`;
dispositions `reviews/task-230/001-plan/artifacts/seq-01-disposition.md` and
`seq-02-disposition.md`). Frozen contract: opt-in `row_tier_layout='hot_cold'`,
implicit mandatory hot vector and identity, `PLAIN`/`fillfactor=100` hot heap
pinned by `attstorage='p'` with a hard 1,536-dimension build boundary and no
`MAIN`/`EXTERNAL` fallback, paired `row_tier_relid`/`cold_tier_relid` mutually
exclusive with the Task 229 sidecar, graph record V2 appending `cold_tid` after
the variable arrays with every V1 offset preserved, graph record as sole locator
authority, graph `is_current`/tombstone as sole visibility gate, descriptor V4 /
receipt V3 / manifest V4, and end-to-end id-only ANN retrieval as the single
preregistered primary decision shape. Carried into packet 004 preregistration:
a failure condition for the cold/mixed secondary cost gates, and shared-buffer
hit ratio as an explicit metric. Priority: P1 storage/retrieval latency.

Packet 002 checkpoint: code `ef558a669`; review request
`reviews/task-230/002-format-and-read-path/request.md`; verdict
`reviews/task-230/002-format-and-read-path/feedback/2026-08-28-01-reviewer.md`
— **NOT DONE**, three blocking items: `maximum_hot_tuple_bytes` is a
caller-supplied parameter validated only against the 8,160 ceiling with no
relation to `exact_vector_dimensions`, so a descriptor declaring 1,536
dimensions and a 1-byte hot tuple validates, encodes, and digests clean;
`identity_maximum_inline_bytes` computes the accepted `bytea(16)` 17-byte
short-varlena contribution but discards it at `row_layout.rs:303`, leaving a
type gate and a test that asserts the helper's own constant; and the
`cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
gate was not run and fails, with `options.rs:1652` introduced by this commit
(the other five errors are pre-existing on main and outside the touched files).
Non-blocking: `row_layout.rs:244-258` reports four distinct failures under one
relation-bounds message, and the indexed vector's type is unvalidated while the
identity's is. Accepted as correct: canonical partition/ordinal validation in
both directions, implicit-identity contract, no hot tombstone, Task 229
byte-for-byte compatibility, reloption mutual exclusion, and version/tier/
trailing-byte decode rejection.

Packet 002 seq-02 fix checkpoint: code `8faac4bad`; request
`reviews/task-230/002-format-and-read-path/request.md` at seq-02. The descriptor
now rejects a maximum hot-tuple bound below its checked header/bitmap,
`vec_id`, vector-varlena, identity, and hot-scalar minimum; persists and
schema-pins the 16-byte UUID versus 17-byte `bytea(16)` identity contribution;
fixes the checkpoint-owned clippy error; splits corrupt placement diagnostics;
and validates indexed-vector/generated-column identity. Focused PG18 tests and
formatting pass. The all-target PG18 clippy gate has only the five pre-existing
failures recorded in the packet manifest. Verdict
`reviews/task-230/002-format-and-read-path/feedback/2026-08-28-02-reviewer.md`
— **DONE for this slice**; all three blocking and both substantive non-blocking
seq-01 findings closed, with the reviewer independently reproducing the clippy
result. Packet 002 stays open: full format/read-path PG18 coverage is still
owed. Carried into the next slice: the preflight must set
`maximum_hot_tuple_bytes` from the catalog-exact maximal formed tuple, not from
`minimum_hot_tuple_bytes`, which omits inter-attribute alignment padding; the
indexed-vector check is name-only and should pin send/receive function identity
the way hot scalars do; and the two new `validate_row_schema` branches
(generated identity, persisted-inline-width drift) need corrupt-fixture
coverage.

Packet 002 seq-03 Graph V2 checkpoint: code `3102e28ef` plus MSRV fix
`9b13d2aca`; request `reviews/task-230/002-format-and-read-path/request.md` at
seq-03. V2 appends the six-byte cold TID after every V1 byte, uses
version-before-version-sized-length dispatch, rejects missing locators, leaves
legacy/tagged and physical-V1 bytes unchanged, and has an independent frozen
fixture. Three focused format tests and two fixture tests pass; formatting
passes; clippy has only the five known pre-existing failures. Verdict
`reviews/task-230/002-format-and-read-path/feedback/2026-08-28-03-reviewer.md`
— **NOT DONE** on two narrow test gaps, with the format work verified correct
(V1 bytes 2..62 confirmed identical between the frozen V1/V2 fixtures by hand,
version-before-length ordering pinned by an error-kind assertion, legacy branch
correctly untouched, MSRV fix correct against `clippy.toml` `msrv = "1.75"`, and
the clippy result independently reproduced). Blocking: the request's "V1 cannot
silently discard a cold locator" guarantee has no test — the guards at
`tuple.rs:172` and `tuple.rs:257` are never reached, since no test sets a valid
`cold_tid` and calls `encode_physical_v1` or the legacy `encode`; and
`decode_into_physical_version` is new public API with zero exercise, leaving the
pooled V2→V1 `cold_tid`-clearing reuse path untested on the traversal hot path.
Noted for the next slice: `distann_node_cold_tid_offset` is V2-only but
unnamed as such, `validate_physical_v2` duplicates the padding block, and
`DISTANN_NODE_HOT_COLD_FORMAT_VERSION` must become the source for
`generation_descriptor.rs`'s graph-record version rather than a second literal.

Packet 002 seq-04 test-only checkpoint: code `7b8edce68`; request at seq-04;
verdict
`reviews/task-230/002-format-and-read-path/feedback/2026-08-28-04-reviewer.md`
— **DONE for the Graph V2 slice**. Both seq-03 gaps closed with tests that fail
if the pinned code is removed: `distann_physical_node_v1_writers_reject_cold_
locator` reaches both the `tuple.rs:172` and `tuple.rs:257` guards, and
`distann_physical_node_decode_into_reuse_clears_v2_cold_locator_for_v1`
exercises `decode_into_physical_version` on both versions and pins the
`tuple.rs:445` reuse clearing. Reviewer independently ran the focused tests and
the clippy gate (same five pre-existing errors, nothing in `tuple.rs`). The next
slice — descriptor V4 / layout identity binding with only version-aware
generation callers switched to V2 — is authorized. Packet 002 remains open for
full format/read-path PG18 coverage.

Packet 002 seq-05 descriptor V4/layout-identity checkpoint: code `a1566fcb9`
plus PG18 preflight follow-up `1407d4504`; request at seq-05 — **review-closed
DONE**.
Legacy descriptor V2 and Task 229 descriptor V3 remain byte-identical; V4 binds
the row-tier layout and selects Graph V2 from the tuple-format constant;
registration V3 binds the layout digest; exact formed-tuple sizing accounts for
alignment and PLAIN `bytea(16)`'s 20 bytes; vector binary-I/O identity and
corrupt schema drift are pinned. A focused PG18 begin-build/replay test exposed
and closed a zeroed-control-metadata bug by sourcing exact dimensions from the
indexed `ecvector` catalog typmod. Relation creation, receipt/manifest version
propagation, handoff, and read paths remain for the next slice. Verdict
`reviews/task-230/002-format-and-read-path/feedback/2026-08-28-05-reviewer.md`
— **DONE**. All six carry-ins closed (seq-02: catalog-exact
`maximum_hot_tuple_bytes` now computed internally and equality-validated, vector
binary-I/O identity pinned including namespace, both drift branches covered;
seq-04: offset renamed, padding validator factored, graph-record version now
sourced from `DISTANN_NODE_FORMAT_VERSION`). The `bytea(16)` figure correctly
moved 17 → 20: under `attstorage='p'` PostgreSQL's `ATT_IS_PACKABLE` is false so
no short-varlena header applies — the reviewer's seq-02 acceptance of 17 was
wrong. Reviewer re-derived the tuple arithmetic (uuid 6204, bytea 6208, fixture
84), verified every `fixed_heap_alignment` against PG `typalign`, confirmed
`ecvector` typmod is the dimension via `lib.rs:1510`/`lib.rs:1024`, and ran the
tests and clippy independently. Carried into the relation-creation slice: the
20-byte figure assumes `attstorage='p'` on the identity column, which no DDL yet
sets or asserts (safe direction, but pin it as Task 229 did at
`ec_distann_physical_lifecycle.rs:5878`, and honour packet-001 §1's maximal
formed-tuple check); the V4 golden fixture has no cold placement so the Cold
tier discriminant is unpinned in frozen bytes; `POSTGRES_VARLENA_HEADER_BYTES`
is used as an alignment value; two redundant `align_up(..,8)` calls open the
sizing function; and `pg_test` reinstalled a debug `.so`, so reinstall release
before any packet-004 latency run.

Packet 002 seq-06 relation-creation checkpoint: code `775174659`; request at
seq-06 — verdict
`reviews/task-230/002-format-and-read-path/feedback/2026-08-28-06-reviewer.md`
— **DONE**. All four seq-05 code carry-ins closed, including the
three-times-carried requirement to pin the estimator against PostgreSQL: the
PG18 test forms a real 1,536-dimension hot tuple and asserts
`pg_column_size` equals the frozen descriptor value (6,204 bytes, re-derived by
the reviewer). Reviewer decoded the new fixture bytes by hand, grepped every
`row_tier_relid` consumer to confirm no lifecycle path misses the cold tier, and
ran the clippy gate independently. Notes: the maximal-tuple `assert_eq!` is
exact only at ≤8 hot attributes because a non-NULL row omits the null bitmap
that the estimator correctly charges — widen it when a wider hot cover is
exercised; `clippy-seq-06.log` is missing from the packet artifacts and must be
added before packet 002 closes; and bootstrap versus migration create
differently-named constraints. The generation catalog now carries a nullable unique
`cold_tier_relid`; descriptor-driven begin creates compact hot/cold heaps with
the control owner/schema/persistence/tablespace and internal dependencies,
enforces hot `fillfactor=100` plus PLAIN exact-vector/identity storage, validates
replay identity and relation existence, and propagates the cold relation through
cache invalidation, abort, retire, reclaim, and rebuild reset. The V4 fixture
now pins a Cold placement. A focused PG18 test at 1,536 dimensions proves the
formed maximal tuple equals the descriptor estimator and that abort removes the
four-relation generation atomically. Handoff, receipts/manifests, and read-path
admission remain for the next packet-002 slice.

Packet 002 seq-07 hot/cold handoff checkpoint: code `885b86be0`; request at
seq-07; verdict
`reviews/task-230/002-format-and-read-path/feedback/2026-08-28-07-reviewer.md`
— **DONE**. The unchanged handoff wire is partitioned by descriptor physical
ordinal into compact hot/cold tuples, cold is inserted first and hot second, and
both real TIDs are written into the version-dispatched Graph V2 record. Reviewer
traced that `heap_tid` and `cold_tid` are assigned from the returned insert TIDs
before `encode_physical_version`, so the pre-flight placeholder cannot survive;
the PG18 test proves it end to end by decoding the graph record and asserting
the V2 trailer equals the cold tuple's actual `ctid`. Compact schema validation
pins each physical column's attnum/name/type/typmod/collation/binary-I/O against
the frozen placement, with the correct asymmetry that a generated source column
maps to a non-generated physical column. Relation admission extends the full
guard set to the cold tier plus `fillfactor=100`, exactly two `attstorage='p'`
attributes, and a nullable-past-`vec_id` shape; the `contype <> 'n'` relaxation
is required by PG18's catalogued NOT NULL and does not weaken the legacy branch.
Legacy and Task 229 paths untouched, with the legacy staging test re-run as a
regression guard. **Required before packet 002 closes:** the seq-07 packet
carries no clippy or format artifact, the second occurrence after seq-06 note 2
was closed by `2d33f5e29`; the reviewer ran clippy independently (five
pre-existing errors, none in `handoff.rs`) but terminal output is not durable
packet evidence. Notes: a NULL indexed vector is not rejected at handoff (an
inherited gap — `prepare_legacy_entries` has it too), and `pg_test` reinstalled a
debug `.so` for the third time, so reinstall release before any packet-004
latency run.

Packet 002 seq-08 receipt V3 / manifest V4 sealing checkpoint: code
`5214b6d98`; request at seq-08; verdict
`reviews/task-230/002-format-and-read-path/feedback/2026-08-28-08-reviewer.md`
— **DONE**. Receipt V3 appends hot/cold content digests and per-tier heap bytes
to the V1 shape for an exact 383 bytes (303 + 32 + 32 + 8 + 8), leaving V1 303
and Task 229 V2 351 byte-identical; the legacy `row_tier_bytes` field becomes the
checked sum of both heaps so legacy readers still see true total row storage
while attribution is preserved in the appended fields. Manifest V4 and
fingerprint V4 bind the layout digest plus roster-ordered global hot/cold content
digests, with `validate()` a total five-tuple admitting exactly three shapes and
rejecting wrong-kind participant receipts, and `graph_record_version` now sourced
from `DISTANN_NODE_HOT_COLD_FORMAT_VERSION`. Sealing dispatches on a total
five-tuple and validates both locators and both internal `vec_id` echoes per
tier. Reviewer seq-07's NULL indexed-vector carry-in is closed on **both** the
legacy and hot/cold paths, wider than requested. Both gate artifacts are present,
closing the seq-07 debt and holding the discipline. **Reviewer methodology
disclosure:** the shared checkout was dirty with the coder's in-flight next
slice, so the reviewer's own clippy run measured HEAD plus uncommitted work and
returned zero errors; the packet's checkpoint `clippy-seq-08.log` is the correct
evidence and shows the usual five pre-existing errors, none in a touched file.
The reviewer reported that the shared dirty checkout appeared to fix all five
pre-existing lints; the exact seq-09 checkpoint rerun confirms all five remain,
so the standing caveat does not retire. Note: `DistannEpochManifestV2::version()`
uses OR over the three hot/cold fields while `validate()` requires all-or-none;
`encode()` validates first so nothing can be emitted wrong, but the two
predicates should agree.

Packet 002 seq-09 production-read checkpoint: code `f4c8fcedf`; request at
seq-09 — **review-open**. Descriptor-versioned Graph reads now admit V2 while
the three legacy tag-guarded paths remain V1; exact-vector reads map the source
attnum to the compact hot physical ordinal; and payload materialization uses
Graph V2's locator pair to fetch and validate typed hot/cold values and rebuild
the logical projection in original order. Tier access is lazy for id-only,
hot-only, cold-only, and mixed projections. Local and remote CustomScan paths
share the typed reconstruction contract and bounded latest-snapshot retry.
Focused PG18 typed and three-owner projection tests pass, including external
cold TOAST, failure cases, rescans, deepening, and forced retry; the Task 229
sidecar projection regression also passes. All three compile gates and format
pass; clippy reports only the same five pre-existing repository failures.
Reviewer seq-08's manifest-version predicate note is closed. Packet 003
lifecycle/DML remains gated on the outside verdict.

Packet 002 seq-09 production read-admission checkpoint: code `f4c8fcedf`;
request at seq-09; verdict
`reviews/task-230/002-format-and-read-path/feedback/2026-08-28-09-reviewer.md`
— **DONE**. Packet-001 acceptance criterion 2 is now satisfied numerically: the
typed PG18 test asserts, with counters reset between arms, that expansion reads
`HotTierTupleReads == 1` / `ExactVectorReads == 1` with
`ColdTierRelationOpens == 0` and `ColdTierTupleReads == 0`, that a cold-only
projection opens no hot relation, and that a hot-only projection opens no cold
relation. Ten hot/cold/exact-vector counters landed for packet-004 attribution.
The slice also found and fixed a memory-safety-class bug via its own three-owner
test — routing an owner-local hot/cold hit through the full-logical-row path read
the internal `vec_id`'s i64 bits as a UUID Datum pointer; reviewer confirmed
`generation_read.rs:79-101` leaves no residual full-row assumption and that the
three legacy tag-guarded V1 paths stay unversioned. Reviewer's seq-08
`version()`/`validate()` carry-in is closed. Clippy verified on a clean tree and
reproduces the packet log exactly, which also settles the seq-08 anomaly as
contamination — the five pre-existing lints remain open. **For packet 004:** the
counters prove tier laziness but not TOAST elimination, since
`ExactVectorReads`/`ExactVectorBytes` look identical inline or detoasted; decide
now whether attribution comes from a detoast counter or per-relation block
statistics including each tier's TOAST relation, or a win will be observed but
unexplained.

Packet 002 seq-10 closure checkpoint: code `03a4015a2`; request at seq-10 —
**review-open**. A canonical handoff row now carries NULL bits for the cold
payload and generated payload while omitting their bytes from the value stream;
the focused PG18 callback proves cold-only reconstruction returns two NULLs,
zero offsets, and no bytes, and mixed reconstruction preserves identity/vector
bytes with equal offsets at both NULL positions. Packet-001 §7 checkpoint 4's
restart and owner-failure read coverage is explicitly moved into packet 003's
lifecycle matrix and must be dynamically exercised there. Packet-004 TOAST
attribution is preregistered as per-shape pre/post `pg_statio_all_tables`
deltas for heap, TOAST heap, and TOAST index reads/hits on isolated arm
surfaces; shared-buffer hit ratio is computed from the same deltas. If seq-10
is DONE, packet 002 closes and packet 003 is authorized.

Packet 002 seq-10 closure checkpoint: code `03a4015a2`; request at seq-10;
verdict
`reviews/task-230/002-format-and-read-path/feedback/2026-08-29-01-reviewer.md`
— **DONE, packet 002 review-closed**. All three seq-09 closing conditions met.
NULL coverage travels the real wire/handoff/seal/read path (built by marking the
wire bitmap rather than mutating a published row, which would have invalidated
Graph V2 CTIDs) and asserts the check that matters: `offsets[1] == offsets[0]`
and `offsets[3] == offsets[2]` prove a NULL never advances the value cursor,
with the terminal offset pinned to `values.len()`. Restart and owner-failure
coverage are reconciled into packet 003 in writing and expressly not waived.
Packet-004 TOAST attribution is decided: pre/post `pg_statio_all_tables` deltas
per isolated arm and query shape covering heap, TOAST heap, and TOAST index
reads/hits, with shared-buffer hit ratio derived from the same deltas — one
measurement serving both needs and no production detoast instrumentation.
Harness note: take the post-reading after `pg_stat_force_next_flush()` from the
same session that ran the queries. Standing requirement from packet 003 onward:
every checkpoint carries both `clippy` and `format-check` logs with manifest
entries, including test-only commits, since `--all-targets` lints test code;
`clippy-seq-10.log` is still owed so packet 002 closes with a complete record.

Packet 003 seq-01 hot/cold DML and reclaim checkpoint: code `6d439e1e3`;
request `reviews/task-230/003-lifecycle-and-dml/request.md` seq-01; verdict
`reviews/task-230/003-lifecycle-and-dml/feedback/2026-08-29-01-reviewer.md`
— **DONE** for the claimed scope. Every `encode_physical_v1`/`decode_physical_v1`
call in `physical_dml.rs` is converted to version-dispatched form with zero
leftovers (reviewer-verified by grep), so backlink read/modify/write,
replacement, and tombstone all carry the V2 cold locator through
decode/mutate/re-encode. The guard required in packet-001 seq-02 —
`validate_physical_v1` rejecting a present `cold_tid` — now provides real
defence: a missed re-encode site would fail loudly rather than silently publish a
locator-less record. Insert is cold-then-hot-then-graph with a fail-closed check
for a missing cold relation; delete touches only the graph tombstone; replacement
appends a new pair plus graph version retaining predecessors. Forwarded owner
payloads deliberately keep the full logical source schema on the wire so a layout
change never becomes a wire change. Both gate logs present from checkpoint 1,
honouring the standing requirement set at packet-002 closure; reviewer's clippy
run on a clean tree reproduces the packet log; the PG18 run covers six callbacks
in one invocation including all four prior packet-002 hot/cold tests as
regression guards. **Added to the packet-003 owed ledger:**
`diagnose_physical_generation` (`handoff.rs:1992`), reached from
`build_topology_row`, still hard-codes `decode_physical_v1` and has no cold-tier
accounting — a hot/cold generation therefore cannot produce a topology report
(fail-closed, not wrong). Packet-001 §3 names topology among the surfaces that
must verify descriptor/receipt/manifest links, and packet 004 will read topology
and storage numbers per arm.

Packet 003 seq-02 hot/cold topology checkpoint: code `760ed15a7`; request at
seq-02; verdict
`reviews/task-230/003-lifecycle-and-dml/feedback/2026-08-29-02-reviewer.md`
— **DONE**, closing the topology carry-in the reviewer added at seq-01. The
diagnostic now decodes at `descriptor.graph_record_version`, opens the cold
relation under the same `AccessShareLock` set, and dispatches reconstruction on a
total five-tuple with a pre-loop shape check. Reconstruction calls the **same**
`fetch_hot_cold_row` the seal path uses rather than a parallel topology-local
implementation, so seal, read, and topology now share one reconstruction
definition. Three optional columns append (19 → 22) with legacy/sidecar
generations reporting NULL, keeping "no cold tier" distinct from "empty cold
tier". Strongest evidence in the checkpoint: the PG18 test asserts
`row_tier_digest` equals the digest frozen in the manifest at seal time, proving
that recombining two compact tuples yields a byte-identical logical digest to the
pre-split single-row form. **Reviewer note carried into retained-predecessor
work and packet 004:** `cold_orphan_row_count` mirrors the existing hot-side
`row_count.saturating_sub(colocated_row_count)` definition, so after a
same-identity replacement — which packet-001 §5 requires to retain predecessor
tuples until reclaim — both orphan columns report non-zero for a perfectly
healthy generation. Document that semantic where the columns are defined, or
exclude retained predecessors, before packet 004 reads these numbers per arm.

Packet 003 seq-03 retained-history and destructive-lifecycle checkpoint: code
`7ff55c0a3`; request at seq-03; verdict
`reviews/task-230/003-lifecycle-and-dml/feedback/2026-08-29-03-reviewer.md`
— **DONE**. The reviewer's seq-02 orphan-semantics note is closed by
documentation (the stated preference) and goes further by pinning the behaviour:
the DML callback proves a healthy same-identity replacement reports exactly one
retained hot tuple and one retained cold tuple, so a later "fix" that excluded
retained predecessors would fail the test rather than change the semantic
silently. The destructive matrix is complete — `DROP INDEX` and successful
`REINDEX` each remove all four relations plus the catalog row, `REINDEX` mints a
fresh control UUID, and the aborted-`REINDEX` case restores all four relations,
the original UUID, **and the cataloged cold relation binding**, which is the
assertion that would catch a dangling `cold_tier_relid`. **Note carried into
packet 004:** the new orphan sentence and the pre-existing "equals the Ready
receipt exactly when the generation is clean" sentence are in tension, and
packet-001 §5 fixes receipt digests as immutable initial content, so on a
generation that has taken post-Ready DML neither the orphan columns nor digest
equality is by itself a corruption signal. State which signal an operator should
trust before packet 004 reads these columns per DML arm.

Packet 003 seq-04 multinode and suite harness checkpoint: code `41a016060`;
request at seq-04; verdict
`reviews/task-230/003-lifecycle-and-dml/feedback/2026-08-29-04-reviewer.md`
— **DONE** for the claimed scope. Follows CLAUDE.md's benchmark rule exactly:
the suite runner is extended in `ecaz-cli` as its own commit *before* the packet
consumes it, with typed `hot_cold_row_tier` / `hot_payload_attnums` step options,
mutual exclusion against `covering_payload_attnums`, and a negative test — no
packet-local sweeper and no raw-argument escape hatch. The highest-value guard is
the fixture-reuse attestation, which makes it impossible to silently reuse a
row-heap control as the candidate; Task 229 lost three of its four gates to
exactly that class of arm contamination. Cold heap bytes now flow into per-owner
and aggregate storage, and the topology reader rejects incomplete cold-column
triples. The reviewer's seq-03 fault-signal carry-in is closed by naming graph
current/tombstone state plus vec-id/schema-checked locator reconstruction as
authoritative after post-Ready DML, with receipt digests and raw orphan counts
demoted to historical attribution. Also confirmed:
`validate_distann_suite_run_dir` already enforces that a step's `run_dir` is a
strict child of `ECAZ_CLUSTER_ROOT`, codifying the disk-fill rule for the many
arms packet 004 will run. **Time-critical gap:** the only
`pg_statio_all_tables` use in the CLI is
`distann_multicluster.rs:9632-9646`, reading `heap_blks_read`/`hit` only as a
cache-residency proxy — there is no `toast_blks_read`/`hit` or
`tidx_blks_read`/`hit` and no per-arm, per-query-shape pre/post delta capture.
Those are the columns that attribute the mechanism (control doing TOAST fetches,
candidate not). Land them as their own `ecaz-cli` commit before the matrix runs;
a matrix run without them has to be run again.

Packet 003 seq-05 row-tier I/O attribution checkpoint: code `50701c204`;
request at seq-05; verdict
`reviews/task-230/003-lifecycle-and-dml/feedback/2026-08-29-05-reviewer.md`
— **DONE**, closing the reviewer's seq-04 must-land item in full. All six
counters (`heap`/`toast`/`tidx` reads and hits) are captured per relation as
pre/post deltas on every owner, in the same backend session that performed the
reads. Three details credited: `pg_stat_clear_snapshot()` alongside
`pg_stat_force_next_flush()` — the reviewer had asked only for the flush, but
`pg_statio_*` is snapshot-stable within a transaction, so without the clear every
delta returns zero and looks like a clean result; `checked_sub` failing closed on
counter reset rather than saturating to zero, which would otherwise be
indistinguishable from a perfect cache hit and read as evidence *for* the
candidate; and one query shape per fresh fixture with reuse rejected, since
`pg_statio` is cumulative and cache state is sticky. Emitted lines carry their own
provenance markers so a packet-004 reviewer can verify methodology from the
artifact rather than manifest prose. The Task 230-only external TOAST fixture
avoids inheriting `materialization_correctness`'s variant matrix — the exact
confound that cost Task 229 three of its four gates — and the schema requires it
for cold/mixed/select-all while correctly not requiring it for id-only.
**Carried into packet-004 preregistration:** `DistannTask230IoQueryShape` is
`IdOnly | ColdOnly | Mixed | SelectAll`, but packet-001 §6 pre-committed
directions for six shapes; hot-scalar and exact-vector have no attribution arm.
The traversal-side mechanism evidence is already carried by id-only, but the
exact-vector *materialization* prediction has no arm. Either add the two shapes
or narrow §6's prediction list before any full-scale result is read — freezing
thresholds for four while predicting six leaves two predictions unfalsifiable.

Packet 003 seq-06 complete six-shape attribution checkpoint: code `428ae9b94`;
review request at seq-06; verdict
`reviews/task-230/003-lifecycle-and-dml/feedback/2026-08-29-06-reviewer.md`
— **NOT DONE**. Adds separately labelled hot-scalar and
exact-vector physical/end-to-end attribution arms, with exact-vector directly
measuring the materialization side of the PLAIN-inline mechanism. Both are
hot-only and do not require the Task 230 external TOAST fixture; the strict
fixture gate remains for cold-only, mixed, and select-all. No packet-004 result
has been read and no threshold has been frozen. Format and four focused CLI
tests pass; the mandatory clippy gate contains only the same five pre-existing
findings. Blocking finding: the pre-existing cold-only direct attribution also
read hot `a_4`, so the six shapes did not follow one stated rule.

Packet 003 seq-07 projection-mirror fix checkpoint: code `95448fa75`; review
request at seq-07, review-open. Physical attribution now explicitly mirrors the
selected end-to-end result projection. Cold-only issues only cold `a_5` in the
candidate and only `payload_note` in the control; id-only/hot-scalar use
`id`/`a_1`, exact-vector uses `embedding`/`a_4`, mixed uses `id, payload_note` /
hot `a_1` plus cold `a_5`, and select-all uses the full row / both tiers. The
rule is emitted in every result line and a table-driven test pins both layouts
for all six shapes. Format and five focused CLI tests pass; mandatory clippy
contains only the same five pre-existing findings.

Packet 003 seq-06 six-shape I/O checkpoint: code `428ae9b94`; request at seq-06;
verdict
`reviews/task-230/003-lifecycle-and-dml/feedback/2026-08-29-06-reviewer.md`
— **NOT DONE**, one blocking item that must be fixed before the packet-004
matrix runs. The two new shapes are correct: `HotScalar` projects `a_1`/`id`
separately from the primary id-only arm, and `ExactVector` projects
`a_4`/`embedding`, supplying the materialization-side mechanism evidence that
id-only cannot carry; both are hot-only and correctly exempt from the external
TOAST fixture that stays required for cold/mixed/select-all. Adding the shapes
rather than narrowing §6's predictions was the right choice and was made before
any threshold was frozen. **Blocking:** the `cold-only` shape's physical
attribution issues `SELECT a_4 FROM <hot_relation>` alongside
`SELECT a_5 FROM <cold_relation>` (`distann_multicluster.rs:10108-10148`), so a
shape labelled cold-only reads the hot tier by construction. That contradicts the
packet-002 seq-09 counter assertion (`HotTierRelationOpens == 0`,
`HotTierTupleReads == 0` for cold-only), does not measure §6's cold-scalar
prediction, and follows no stated rule — `a_4` appears in the physical arm for
`IdOnly`, `ColdOnly`, and `Mixed`, none of which project the vector end-to-end,
yet `ColdOnly` alone omits `a_1`. State whether the physical query mirrors the
shape's projection or the engine's actual touches, then apply it to all six.
**Reviewer note:** this mapping is pre-existing from `50701c204`, which the
reviewer accepted at seq-05 after verifying counter subtraction, flush/clear
pairing, fail-closed reset, and fixture isolation, but without reading the SQL
each shape issues — the miss is the reviewer's, not a regression here.

Packet 003 seq-07 projection-mirrored I/O attribution checkpoint: code
`95448fa75`; request at seq-07; verdict
`reviews/task-230/003-lifecycle-and-dml/feedback/2026-08-29-07-reviewer.md`
— **DONE**, closing the seq-06 blocking item with the option the reviewer
recommended. `task230_local_io_projections` returns
`(Option<hot_or_row>, Option<cold>)` so "this shape does not touch that tier" is
representable in the type rather than worked around: cold-only issues no
hot-relation query at all, which makes the packet-002 seq-09 counter assertion
and the packet-004 I/O table agree instead of contradict. The row-heap control
also dropped `embedding` from cold-only — fixing only the candidate would have
left the A/B comparing a cold-payload read against a vector-plus-payload read, a
worse error because it would have looked corrected. A table-driven test pins the
row/hot/cold projections for all six shapes, and both log lines now carry
`physical_projection_rule=mirrors_end_to_end_projection`, so the rule that was
previously invisible in the output — which is why it survived the reviewer's
seq-05 pass — is now auditable from the artifact. **Carried into packet-004
reporting:** `id-only` and `hot-scalar` are now the same projection (`a_1`/`id`)
because `id` is the only additional hot scalar the fixture schema admits, so §6's
six predictions resolve to five distinct measurements; report the two rows as one
measurement under two labels rather than letting equal numbers read as
independent corroboration.

Packet 003 seq-08 hot/cold lifecycle and fault-matrix harness checkpoint: code
`c4b96fe89`; request at seq-08, review-open. Adds typed suite fields for the
Task 234 read-RPC and Task 235 write/lifecycle matrices before this packet
consumes them. Hot/cold lifecycle states now require four live relations; every
write-fault snapshot requires the compact hot identity, a present cold tier,
and equal total hot/cold tuple counts. Retained predecessor coverage now
materializes projection attnums 1 and 3 on every owner after retirement and
before reclaim, forcing mixed hot/cold reconstruction from a `Retired`
generation. Format and six focused CLI tests pass; mandatory clippy contains
only the same five pre-existing findings. No runtime matrix has run yet.

Packet 003 seq-08 lifecycle and fault-matrix harness checkpoint: code
`c4b96fe89`; request at seq-08; verdict
`reviews/task-230/003-lifecycle-and-dml/feedback/2026-08-29-08-reviewer.md`
— **DONE**. The cross-tier invariant is the right one: hot and cold tuple totals
must stay equal before, during, and after 2PC recovery, which is the
characteristic failure mode of a two-relation write path and is caught by nothing
else in the suite. The retained-generation read is correctly placed after
successor publication and predecessor retirement but before reclaim — the only
window where retained predecessors are observable — and requesting attnums 1 and
3 forces genuine hot-plus-cold reconstruction. **To reconcile before packet 004
cites either:** the CLI asserts `array_length(payload_offsets, 1) = 3` for a
two-attnum request, while the packet-002 seq-10 pgrx tests assert two offsets for
two attnums and four for four. Both suites are green, so they cannot be checking
the same function under the same convention; state which arity is authoritative
for `ec_distann_materialize_physical_row_payloads` and make both assert it the
same way, since an offsets off-by-one produces correct-looking rows with shifted
attribute boundaries.

Packet 003 seq-09 payload-offset contract and CLI lint gate: code `deb245711`;
request at seq-09; verdict
`reviews/task-230/003-lifecycle-and-dml/feedback/2026-08-29-09-reviewer.md`
— **DONE**, closing both seq-08 carry-ins. The reviewer resolved the offsets
arity at the encoder rather than by arbitrating between two test suites:
`remote_endpoint.rs:847-855` builds `payload_offsets` with
`capacity(payload_values.len())` and pushes one cumulative end offset per value,
so the contract is N offsets for N attnums. The whole encoding is coherent once
`payload_values` is seen to carry one entry per requested attribute with NULLs
contributing zero bytes — which is why seq-10's `offsets[1] == offsets[0]` holds
and why its length is 4 rather than 2. The fix was a one-character test-assertion
correction; production behaviour was always right, and the wrong arity would have
failed the retained-read arm at runtime with a misleading `offsets_valid=false`.
`cargo clippy -p ecaz-cli --all-targets` is now a packet receipt, exiting 0
against a 77/78-warning pre-existing baseline with nothing at the seq-09 change.
**Suggestion carried:** record the baseline *count* in the manifest and compare
each run — a 77-warning list is too large to eyeball, so a future checkpoint's new
warning would go unnoticed; a count comparison turns the receipt from a filed
artifact into a check that can fail.

Packet 003 seq-10 runtime fault and lifecycle evidence: code `ef0134501`;
request at seq-10; verdict
`reviews/task-230/003-lifecycle-and-dml/feedback/2026-08-29-10-reviewer.md`
— **DONE, packet 003 review-closed**. Preregistration was done in the form that
means something: `ef0134501` contains one file and nothing else — the 51-line
suite config — pushed before any result existed. The reviewer verified every
numeric claim against the artifacts rather than the request prose: 71 result
rows, 25 read-matrix `pass=true` cells across five fault modes
(`connection_reset`, `local_query_cancel`, `local_statement_timeout`,
`remote_backend_termination`, `remote_statement_timeout`), 12
`cold_pair_balanced=true` write snapshots, three retained-generation reads with
`state=Retired tuple_payload_missing=false offsets_valid=true payload_bytes=156`
one per owner, and both release preflights `unanimous=true
extension_build_profile=release` at the preregistration SHA. Both arms used
distinct `--run-dir` values under `~/.ecaz/clusters/` and the log records
`removed run_dir ... after durable artifact capture` for each. **Must change for
packet 004:** both steps pass `--allow-debug-extension` (`debug_override=true`).
It did no harm here — the build genuinely was `--release` — but it disables the
one guard that catches a debug `.so`, and `pg_test` has silently reinstalled a
debug build four times on this task. Packet 004 must run without the flag so the
release preflight is load-bearing. **Framing note:** this evidence ran at
`--rows 2000 --dim 16`, which is right for a fault matrix and proves only
dimension-independent invariants; at dim 16 the hot tuple is inline regardless of
`attstorage='p'`, so packet 003 is *not* evidence about the storage regime — that
lives in packet-002 seq-06's maximal-formed-tuple test.

Packet 004 seq-01 decision preregistration: config `5837f4bec`; request at
seq-01; verdict
`reviews/task-230/004-full-scale-decision/feedback/2026-08-29-01-reviewer.md`
— **NOT DONE**, two threshold fixes before the run. Accepted and not reopened:
the 20-step shape (verified — config checked in, dry-run expands 20 steps,
`suite-audit.log` reports "audit passed: 20 steps", every `run_dir` a distinct
child of `~/.ecaz/clusters`, zero `allow_debug_extension` occurrences), the
conjunctive two-pair 100k mean rule with no tie-break, the combined
percent-and-absolute form throughout, §3's "a gate is not attributed if the two
arms do not have the same fixture schema and payload" (Task 229's confound
written into policy), and §1's requirement that physical prediction files be
**byte-identical** within every primary pair — the strongest available parity
check for a change that moves bytes without touching the graph or vectors. All
five reviewer carry-ins are honored, several better than asked. **Blocking:**
(1) the 1.35× storage gate mixes denominators — 30% of the hot tier is ≈ 8% of
the generation, so the gate cannot fail for the intended design; (2) no
disposition is stated for a §6 direction the data contradicts, so a flat
exact-vector result would pass every gate while falsifying the clearest timing
test of the mechanism.

Packet 004 seq-02 corrected decision policy: unchanged config `5837f4bec`;
request at seq-02; verdict
`reviews/task-230/004-full-scale-decision/feedback/2026-08-29-02-reviewer.md`
— **DONE, full-scale run authorized.** The reviewer verified the commit touches
only `request.md`, the packet manifest, and bookkeeping — nothing under
`crates/ecaz-cli/suites/` — so the audited 20-step matrix stands and only the
decision math changed, before any result existed. Storage is now two gates with
two denominators: per-tier hot main-heap ≤ 1.35× `raw_vector_bytes`, which sits
~1.2% above the derived 8192/6144 = 1.3333 and can therefore fail if PLAIN
amplification exceeds the prediction; and whole-generation ≤ 1.15× control
against a stated ≈ 1.08× expectation. New §5 makes prediction accounting explicit
in both directions and keeps it separate from the gates: "Prediction
classification is part of the decision record; the numeric gates above still
determine PROMOTE versus STOP." **Standard fixed for reading the results**, per
the verdict: entry gates actually ran (release SHA, `debug_override=false`,
clippy baseline not grown); byte-identical prediction files within every primary
pair; both 100k mean wins conjunctively at ≥5.0% and ≥0.50 ms with no tie-break;
zero cold-tier accesses on id-only/exact-vector and zero hot-tier on cold-only;
both storage gates against their own denominators; the §5 table honest in both
directions; and run dirs distinct, under `~/.ecaz/clusters`, removed after
capture.

Packet 005 suite-runner attribution count: code `0b15cf020`; request at seq-01;
verdict
`reviews/task-230/005-suite-runner-attribution-count/feedback/2026-08-29-01-reviewer.md`
— **DONE**. The 62-row contract reconciles exactly and the reviewer checked every
number in the chain rather than the claim: `DistannMaterializationWork::ALL` is
`[Self; 61]`, the latency child appends one `client_result_rows`, and the old 52
was 51 + 1, with Task 230's ten packet-002 seq-09 counters accounting for the
difference. The diff is two lines — a stale comment and the constant — touching
no suite config, measurement option, threshold, or interpretation logic. The
failure itself did the right thing: a runner invariant refused to attribute
latency against an unrecognised row shape, in step 1 of 20, before any
measurement existed. The disposition is right on every point — invalid, not
interpreted, **not resumed**, restart from step 1 with a new empty results file
and clean run directory; resuming would have left a run whose first arm used a
different CLI than the rest. The failed-arm summary also confirms the entry gate
works in the real run: `extension_build_profile=release`, `debug_override=false`,
`unanimous=true` at the preregistered SHA, and `source_config_sha256=e141ac65…`
captured so the restart can be proved to use the same frozen config. **On the
rerun the reviewer will additionally check** that the config SHA-256 matches
`e141ac65…` and that the results file is genuinely new and begins at step 1.

Packet 006 work-count coupling note: code `1a927c22d`; request at seq-01;
verdict
`reviews/task-230/006-counter-coupling-note/feedback/2026-08-29-01-reviewer.md`
— **DONE**. Comment-only, and placed on precisely the path that was uncovered:
adding an enum variant and forgetting `ALL` is already caught by the compiler
through the array length, so the real gap was updating both correctly and
forgetting `DISTANN_WORK_ROWS` in another crate. The note sits immediately above
`ALL` — the second edit a counter author must make — and names the exact file and
the exact `server count + client_result_rows` relationship rather than gesturing
at a coupling. The reviewer ran no gates and expected none: clippy does not lint
comment content and the lines are inside the format width, so skipping tests here
is a correct application of the repository's own "static review is sufficient"
policy rather than a shortcut. The stronger option — having the extension declare
its own metric count so the CLI validates against the server instead of a literal
— remains available if this bites again.

Program ledger: `plan/design/ec-distann-recall-latency-roadmap.md`, candidate
ARCH-16. This task evaluates a vertical layout independently of Task 229's
covering sidecar; it is not skipped if Task 229 wins.

## Why

The frozen generation row tier mixes attributes with very different read
behavior: the exact vector is read by ANN expansion/rerank, small identifiers
and scalars dominate common result projections, and large arrays or toasted
payloads are normally cold. Keeping all of them in one PostgreSQL heap tuple
couples hot retrieval to tuple deformation and TOAST state for unrelated
columns.

A vertical split can keep the exact vector and a deliberately small scalar
cover in a dense hot tier while retaining arbitrary PostgreSQL values in a cold
tier. Unlike Task 229, this changes the authoritative row-tier representation
and can benefit both graph expansion and result materialization.

## Goal

Implement and benchmark an opt-in hot/cold generation layout that reads the
exact vector and selected scalar attributes without touching cold payload
storage, while reconstructing arbitrary SQL rows exactly when cold attributes
are required.

## Entry conditions

1. Tasks 222--224 are review-closed and define the frozen unstacked control.
2. Task 229 has completed its full matrix, but its sidecar is not enabled in
   this candidate arm.
3. The hot/cold attribute contract and expected storage amplification are
   reviewed before the persisted format is written.

## Required implementation

### P1 — Vertical format

- Define a generation-owned hot tier containing vec_id, the full-precision
  exact vector, implicit source identity, and an explicitly bounded optional
  scalar cover. Graph current/tombstone state is the sole visibility gate.
- Define a cold tier containing every remaining source attribute with enough
  attnum/type metadata to reconstruct the original row descriptor exactly.
- Store each logical attribute in exactly one authoritative tier. Do not keep a
  second exact vector or duplicate the full source row merely to simplify
  fallback.
- Give graph records a versioned locator that resolves the hot row and its cold
  counterpart without interpreting a cross-node CTID.

### P2 — Read and mutation paths

- Expansion/rerank reads only the hot exact-vector surface.
- Result materialization reads only the tiers required by Task 222's mask and
  merges values in original attnum order.
- Task 167 insert/replacement/delete operations update hot, cold, graph, and
  directory state atomically under the existing intent/retry rules.
- Bind both tier digests and bytes into receipts, manifest/fingerprint,
  topology, storage accounting, retirement, and reclaim.
- Preserve rebuild rollback and a readable fallback for older row-heap
  generations.

### P3 — Evidence

- Cover id-only, hot-scalar, exact-vector projection, cold-scalar, mixed
  hot/cold, `SELECT *`, NULL, external TOAST, qual-only, deepening, rescan,
  mutation, restart, retirement, and owner failure.
- Run isolated row-heap versus hot/cold A/B at 10k, 50k, and 100k using
  `ecaz bench suite`. Do not combine Task 229's sidecar or Task 231's fixed
  graph blocks with this candidate.
- Compare arms at matched fixture position, or use a preregistered
  counterbalanced envelope that separates position/warmth from the candidate.
  Never compare a fresh-build control only against a reused candidate.
- Report exact-vector reads, hot/cold tuple and block reads, detoast/send work,
  payload bytes, graph expansion stages, end-to-end latency/tails, recall,
  build/DML cost, per-tier bytes, and conformance.

## Decision rule

An implemented prototype and full 10k/50k/100k matrix are mandatory. Close
PROMOTE or STOP based on measured end-to-end retrieval, storage, construction,
and mutation effects. Continue to Tasks 231 and 232 regardless of outcome.

## Non-goals

- A fully columnar representation; Task 232 owns that design.
- Fixed-stride graph-node extents; Task 231 owns them.
- Replicating hot rows at the coordinator.
- Stacking the Task 229 sidecar into the candidate arm.

## Acceptance

1. Every source attribute has one authoritative tier and exact row
   reconstruction is test-pinned.
2. Exact-vector reads avoid the cold tier and are observable in counters.
3. DML and generation lifecycle behavior remains transactional and fail-closed.
4. Full-scale packet evidence supports a reviewed PROMOTE or STOP disposition.

## Required review packets

1. `reviews/task-230/001-plan/`
2. `reviews/task-230/002-format-and-read-path/`
3. `reviews/task-230/003-lifecycle-and-dml/`
4. `reviews/task-230/004-full-scale-decision/`

## References

- Tasks 222--224 and 229
- FR-076, FR-078, FR-079, FR-082, FR-083
- NFR-016, NFR-018, NFR-021, NFR-022
- PostgreSQL TOAST storage behavior
