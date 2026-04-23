# Task 18: Parallel Index Scan

Status: proposed — broad-reach latency win, no Postgres vector extension has this today.

Executes ADR-040.

## Scope

Enable `amcanparallel=true` for `ec_hnsw` so a single `ORDER BY v <#> q LIMIT k`
query can be split across multiple Postgres workers. Workers run independent
beam searches against a shared top-K coordinator; `ef_search` is budgeted per
worker with a small overlap term so aggregate recall matches (or exceeds) a
single-worker scan at the same total budget.

Goal: linear-ish latency reduction on warm indexes for 2/4/8 workers, and
automatic inheritance of parallelism by DiskANN (task 17) and any future AM
that shares the scan seam.

## Why now

- No other Postgres vector extension (pgvector, pgvectorscale, vectorchord)
  ships a parallel index scan today. Strongest single differentiator per unit
  of effort.
- Broad-reach: every `ORDER BY v <#> q` query benefits once the planner picks
  the parallel plan.
- Compounds with everything downstream. OPQ, AQ/RVQ, DiskANN, SPANN all
  inherit parallelism automatically because the seam is in scan coordination,
  not the scoring kernel.
- Cache-line contention is the only real risk, and the coordinator-only
  shared state (top-K heap) keeps contention bounded.

## Design outline

See ADR-040 for the full shape. Summary:

- **Shared state (DSM):** single top-K min-heap, protected by a lightweight
  lock. Workers push candidates that beat the current kth; coordinator pops
  on scan end.
- **Per-worker state (DSM slots):** independent beam frontier, visited set,
  and scoring scratch. No shared visited set (the coordination cost
  outweighs the redundancy savings at typical `ef_search`).
- **Budget split:** per-worker `ef_search = ceil(ef_search_total / n) *
  (1 + overlap)`, with `overlap` in the 5–15% range. Overlap compensates
  for workers missing neighbors the others already explored.
- **Entry points:** each worker starts from the same Layer-N+ entry point
  but with a distinct RNG seed for beam initialization (prevents all workers
  exploring identical paths).
- **Correctness invariant:** with `n=1` the parallel path must produce
  byte-identical results to today's serial path. Enforced by a scan-mode
  test.

## Subtasks

### Coordinator and DSM

- [ ] **DSM layout.** Define shared top-K heap, coordinator serializer, and
  worker state
  slots in `src/am/common/parallel.rs`. Size computed by
  `amestimateparallelscan`.
- [ ] **Shared top-K push/pop.** Lock-guarded; hot path is "is candidate
  better than current kth". Fast-reject without taking the lock when
  candidate score is clearly worse than a snapshot of the current kth.
- [ ] **Per-worker state carriers.** Beam frontier, visited bitmap/bloom,
  score scratch. Lives in per-worker DSM slot, never touched by peers.

### AM callback wiring

- [ ] **`amcanparallel = true`** in the `IndexAmRoutine` for `ec_hnsw`.
- [ ] **`amestimateparallelscan`.** Returns DSM size = coordinator state +
  `n * per_worker_state`.
- [ ] **`aminitparallelscan`.** Populate coordinator heap, initialize
  per-worker slots.
- [ ] **`amparallelrescan`.** Reset coordinator and per-worker state for
  re-execution (nested loops, param re-bind).
- [ ] **Worker-side scan entry.** Each worker's `ambeginscan` path detects
  the parallel DSM slot and configures its local `TqScanOpaque` against
  it.

### ef_search budget split

- [ ] **Budget math in `resolve_scan_tuning`.** Compute per-worker
  `ef_search` as documented above. GUC `ec_hnsw.parallel_ef_overlap`
  (default `0.1`, range `[0.0, 0.5]`) controls the overlap term.
- [ ] **Single-worker equivalence test.** `n=1` parallel scan produces
  byte-identical results to serial scan at the same `ef_search`.

### Planner integration

- [ ] **Parallel cost in `amcostestimate`.** Extend the cost model (D2
  lane in task 11) so the planner prefers the parallel path on
  large indexes when `max_parallel_workers_per_gather > 0`.
