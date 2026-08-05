---
agent: claude
role: reviewer
model: claude-fable-5
date: 2026-08-04
seq: 02
---

# Task 206 packet 004 — corrected closeout: REQUEST CHANGES

Much of this round is right: BW64/H8 (the actual Pareto point) at
10k/50k/100k with 50 timed queries and 10 warmups, NFR-021 preregistered and
emitted as a structured conformance row (normalized growth 1.0947 vs 2.0 per
the spec's actual criterion, with the 11.12 raw fixed-roster growth honestly
reported and correctly not gated), real run commands in the manifest, the
owner control added, and the telemetry lane quarantined on the feature build.
All cited numbers match `run/results.jsonl`.

But the packet's decision axis is broken:

1. **P1 — the k_head 128-vs-200 A/B is inert; both arms ran identical
   configurations.** `ec_distann.benchmark_head_seed_count` and
   `benchmark_head_search_width` are feature-gated: the GUC statics and the
   getters exist only under `distann-head-attribution-benchmark`
   (`options.rs:74-77`, `:704-730`). The manifest's own install command —
   `cargo pgrx install --release --no-default-features --features pg18` —
   compiles them out, and `select_seed_candidates` then uses
   `(beam_width * 2).max(32)` = 128 for both arms
   (`generation_read.rs:3667`). The fixture's `SET
   ec_distann.benchmark_head_seed_count = 200` lands on an unregistered
   placeholder GUC and is silently ignored. The proof is in the results:
   recall is bit-identical between k128 and k200 at every scale
   (0.9884 / 0.9601 / 0.9585), and the `head_seed_count=200` echoed in the
   logs is the *requested* value, not the effective one. This is the
   inert-mechanism failure mode a third time (Task 205 pushdown, Task 203
   BW=8), now with an A/B built on it. Phase 3 (NEG-01 / k_head
   requalification) therefore remains **not done** — and note it *cannot* be
   done on a production build at all until `head_seed_count` is a production
   GUC/reloption. Either promote it (small, reviewable) or run the k_head
   lane on the feature build with the latency caveat stated, and in both
   cases log the *effective* seed count from inside the scan as activation
   evidence.

2. **P1 — per-round observability is still inert on the measured path.** The
   counters and parsers exist, but the NOTICE emitter lives only in the
   legacy local scan path (`routine.rs:614-641`); the physical/generation
   scan path never emits (`generation_read.rs:3488` returns counters that go
   nowhere — no `scan_profile_notice_enabled()` call outside routine.rs).
   The packet's own `round-run/100k` artifacts show the GUC set and zero
   `scan_round` rows — which the coder honestly reported rather than
   claiming numbers, appreciated. But the Task 206 phase-2 requirement
   (transport wait, straggler spread, expanded nodes, bytes per round) is
   still unmet, and the ~190 ms-vs-36 ms physical/single gap remains
   unattributed. Wire the emit into the physical path (it is a small change:
   the counters already come back to the coordinator). Two sub-defects when
   you do: transport ns/bytes fields are populated only under the
   attribution feature (`remote_transport.rs:1293-1356`) and print as
   *silent zeros* on default builds — label unmeasured as absent, not 0; and
   `expanded_nodes = responses.len()` equals `requested_nodes` by the FR-079
   contract, so it measures nothing.

3. **P3 — 10k owner control ran on the feature build** (correctly
   quarantined and labeled diagnostic). Fine, but say in the manifest that
   owner-lane latency is incomparable with the release matrix for that
   reason, not only that it is "diagnostic."

The BW64/H8 release matrix itself is decision-grade and does not need to be
re-run. What needs to change: a *live* k_head A/B (or an explicit deferral
with the production-GUC gap named), and either working per-round telemetry or
an operator-approved waiver of the phase-2 reporting requirement recorded in
the packet.
