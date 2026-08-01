# Audit: NFR-017 … NFR-022 vs enforcement machinery

Task 214 P0 slice. Auditor: parallel subagent, 2026-08-01, worktree
`.worktrees/task-203` @ `baf81d498`.

## Verified intact (no drift)
- **NFR-017 best-effort**: spec records the 2026-07-17 ruling (`NFR-017:26-32`); no machinery hard-codes 0.999/37.6.
- **NFR-021 classes**: emitter emits `coordinator_resident_unsharded` and `control` (`distann_multicluster.rs:5515-5521`); reader accepts `bounded`/`control`/`coordinator_resident_unsharded`, any other/absent class → derived-bytes hard violation (`suite.rs:2170-2208`). The 0e4e28641 fix is intact.
- **Allowlist deleted**: `NFR_021_KNOWN_DISTRIBUTION_GAPS: [(&str, &str); 0] = []` (`suite.rs:2036`); coordinator-resident unsharded non-zero bytes hard-fails (`suite.rs:2290-2316`).
- **≤2.0 growth**: normalized bytes-per-owned-record gate enforced (`suite.rs:2234,2316`); raw fixed-roster growth emitted `reported_not_threshold_fixed_roster` (`suite.rs:1980-1984`), matching NFR-021's current text. The unmeetable raw row survives only in NFR-018 (F1).

## F1 — NFR-018 stale raw ≤2.0 growth threshold (contradicts NFR-021 + code)
- **spec:** `NFR-018:79` — "max single-node graph-side bytes, growth 100k ÷ 10k | ≤ 2.0"
- **code:** `suite.rs:1980-1984` (reported-not-threshold), 2234-2316 (only normalized ratio gated); `NFR-021:141-143` states raw ratio "is not a conformance threshold"
- **type:** specified-but-changed · **severity:** high
- The Task 205 "unmeetable on a fixed roster" row: NFR-021 was rebased, NFR-018 was not; the two NFRs contradict each other and the code sides with NFR-021.

## F2 — NFR-018 4.0× budget never evaluated
- **spec:** `NFR-018:77-78,89-91` — ratio ≤ 4.0, per-node bound, "breach fails the milestone closeout"
- **code:** `suite.rs:1885-1921` (row presence only); `distann_multicluster.rs:5528-5531` (ratio emitted, never compared); optional `ThresholdConfig` (`suite.rs:3550-3595`) that nothing requires
- **type:** specified-but-changed · **severity:** medium

## F3 — Unclassified relation → `nonconforming`, spec says `unavailable`
- **spec:** `NFR-021:129` — "an unclassified coordinator-resident relation makes the verdict `unavailable`, never a pass"
- **code:** `suite.rs:2175-2208` → derived bytes; :2311-2323 → `Nonconforming`
- **type:** specified-but-changed · **severity:** medium
- Load-bearing: an arm pre-registered `nonconforming` (legitimate context lane) matches a nonconforming verdict and passes (`suite.rs:2324-2331`), silently absorbing unclassified relations; `unavailable` would never match.

## F4 — Pre-registration screening only for the traversal replica
- **spec:** `NFR-022:22-24` — screening at pre-registration, before measurement; inadmissible candidate SHALL NOT be benchmarked
- **code:** `suite.rs:523/725` (registration optional), 4218-4245 (config-time screening only for `traversal_replica` variants); `local_head: true` steps caught only post-run (`suite.rs:2413-2447`)
- **type:** specified-but-changed · **severity:** medium

## F5 — 100%-labeled metric has no mechanical basis
- **spec:** `NFR-022:83,90-92` — non-conforming lanes labeled 100% in `results.jsonl`
- **code:** conformance rows only for registered arms (`suite.rs:2038-2077`); `nfr_021` optional; fixture emits no `local_head=`/head-sharding field on result rows
- **type:** specified-but-removed · **severity:** medium