- [ ] **EXPLAIN.** Surface per-worker counter rollups (pages read,
  elements scored) via the EXPLAIN hook from task 11.

### Tests and benchmarks

- [ ] **Correctness harness.** Same query across `n = 1, 2, 4, 8`
  workers must produce the same top-K identities (IDs may tie; require
  identity-set equality or a bounded score-delta tolerance).
- [ ] **Recall parity.** At the same *aggregate* `ef_search`, parallel
  recall should match serial within ±1 pp on the 50k warm real seam.
- [ ] **Latency benchmark.** Warm index, `LIMIT 10`, `ef_search=40`, rows
  between 100k and 10M. Report mean / p95 at 1/2/4/8 workers.
- [ ] **Contention stress.** Measure coordinator-lock wait time at 16
  workers to bound the scaling ceiling.

## Owns

- ADR-040
- `src/am/common/parallel.rs` (new)
- Parallel-scan callbacks in `src/am/mod.rs`

## Dependencies

- Task 15 (PqFastScan first-class). The parallel seam sits above scoring;
  once the two formats share a stable scan loop, adding parallel is
  additive rather than per-format.
- Task 11 D2 planner wiring. Parallel plan selection depends on the
  planner trusting `amcostestimate`, which is gated on the recall gate.

## Unblocks

- DiskANN (task 17) inherits parallel scan the moment it lands, since
  the seam is at the coordination layer not the scoring layer.
- SPANN (ADR-035) likewise.
- Multi-core utilization on any vector workload — the single biggest
  user-visible latency win short of a scoring-kernel change.

## Out of scope

- Parallel build. Build-side parallelism is a separate, larger project
  (coordinator-free, different bottlenecks).
- Cross-query batching on a single worker.
- Parallel vacuum.

## Notes

- **Staging checkpoint.** The first landing wires the callback surface and the
  shared AM-private descriptor while leaving `amcanparallel = false`. Planner
  visibility only flips once the coordinator and worker-local traversal
  contracts are live.

- **Descriptor sizing.** `amestimateparallelscan` does not receive the chosen
  executor worker count, so the staged shared descriptor reserves coordinator
  and worker-slot headers for up to `max_parallel_workers_per_gather + 1`
  participants.

- **Worker-slot staging.** Scan attachment now claims and releases one shared
  worker slot per live `TqScanOpaque`, keyed by the current rescan epoch.
  The slot also carries a staged runtime snapshot for phase, frontier, visited,
  emitted, and pending-result state at scan lifecycle boundaries.
  `amcanparallel` still stays `false` until the coordinator heap and
  worker-local traversal contracts are live.

- **Coordinator-result staging.** The shared descriptor now also reserves one
  coordinator-owned staged current-result slot per worker slot, keyed to the
  same rescan epoch. Scan lifecycle publishes the current result element/score
  state there, while the true shared top-K heap ordering and merge path remain
  deferred to the next Task 18 packets.

- **Coordinator selection staging.** Shared helpers can now scan the published
  coordinator result slots and pick the current best staged result by score,
  with slot-index tie-breaking for determinism. This is still a read-only seam;
  the real shared top-K heap mutation path remains deferred.

- **Coordinator snapshot staging.** The coordinator header now carries an
  explicit snapshot of the currently selected staged result slot and score.
  Publish, clear, release, and rescan refresh that snapshot so later merge
  work can read coordinator state directly without rescanning the staged slots
  on every access.

- **Claim-aware coordinator drain.** The staged coordinator selection/read/take
  path now treats a result slot as dead when its owning worker slot is no
  longer claimed for the active rescan epoch, refreshes past that stale
  fast-path entry before exposing the next live staged result, and reaps the
  dead staged slot from the shared published-result counts.

- **Coordinator fast-path staging.** Shared helpers can now read the staged
  selected result directly from the coordinator snapshot and slot header,
  without rescanning all staged result slots. The full shared top-K drain
  path is still deferred.

