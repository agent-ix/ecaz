# Review Request: Task 28 IVF Optimization Handoff

Scope: Phase 8 handoff planning after the PG18 correctness and integration
checkpoints are green. This packet does not make a measurement claim. It
records which IVF follow-on optimizations are worth pursuing, which ones can be
developed safely on a slower machine before benching on faster hardware, and
tracks the first such slower-machine checkpoint.

Task: `plan/tasks/28-ivf-access-method.md` Phase 8

Branch: `task28-ivf`

Head SHA: `541f35f64fe0a5cb57cfd3ec46d57840da89fd8d`

Owner: coder2

Files:

- `src/am/ec_ivf/scan.rs`
- `src/am/ec_ivf/page.rs`
- `src/am/ec_ivf/insert.rs`
- `src/am/ec_ivf/cost.rs`
- `plan/tasks/28-ivf-access-method.md`

Validation for the current slower-machine checkpoint:

- `cargo test --no-default-features --features pg18 am::ec_ivf::scan::tests`
- `git diff --check`

## Current State

PG18 integration work for IVF is in place:

- PG18 planner callbacks are wired.
- PG18 shared stats are wired.
- PG18 posting-list reads already use `ReadStream`.
- The remaining work is measurement and performance tuning, not more base PG18
  compatibility work.

The current scan path already benefits from PostgreSQL 18 async I/O indirectly
through `ReadStream`; the extension does not need a separate direct AIO API
integration patch.

## Optimization Candidates

### Safe to develop on the slower machine

These are structural or algorithmic improvements that can be implemented and
validated with unit tests, focused PG18 tests, and code review before running
real measurements on the faster machine:

- Multi-range posting-list read planning:
  replace one-`ReadStream`-per-list setup with one scan-level plan that can
  walk all selected posting-list block ranges in score order or block order.
- Score-as-you-read candidate flow:
  avoid materializing all postings into a temporary `Vec` before scoring and
  dedup/top-k selection.
- Partial centroid selection:
  avoid full sort of all centroid scores when only `nprobe` best lists are
  needed.
- Candidate-state allocation cleanup:
  reduce intermediate allocations in probe candidate materialization and result
  handoff.
- Quantizer batch plumbing:
  add internal batch-friendly interfaces so `turboquant`, `pq_fastscan`, and
  `rabitq` scoring paths can process posting payloads in chunks once the
  benchmark data shows where the hot path is.

These changes are good slower-machine work because they do not require large
corpus timing data to validate correctness, and their value can be reviewed
structurally before performance numbers matter.

## Current Slower-Machine Checkpoint

Checkpoint in progress / complete on this branch:

- Bounded centroid probe-list selection now avoids sorting the full centroid
  score vector when `nprobe < nlists`.
- The scan path retains the same externally visible probe-list ordering
  contract: higher centroid score first, then lower `list_id` as the
  deterministic tiebreaker.
- Unit coverage now locks in both the truncation behavior and the tiebreak
  contract for `select_probe_lists`.
- IVF posting-list scan now has a visitor seam in `page.rs`, and
  `materialize_probe_candidates` consumes postings directly while buffers are
  read instead of materializing one temporary posting `Vec` per selected list.
- The current score/dedup/top-k behavior is intentionally unchanged; this slice
  only removes intermediate posting allocation from the scan path.
- PG18 scan now merges the selected IVF list ranges into one deduplicated
  posting-block sequence and streams that union through a single sequential
  `ReadStream`, instead of starting one `ReadStream` per selected list.
- Overlapping or shared posting blocks across selected lists are now read once
  per scan materialization pass, with posting-level list filtering preserved in
  the scan layer.
- Scan candidate materialization no longer retains a full IVF directory vector
  just to derive selected-list counts and ranges; it now builds a narrower
  selected-probe plan directly from the ordered directory chain.
- Runtime candidate materialization now sorts the deduplicated candidate set
  directly after the probe pass instead of feeding it through a no-op top-k
  heap whose bound already covered the full selected-list live-count upper
  bound.

Why this slice is worth doing here:

- It is a pure scan-path structural improvement.
- It does not depend on benchmark hardware to validate correctness.
- It reduces avoidable work in a path that scales with `nlists`, which makes it
  a safe precursor to later `nprobe` / `nlists` measurement sweeps.

### Better left for the faster machine or measurement lane

These depend on real timing, cache, I/O, or WAL behavior and should not be
treated as wins until measured:

- AIO / `io_method` comparison:
  cold-cache `sync` vs `worker` vs `io_uring` behavior under PG18.
- `nlists` / `nprobe` recall-latency sweeps.
- `storage_format` comparisons across `turboquant`, `pq_fastscan`, and
  `rabitq`.
- WAL and storage tuning:
  page density, build WAL volume, live-insert WAL volume, vacuum rewrite cost.
- Duplicate-check optimization for live insert:
  correctness-first behavior is already in place; changing it should wait for
  ingest-path profiling.
- Planner-cost constant tuning:
  cost model shape exists, but constant selection should follow measured scan
  behavior.

### Order of operations

Recommended sequence:

1. Establish Phase 8 baseline measurements first.
2. Use those measurements to identify whether IVF is scan-bound, scoring-bound,
   allocation-bound, or insert/WAL-bound.
3. Pick one structural optimization at a time and re-measure against the same
   baseline.

Without that baseline, it is too easy to spend time on plausible cleanups that
do not move recall, latency, or WAL in a meaningful way.

## Recommended Near-Term Work Split

Work that can happen here on the slower machine:

- Prepare the multi-range read design note.
- Refactor candidate materialization toward score-as-you-read.
- Add bounded-selection logic for centroid top-`nprobe`.
- Add batch-scoring seams for the available quantizer profiles.

Work that should stay with the fast-machine bench lane:

- Full recall packet.
- Warm/cold latency packet.
- Storage/WAL packet.
- AIO `io_method` comparison packet.
- Final planner/cost tuning based on those results.

## Review Focus

Please review for:

- whether the slower-machine / faster-machine split is the right operational
  boundary;
- whether multi-range reads and score-as-you-read are the highest-value first
  tuning candidates;
- whether any listed “safe here” item actually depends on benchmark data first.