## F6 — `--local-head` self-documents as "Control arm" against NFR-022
- **code:** `distann_multicluster.rs:177-182`
- **type:** shipped-but-unspecified · **severity:** low
- The decision-role guard exists (`suite.rs:3690-3699`), so a documentation/affordance contradiction, not a live bypass — but the flag invites the exact registration NFR-022 prohibits (and falls into F4's gap).

## F7 — NFR-019 EXPLAIN surface does not exist
- **spec:** `NFR-019:39-40` — "The scan SHALL report the per-query expanded-record count in EXPLAIN"
- **code:** `custom_scan.rs:66` — `ExplainCustomScan: None`; counter exists internally (`scan.rs:102/488`) with no EXPLAIN surface
- **type:** specified-but-removed · **severity:** high

## F8 — NFR-019 assertion regime does not exist
- **spec:** `NFR-019:70-87` — per-query MAX caps asserted every cell, BW×H per attempt, ≤1.1 cross-scale ratio, "any breach fails the run"
- **code:** only a `debug_assert!` (`scan.rs:541-544`, compiled out of release — the only builds the gate accepts); `stage_counters.rs` are per-backend aggregates entirely behind `#[cfg(feature = "distann-head-attribution-benchmark")]`; counters emitted only under `--distann-stage-counters` full-metrics mode and never compared (`distann_multicluster.rs:5206-5230`); no suite code compares any counter to BW×H, D, or the ratio
- **type:** specified-but-changed · **severity:** high
- The one hard runtime enforcement (stable-prefix duplicate-payload error, `custom_scan.rs:1103-1119`) is also feature-gated. Window/deepening policy itself matches spec (W=10 `options.rs:96`; D = effective·64 max 1024 `custom_scan.rs:1292`).

## F9 — NFR-020 drill-name taxonomy drifted
- **spec:** `NFR-020:39-48` — reused drill cases incl. `connection_reset_mid_batch`, `epoch_mismatch`
- **code:** `distann_multicluster.rs:6863-7100` — 12 drills; no `connection_reset_mid_batch` anywhere; `epoch_mismatch` split into `remote_content_divergence` + `epoch_bump_no_false_reject` (a pass-on-absence criterion the spec never describes)
- **type:** specified-but-removed · **severity:** medium

## F10 — NFR-020 boundary-injection claims exceed the fixture
- **spec:** `NFR-020:52-68,102-108` — 17 handoff + 9 publication + 8 scan-retention boundaries all injected, scored 100%
- **code:** `distann_multicluster.rs:6523-6612` (participant_down_partial + post-ack/pre-pointer only) + Task-199 drills (:1969,2500,2815,3414-3746) — roughly a half-dozen boundaries; no drills for pre-decision coordinator crash, abort-racing-recovery, retention-count drift, WAL/restart resume, malformed-entry/wire-version/replay handoff cases. Also unenforced: "participant pin/unpin ops | 0 | counter assertion" (no counter); epoch-mismatch single-retry (`scan.rs:565-569` implements, no drill asserts count)
- **type:** specified-but-changed · **severity:** medium

## F11 — NFR-021 head row's "build manifest inspection" not performed
- **spec:** `NFR-021:131` — head replica count ≥ 1 per roster shard, build manifest inspection
- **code:** `suite.rs:2132-2135,2276` (only `head_capacity_constant` checked); `options.rs:44,59` (sharded defaults on), 380-389 (`head_replica_count` default 0)
- **type:** specified-but-changed · **severity:** low
- Head shardedness enforced only indirectly via 0-byte coordinator relations. Clause 5 (shipped default) is satisfied.

## F12 — `bounded` class is reader-only with no producer, no vocabulary in spec
- **code:** `suite.rs:2170-2181` accepts+skips `bounded`; no emitter writes it
- **type:** shipped-but-unspecified · **severity:** low
- Nothing constrains what may claim the tag; NFR-021 defines the bounded-structure list (:56-63) but no class vocabulary. Same family: `outstanding_distribution_gap`/`unowned` scaffolding (`suite.rs:2294-2305`) is dead machinery from the deleted allowlist with no spec counterpart.

## F13 — NFR-017 SHALL-worded machinery absent (consistent with best-effort ruling)
- **spec:** `NFR-017:80-81,90-92` — suite SHALL invalidate recall/latency rows when topology audit absent/fails; informational netem injected-latency run
- **code:** invalidation only transitive (fixture aborts pre-emission, `distann_multicluster.rs:1261+`; NFR-021-registered arms require topology evidence `suite.rs:2271-2273`) — unregistered steps with missing topology rows keep recall/latency rows; no netem machinery at all
- **type:** specified-but-changed · **severity:** low

## Summary
NFR-021/NFR-022 conformance machinery is real and close to spec. Load-bearing
drifts: F1 (NFR-018 stale raw threshold), F7/F8 (NFR-019 verification regime
does not exist — no EXPLAIN, no cap assertions, feature-gated aggregate
counters), F3/F4/F5 (NFR-021/022 enforcement-shape gaps).