- **Coordinator take staging.** Shared helpers can now take the currently
  selected staged result, clear that slot, and refresh the coordinator fast
  path to the next best staged result when one exists. This is still a staged
  result-slot consume seam, not the final shared top-K heap drain path.

- **Shared heap frontier staging.** The shared descriptor now carries a
  coordinator-owned min-heap over the one-live-result-per-worker staged
  frontier, keyed by the current rescan epoch. Heap capacity stays bounded by
  worker-slot capacity, so the heap layout remains query-independent while the
  real lock-guarded push/pop admission path is still deferred.

- **Heap-root drain staging.** Coordinator staged-result take now clears the
  selected slot, pops the shared heap root in place, and refreshes the
  fast-path snapshot from the next heap root instead of rebuilding the entire
  heap after every staged consume. Full shared top-K admission and mutation
  remain deferred.

- **Incremental staged-heap maintenance.** Worker result publish, clear, and
  staged coordinator take now maintain reverse slot-to-heap membership and
  reheapify in place, so the shared staged frontier no longer does a full heap
  rebuild on every per-slot mutation. Full lock-guarded shared top-K admission
  remains deferred.

- **Serialized staged-heap mutation.** Coordinator staged-heap mutation now
  runs behind a shared lock word in the AM-private descriptor so publish,
  clear, and staged take no longer depend on the single-writer assumption once
  real parallel execution starts wiring in. Planner-visible parallel scans and
  the eventual shared top-K admission path still remain deferred.

- **Pending-output staging.** Each worker-frontier result slot now carries the
  full inline heap-TID buffer plus pending-index state, and the coordinator
  can drain one pending heap TID at a time without clearing the slot until that
  worker result is fully emitted. This is still the staged worker-frontier
  merge seam, not the final shared top-K admission heap described by ADR-040.

- **Coordinator pending-output fast path.** The coordinator snapshot now also
  caches the currently selected pending output itself, so later merge work can
  read the next global heap TID plus score metadata directly instead of
  recomputing it from the staged worker slot on every read.

- **Admitted-window consume staging.** The coordinator-owned admitted-result
  window can now return and remove its current best admitted result one at a
  time while keeping the remaining admitted prefix compact, score-ordered, and
  generation-tracked. This is still a staged consume seam, not the final
  planner-visible shared top-K execution path.

- **Admitted-head fast path.** The coordinator now also caches the current
  admitted-window head in its snapshot state so later final-output drain work
  can read the next admitted heap TID directly and only fall back to the
  shared admitted array when the cache needs refresh.

- **Admission probe fast path.** Workers can now read a claim-safe probe for
  the currently selected pending output and tell whether it would enter the
  admitted window before taking the coordinator serializer, including duplicate
  rejection and full-window tail comparison.

- **Admission fast-reject staging.** The mutating selected-pending-output
  admission path now returns directly from that probe state when the rejection
  stays current, so duplicate and full-window loser cases no longer need the
  coordinator serializer before preserving the admitted window unchanged.

- **Coordinator merge staging.** A staged merge helper can now choose between
  the admitted head and the selected pending output, admitting the selected
  output first when it beats the admitted head and otherwise draining the
  admitted head in score order.

- **Admitted-result provenance staging.** The admitted window now retains the
  source worker-slot index and element TID alongside each pending-output
  snapshot, and scan-side helpers can project an admitted row back into the
  normal `PendingScanOutput` shape while advancing the local duplicate-drain
  cursor when that admitted row came from this worker slot. The actual
  parallel scan execution loop still remains deferred.

- **Scan-side merge consume staging.** `produce_next_scan_heap_tid(...)` now
  checks the staged shared coordinator merge seam first when a parallel-scan
  descriptor is bound, consumes the admitted row through the scan-side helper,
  and republishes the local worker snapshot afterward so the next duplicate or
  next staged row stays visible. This still uses the descriptor-capacity
  admitted window because planner-visible LIMIT budgeting is not wired yet.
  Newly materialized linear-fallback rows now stage through that same shared
  merge seam instead of bypassing the coordinator on first emit. Prefetched
  graph-traversal rows now do the same at emit time.

