# Task 214 P0 — Consolidated spec-vs-code drift + gap inventory

Date: 2026-08-01. Head: `baf81d498`, branch `task-203-ec-distann-conformance`.
Method: eight parallel audit slices (per-FR/NFR cluster + catalog inventory),
each walking the spec text against the implementation with file:line evidence.
Raw slice reports live in `artifacts/audit-*.md` and
`artifacts/catalog-inventory.md`; every item below traces to a slice finding
(referenced as `<slice>/<id>`). This inventory is the work list for Task 214
P1–P5 and the Task 211–213 P0 spec rounds.

Legend: **[P2]** amend/author FR/NFR via `/specify` · **[P3]** table docs ·
**[P4]** diagrams · **[ADR]** superseding ADR · **[CODE?]** candidate code fix
(spec may be right and code wrong — flagged for the user/coder, not fixed in
this documentation task).

---

## A. Architecture-level drift (the big four)

**A1. The sharded, membership-only head is the shipped default and is
normatively undocumented.** FR-080's core premise (coordinator-resident head,
zero-RTT first hops) is the inverse of shipped behavior; the membership blob,
the per-owner head-shard build/serve protocol, the §4.1 replica population +
attestation subsystem, the members-derived shard ordinal, the seed-merge
contract, and all five head-topology GUCs appear in no spec.
[P2: rewrite FR-080 around the sharded default; author the head-sharding +
replica FR content (aligns with Task 211's head FR work); ADR: amend/supersede
ADR-085 item 5/D3]
(fr080/F1-F11; fr075-076/F4; fr079-081/F12; fr077-078/F9; nfr/F11)

**A2. The FR-084 traversal replica was demoted to opt-in nonconforming and the
"rejected" TRAV-30 gateway copies shipped instead.** ADR-086's ACCEPTED
Decision (PROMOTE, replica-preference-normal) was affirmatively reversed by
Tasks 203/210; the gateway-copy mechanism (copy set, export endpoint,
`skip_neighbor_vec_ids` wire field, capacity GUC, batch-L re-application,
stats fn) has no owning FR and materially alters the FR-079/FR-081 contracts.
[ADR: one superseding ADR covering demotion + TRAV-30 selection; P2: edit
FR-084 to the opt-in/nonconforming posture, amend FR-079/FR-081 for gateway
copies (or a new FR); note: Task 212's crown FR should cite this lineage]
(fr084-adr/F-1,F-2,F-3; fr079-081/F7)

**A3. Two implementation lanes (legacy v4 vs physical v5) — the spec describes
only v5, but the v4 lane still ships and drives multi-node scans.** Two
incompatible epoch-fingerprint schemes (16-byte FNV vs 34-byte SHA-256); a
complete legacy lifecycle SQL surface with name collisions; the session-GUC
roster lane (`ec_distann.roster` carrying raw conninfo) contradicting FR-075's
"no session state or GUC overrides it" and FR-078's identity rules; a third
heap_tid lane (loopback multi-node) the spec taxonomy does not admit.
[P2: spec must either name the legacy lane and its boundaries (fixture-only?
deprecation path?) or the code should retire it — surface to user; the
research-no-backward-compat convention suggests the honest spec is
"v5 is the design; v4 is the fixture substrate" stated explicitly]
(fr082-083/F-01,F-02; fr075-076/F2; fr077-078/F10; fr079-081/F11)

**A4. FR-083 DML on the distributed lane is unimplemented and the spec reads
as if it ships.** v5 deletes are silently dropped (noop bulkdelete); routed
tombstone writes, the entire update contract (stable vec_id, replacement
append, directory redirect), distributed insert, and 2 of 3 remote-write
operations do not exist. [P2: re-scope FR-083 into implemented-now vs
committed-final-milestone with explicit status per clause; CODE?: silent
noop delete on v5 is a data-integrity hazard worth flagging independently]
(fr082-083/F-10..F-15)

## B. Observability contract drift

**B1. EXPLAIN counter surface does not exist** (`ExplainCustomScan: None`);
promised by FR-075 Outputs, FR-081-AC-5, NFR-019:39. Counters are
NOTICE-gated; per-node batch sizes + pool reuse only under the benchmark
feature. [P2 + CODE?] (fr075-076/F1; fr079-081/F8; nfr/F7)

**B2. NFR-019's assertion regime does not exist in machinery** — no per-cell
cap assertions, debug_assert-only BW×H check, feature-gated aggregate
counters, no cross-scale ratio rows. [P2: rewrite NFR-019 verification to
what is enforceable, or spec the machinery as an obligation] (nfr/F8)

**B3. Activation counters are feature-gated out of production builds**
(`head_replica_shards_served`, stage counters) — collides with the standing
ruling "every new mechanism ships with activation counters asserted non-zero
in its A/B"; needs a spec decision on counter availability.
(fr080/unspec-6; fr077-078/unspec stage-counters)

## C. Per-FR itemized drift (P2 amendments)

- **FR-075**: GUC surface understated (2 listed vs ~17+8 shipped); deployment-
  mode sentence wrong for default lane (A3); `build_shards` reloption missing;
  amoptions-time validation claim false for string reloptions; EXPLAIN (B1).
  (fr075-076/F2,F3,F5,F10)
- **FR-076**: default vec_id is a local heap-TID hash mode the spec never
  mentions; "epoch-versioned" record wording overstates; default lane still
  writes legacy `(0x09,0)` prefix; handoff identity pinned to 16 bytes.
  (fr075-076/F6-F9)
- **FR-077**: manifest stats/wall-time/peak-memory rows unsatisfiable (AC-3,
  CON-4) — stats are log-only [CODE? or spec rebase]; reachability-repair pass
  unspecified; extract-to-shared reuse-mode paragraph stale; auto shard-count
  policy undefined. (fr077-078/F1-F4)
- **FR-078**: `recover_epoch_publish` signature drift (also FR-082);
  `last_error_category` never emitted; status fails closed on remote
  participants; `build_epoch_with_training` + training-relation contract
  unspecified; membership digest vs "canonical head sample" (A1); partitioned-
  source rejection, preload + global-gate-lock preconditions, trained-cap-4096
  rule unspecified; candidate-vs-manifest digest wording. (fr077-078/F5-F14)
- **FR-079**: 8-param physical overload + query-digest session state; legacy
  telemetry columns; caller-supplied send-function names on the legacy lane
  (AC-9 violated) [CODE?]; `EC_VECTOR_MISSING` never raised, silent drop on
  legacy lane [CODE?]; error taxonomy diverges on 4 of 9 rows; three
  unspecified auxiliary endpoints; NoTls/NFR-014 deferral undeclared.
  (fr079-081/F1-F6,F10)
- **FR-081**: gateway copies alter the merge step (A2); four RPCs bypass the
  deadline/interrupt wrapper [CODE?]; EXPLAIN (B1); head-descent wording
  predates sharded head (A1). (fr079-081/F7-F9,F12)
- **FR-082**: fingerprint duality (A3); recovery signature; advisory-lock +
  scan-triggered-T4a text stale; abandon replay ignores caller [CODE?];
  `EC_PUBLISH_PENDING` absent; parent-epoch ordering checked too late [CODE?];
  status 11-column/`CancelledReclaimed` extension; no `Aborted` generation
  state; `reclaim_cancelled_generation` signature missing from the function
  list; retirement head-state cleanup unmentioned. (fr082-083/F-01..F-09,F-16)
- **FR-083**: unimplemented contract (A4); fold endpoint outside the protected
  class [CODE?]; collision handling collapses spec branches; duplicate
  AC-5 identifier (spec defect). (fr082-083/F-10..F-15,F-17)
- **FR-084**: posture rewrite (A2); SECURITY DEFINER claim vs invoker-rights
  reality; four extra control/recovery functions + VACUUM invalidation +
  password-file GUC + suppression cache unspecified. (fr084-adr/F-3,F-4,F-5)

## D. Per-NFR itemized drift (P2)

- **NFR-018**: stale raw ≤2.0 growth row contradicts NFR-021 + code (the Task
  205 unmeetable row — remove/rebase); 4.0× budget never mechanically
  evaluated (state the manual-gate reality or spec the check). (nfr/F1,F2)
- **NFR-019**: B1/B2 above. (nfr/F7,F8)
- **NFR-020**: drill-name taxonomy drifted (`connection_reset_mid_batch`
  absent; `epoch_mismatch` split); boundary-injection claims exceed the
  fixture by dozens of named boundaries; pin/unpin counter row has no counter;
  retry-count assertion missing. Rebase the taxonomy to the shipped drill
  matrix and mark unimplemented boundaries as open obligations. (nfr/F9,F10)
- **NFR-021**: unclassified-relation verdict is `nonconforming` not
  `unavailable` [CODE?]; head row's "build manifest inspection" not performed;
  `bounded` class has no producer and no spec vocabulary; dead
  `outstanding_distribution_gap`/`unowned` scaffolding. (nfr/F3,F11,F12)
- **NFR-022**: pre-registration screening only for the replica [CODE?];
  100%-labeling metric has no mechanical basis (no `local_head=` field on
  result rows) [CODE?]; `--local-head` flag doc says "Control arm" against the
  NFR's own rule [CODE: reword flag doc]. (nfr/F4,F5,F6)

## E. Catalog surface (P3 work list)

20 tables enumerated in `artifacts/catalog-inventory.md` with DDL/readers/
writers/lifecycle; **17 of 20 appear in no spec** (only `node_descriptor`,
`build_candidate` by name in FR-078, replica domains in FR-082/084). P3
authors per-table docs incl. NFR-021 storage class per relation. Findings to
carry: 4 tables missing from the REVOKE block; `head_shard_replica` +
`head_replica_state` have **no deletion path** (epoch/index leaks) [CODE?];
legacy v1 endpoint name collisions; `active_epoch` single-pointer invariant;
the `membership` CHECK encoding NFR-021 clause 3.

## F. Diagram inventory (P4 work list)

From the audited flows: (1) epoch build T-stages T1→T2→T3→T4a incl. candidate/
decision/disposition tables; (2) publish/decide/retire/abandon + cancel
recovery state machine; (3) sharded head search — coordinator membership →
per-owner shard build/cache → seed merge, incl. replica serving + attestation
gate + clamp-to-owner fallback; (4) replica population/attestation
(export→import per pair → attest-after-all); (5) gateway-copy population +
skip-mask expansion + coordinator candidate-half reconstruction; (6) scan
open/epoch pin/retirement fence lifecycle (scan_registry); (7) DML paths as
shipped (legacy delta/tombstone; v5 posture). Domain-model ER diagram of the
20-table catalog belongs to the P1 FR-048-style anchor.

## G. Verified-conformant (no action; recorded to bound the audit)

Placement hash + golden vectors; all 15 digest domains; handoff wire formats
field-for-field; 107-byte hash state + sha2 pin; frozen v1 validity domain;
registration digest + lock order; `require_read_committed` everywhere;
manifest v2 + 303-byte receipt + 34-byte fingerprint (v5); T3/T4a CAS
machinery; scan registry fences/tokens per FR-082; Task 205 pushdown semantics
(threshold from L-th retained, batch-L once, gateway re-application); lazy-10
window policy; FR-079-AC-1 positional reassembly at every layer; privilege
DO-block incl. its three named exceptions; v4/v5 metadata page layouts;
record/tuple arithmetic; Vamana core shared not forked.

## H. Cross-cutting notes for sequencing

- P1 elevation should land **before** P2 so amendments write to
  `spec/functional/distann/` paths once. 31 files reference the old path
  (list captured in packet manifest).
- Task 211 (head scaling law FR) and Task 212/213 (crown/fused-hop FRs) write
  into the same FR-080-successor territory as A1 — sequence: elevate (P1),
  rewrite FR-080 + head-sharding content (P2/A1), then 211/212/213 P0 specs
  against that base.
- Items tagged [CODE?] are candidate code bugs, not documentation drift:
  silent v5 delete noop (A4), silent row drop on missing payloads (FR-079 F4),
  four unwrapped RPCs (F9), fold-endpoint hardening (F-14), abandon-replay
  caller check (F-05), parent-epoch ordering timing (F-07), replica-table
  leaks (E), NFR-021 unavailable-verdict shape (D), NFR-022 labeling gaps (D).
  These go to the user/coder as a list; Task 214 does not change runtime code.
