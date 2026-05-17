# Review Request: C1 ADR-030 V2 Loaded-State Unavailable Seam

## Context

Packet `325` carried `GraphStorageDescriptor` into scan state and routed cached graph-element loads
through typed graph tuple accessors.

That made the cache format-aware, but the loaded-state bookkeeping still collapsed two different
cases into `LoadedElementState::None`:

1. no cached score or payload has been loaded yet
2. a live grouped-v2 tuple was loaded, but it intentionally has no exact scalar payload in the hot
   tuple

That ambiguity is the wrong seam for future grouped scoring.

## Problem

Grouped-v2 graph tuples are supposed to load enough header state to participate in traversal before
exact rerank is available.

With the old state model, a grouped live tuple ended up looking the same as a cache entry that had
not loaded any score state at all. That creates two risks:

1. later grouped-score work cannot distinguish `header loaded, exact unavailable` from `nothing
   loaded`
2. generic exact-score fallback paths may keep treating grouped tuples as if they should lazily
   discover scalar score inputs later

We need a narrow state seam that makes grouped exact-unavailability explicit without enabling
grouped scoring yet.

## Planned Slice

Add one new loaded-state variant:

1. keep existing exact-score and exact-payload states unchanged
2. map live tuples with no exact payload to `ExactUnavailable`
3. fail explicitly if exact scoring is requested from that state

This still excludes:

- no grouped-v2 runtime enablement
- no grouped approximate scorer
- no grouped rerank implementation
- no change to the existing grouped-v2 ordered-scan rejection

## Implementation

Updated `src/am/scan.rs`:

1. added `LoadedElementState::ExactUnavailable`
2. added `live_loaded_state_from_exact_payload(...)` to centralize how live tuple payload state is
   classified
3. changed `cached_graph_element(...)` so live grouped-v2 tuples record `ExactUnavailable` instead
   of falling back to `None`
4. tightened the live-element assertion so `None` no longer silently covers grouped live tuples
5. changed `exact_score_cached_graph_element(...)` to fail immediately with the existing
   grouped-v2 unsupported-runtime error when it sees `ExactUnavailable`

This is still a seam packet rather than a capability packet. Grouped-v2 scans remain unsupported,
but the cache state now preserves the difference between “exact input exists” and “exact input is
structurally absent from the loaded tuple.”

## Measurements

This packet does not add a new runtime feature, so there are no new latency or recall
measurements.

Known validation results for this attempt:

- first parallel checkpoint attempt:
  - `cargo test`: invalid as a representative checkpoint because it overlapped with `cargo pgrx
    test pg17` and shared-target contention polluted the run shape
  - `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`: invalid for the same reason; the run
    failed broadly from concurrent target/install contention rather than from a slice-local code
    failure
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`: passed
- clean serial rerun:
  - `cargo test`: passed
  - `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`: passed
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`: passed

## Outcome

ADR-030 v2 scan state now records grouped live tuples as “exact unavailable” instead of pretending
they have no loaded state at all.

What this de-risks:

1. future grouped-score work can key off an explicit state boundary instead of inferring grouped
   absence from `None`
2. exact-score fallback paths now fail at the state boundary rather than drifting into generic
   scalar assumptions
3. the next scan-side slice can start introducing grouped approximate score state without first
   untangling ambiguous cache bookkeeping

## Next Slice

The next narrow slice should build the first grouped score-carrier seam:

1. extend loaded-state or cached-element state so grouped tuples can carry packed grouped search
   codes explicitly
2. keep exact rerank unavailable, but make grouped hot-payload availability explicit at the cache
   boundary
3. prepare the minimal scan-side hook needed to plug in a grouped approximate scorer later