- **Worker bootstrap diversification staging.** Parallel-bound scans now use
  the claimed worker slot plus `scan_seed` to rotate and stride the layer-0
  bootstrap tail while retaining the shared best seed candidate. Serial and
  `n=1` paths stay byte-identical because unbound and single-worker scans keep
  the original ordered bootstrap candidate list.

- **Capacity-based `ef_search` split staging.** Bound parallel scans now use
  `ec_hnsw.parallel_ef_overlap` (default `0.1`) plus the shared descriptor's
  worker-slot capacity to derive a staged per-worker bootstrap frontier limit.
  This is still an upper-bound stand-in for the eventual executor-visible
  actual worker count, so planner-visible cost and LIMIT budgeting remain
  deferred.

- **Owner-aware staged drain.** The scan-side shared take helper now only
  advances staged pending or admitted outputs for the owning worker slot.
  Foreign workers can observe that work exists, but they no longer mutate a
  peer's duplicate-drain cursor just by probing the shared merge seam.

- **Blocked-owner staging fallback.** When a foreign admitted head still stays
  ahead, the current staged scan helpers now return `None` instead of
  panicking, and the serial local emit path republishes the advanced local
  cursor back into the shared snapshot. This keeps the staging branch usable
  while the final multi-worker output-handoff contract is still deferred.

- **Explicit owner readiness staging.** The scan-side helper now distinguishes
  `Empty`, `Blocked`, and `Emitted` states instead of collapsing blocked-owner
  waits into a plain `None`. That gives the remaining handoff work a concrete
  state machine at the scan layer instead of inferring ownership from absence.
  The blocked state now also carries the blocker reason
  (`ForeignSelectedPending`, `ForeignAdmittedHead`, or `AdmissionWindow`) so
  the eventual worker/consumer handoff can branch on explicit ownership
  metadata instead of reverse-engineering it from coordinator side effects.
  The linear and graph scan paths now use that blocker taxonomy too:
  admission-window losers are dropped from the staged local current slot and
  local search continues, while foreign-owner blockers still use the explicit
  staging fallback until the real handoff contract lands.

- **Shared blocker snapshot staging.** Worker-runtime snapshots now publish the
  current ownership blocker kind and blocker slot when a scan is blocked on a
  foreign selected/admitted output. That makes the remaining handoff seam
  visible in shared state instead of only in local scan control flow.

- **Blocker generation snapshots.** Blocked-owner state now also carries the
  relevant coordinator generation (`result_publish_generation` for foreign
  selected output, `admitted_result_generation` for foreign head/window state)
  so workers can tell whether they are still blocked on the same foreign state
  or the owner already advanced underneath them.

- **Generation-aware foreign blocker retry.** Foreign-owner blockers now get
  one immediate retry when the blocker generation changes for the same staged
  row; only a repeated stable blocker falls back to the current local
  keep-and-emit path. That keeps the staged foreign-owner fallback from holding
  the same row forever after the owner has already advanced.

- **Local-only foreign fallback staging.** When a stable foreign-owner blocker
  does fall back to local emit, the row now becomes local-only between retries:
  the worker snapshot still reports an active local row, but the coordinator
  result slot is cleared until the next shared retry explicitly republishes it.
  The worker snapshot also retains the foreign blocker kind/slot/generation
  during that local-only window so the handoff seam stays visible even after
  coordinator publication is suppressed.

- **Foreign-duplicate suppression staging.** Foreign-owner blockers now also
  carry the blocking element TID, and scan-side ownership fallback drops the
  local row outright when the foreign worker already owns that same element.
  That keeps the staged handoff seam from degrading into a second local emit
  for an already-owned duplicate row.

- **Blocked-owner EXPLAIN counters.** The staged ownership blocker now also
  increments dedicated EXPLAIN counters for foreign-selected, foreign-head, and
  admission-window stalls so scan diagnostics can distinguish why a
  parallel-bound worker stayed blocked.

- **LWLock-backed coordinator serializer.** The shared coordinator serializer
  no longer uses the staged raw atomic lock word. The DSM heap state now embeds
  a real PostgreSQL `LWLock`, the descriptor initializer assigns and registers
  a named tranche for it, and attach-time validation re-registers that tranche
  before using the shared lock. The standalone unit-test backend keeps a local
  atomic shim over the embedded `LWLock.state` field so Rust unit tests do not
  trip `pgrx`'s cross-thread FFI guard while the runtime path still exercises
  real LWLock acquire/release on PG18. Runtime release now mirrors PostgreSQL's
  normal unconditional `LWLockRelease` path and relies on abort cleanup via
  `LWLockReleaseAll()` rather than a local `InterruptHoldoffCount` guard.

- **Foreign admitted-head handoff staging.** When a worker is blocked only by
  a foreign admitted head, scan-side handoff can now drain that already-admitted
  global row through the shared merge path instead of immediately degrading into
  local-only fallback. This still does not hand off foreign selected-pending
  cursors; it only consumes rows that are already in the admitted window.

- **Foreign selected-pending handoff staging.** The same scan-side handoff seam
  can now also drain a foreign selected-pending row through the shared global
  next-output path. This still is not a full ownership transfer protocol: the
  helper consumes the globally selected row, but the broader blocked-owner state
  machine and planner-visible parallel execution remain deferred.

- **Owner-slot reconciliation staging.** When a worker falls back behind a
  foreign-owner blocker, it now reconciles its local duplicate-drain cursor
  against the owning shared result slot before degrading into local emit. If a
  foreign worker already advanced or fully drained that slot, the local worker
  catches up and retries the shared seam instead of keeping a stale local
  cursor.

- **Post-handoff republish reconciliation.** The worker snapshot/result-slot
  republish path now also reconciles against the worker's own shared slot
  before publishing. That keeps a worker from re-staging an already-drained
  foreign-handoff row when another worker consumed its selected pending output
  first.

- **Stale foreign-selected handoff guard.** Foreign selected-pending handoff
  no longer goes through the generic global-next drain. It now only advances
  the specific blocked foreign slot while the blocker slot/generation still
  match, so a stale blocker cannot accidentally drain a newer selected row.

- **Deferred blocked-output stash.** A stable foreign-owner blocker no longer
  immediately forces out-of-order local emit. Instead, the worker can now hide
  its blocked local row in a scan-local deferred stash, keep that row visible
  to staged-duplicate suppression and blocker diagnostics, and resume shared
  work until the scan exhausts. Only once the shared seam is empty does the
  worker drain that deferred local row. This removes the eager `KeepLocalEmit`
  behavior, but it is still a scan-local fallback rather than a full
  cross-worker ownership transfer protocol.

- **Per-row deferred blocker metadata.** Deferred blocked rows no longer share
  one global retained blocker record. Each deferred row now carries its own
  blocker metadata, and worker-runtime snapshots publish the blocker attached
  to the best deferred row. That keeps ownership diagnostics aligned even when
  multiple blocked local rows accumulate before the final ownership-transfer
  seam lands.

- **Deferred-row shared handoff retry.** A deferred blocked row now remembers
  which scan phase produced it and gets one last shared-handoff retry before
  local emit. The scan temporarily restores that deferred row into its original
  graph or linear state, probes the shared ownership seam again, and only falls
  back to local emit if the foreign blocker is still unresolved. This still is
  not a full ownership transfer, but it narrows the remaining gap by retrying
  the shared seam at the last possible point instead of draining every deferred
  row locally by default.

- **Deferred-row obsolete-drop guard.** After that final shared retry, deferred
  rows no longer locally emit when the blocker proves they are already obsolete.
  Admission-window losers and same-element foreign duplicates now drop out of
  the deferred stash instead of bypassing the ownership seam on the last drain.

- **Deferred-row score-order preference.** The scan no longer waits for phase
  exhaustion to revisit every deferred row. When the best deferred blocked row
  already scores better than the currently active local row, the scan now drains
  that deferred row first. This still does not solve the final ownership
  transfer, but it narrows ordering drift by preferring the better deferred
  candidate before emitting a worse live local row.

- **Deferred-drain ready-row preference.** When deferred-only drain reaches a
  still-live blocked best row, the scan now keeps looking for the next ready
  deferred row before falling back to local emit. Only when no deferred row can
  hand off or drain safely does the staged path locally emit the remaining
  blocked row. This still is not the final ownership transfer, but it reduces
  unnecessary local fallback while preserving progress.

- **Deferred local-emit EXPLAIN counter.** That last-resort deferred local emit
  is now explicit in the `Ecaz Stats` output as
  `Parallel Deferred Local Emits`, so the remaining ownership gap is visible in
  PG18 explain output instead of staying hidden behind ordinary heap-tid
  returned counts.

- **Deferred local-emit blocker breakdown.** The same EXPLAIN surface now also
  splits that last-resort deferred local emit by foreign-selected versus
  foreign-admitted blockers, so the remaining ownership gap is measurable by
  blocker kind instead of only as one aggregate fallback count.

- **Deferred duplicate suppression against live foreign output.** Before that
  last-resort deferred local emit drains, the staged path now checks whether a
  still-live foreign selected/admitted output already owns the same next heap
  TID. If so, the local deferred path skips that duplicate heap TID instead of
  re-emitting it locally and only falls back to the next unique local heap TID.

- **Active duplicate suppression before defer.** The same live-foreign duplicate
  check now also runs at the first blocked-owner disposition for the active row.
  If the foreign selected/admitted output already owns the next local heap TID,
  the scan consumes that duplicate immediately, republishes its worker snapshot,
  and retries the shared seam before the row ever enters the deferred stash.

- **Deferred duplicate skip now reopens shared handoff.** When deferred local
  fallback skips a foreign-owned duplicate heap TID, it now retries the shared
  handoff seam for that row immediately instead of sliding straight toward local
  emit. That lets the worker drain the still-live foreign selected/admitted
  output before considering local-only fallback for the remaining unique row.

- **Local-only wakeup republished into shared state.** A row hidden in
  local-only fallback no longer gets mistaken for a stale drained owner when
  the foreign blocker clears. The next shared retry now republishes that row
  back into the coordinator slot first, then lets it resume normal shared
  drain/admission behavior.

- **Better deferred rows outrank hidden local-only wakeup.** When a concealed
  local-only row is still staged but a better ready deferred row already exists,
  the scan now lets that deferred row emit first instead of waking the
  concealed row immediately. The hidden row still wakes back into the shared
  seam afterward.

- **Ready deferred rows retry the shared seam.** A deferred row that is no
  longer carrying a live blocker no longer falls straight into deferred
  local-only emit. It now restores itself through the shared next-output seam
  first, so ready deferred work still drains under the staged coordinator
  contract before the branch uses the last-resort local fallback.

- **Current blocker.** `n=1` parity is live, but real multi-worker output
  ownership transfer is not. The staged shared merge seam still needs a
  concrete worker/consumer contract for genuinely blocked unique outputs
  before `amcanparallel` can flip on without duplicate or out-of-owner output
  hazards.

- **No shared visited set.** Cost analysis in ADR-040 shows the cross-
  worker synchronization cost exceeds the ~5–15% redundant-work savings
  for `ef_search ≤ 200`. Revisit if a workload emerges where `ef_search`
  is routinely above 500.
- **Overlap default.** Start at 10%. Can drop to 0 for throughput-
  sensitive workloads willing to accept a small recall hit.
- **Cache-line contention.** Top-K heap lock is the one hot contention
  point. Mitigation: workers snapshot the current kth score and
  fast-reject locally before taking the lock.
- **PG17 vs PG18.** Parallel index scan callbacks exist in both. No
  PG18 gate needed for this work.
