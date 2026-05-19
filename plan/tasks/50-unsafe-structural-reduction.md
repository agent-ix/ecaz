# Task 50: Unsafe Structural Reduction (Post-Burndown)

Status: **deferred** — depends on Task 35 reaching zero baseline entries and on
Tasks 40, 41, and 43 landing the structural lifts they already own. This task
picks up where Task 35 stops: Task 35 ensures every `unsafe { ... }` site is
either deleted, wrapped, or accompanied by a specific `// SAFETY:` comment.
This task drives the *count* of `unsafe { ... }` blocks down by changing the
code shape, not by documenting the existing shape.

## Why

A baseline-zero Task 35 result proves every remaining `unsafe { ... }` site has
a written invariant. It does not prove the invariant is enforced by the type
system. Two failure modes survive a comment-only burndown:

1. The contract holds today but a future edit silently invalidates it; the
   comment goes stale and the reviewer is the only line of defense.
2. The contract is large enough that the comment paraphrases the code rather
   than naming the invariant, so the comment is information-free.

Structural reduction — encapsulation, type-lifted invariants, narrowed blocks,
container-owned state, closure APIs — moves the contract into types and lets
the borrow checker, not the reviewer, enforce it. The remaining `unsafe`
becomes small, local, and tied to a single FFI or layout fact.

## Scope

Audit and structurally reduce `unsafe { ... }` blocks in the densest remaining
modules after Task 35 closes. Use the post-Task-35 distribution from
`make unsafe-baseline-report` (when baseline is zero, switch to a direct count
script that just greps `unsafe\s*\{` per file). Initial expected hotspots,
subject to refresh once Task 35 ends:

- `src/am/ec_hnsw/scan.rs` and `src/am/ec_hnsw/build.rs`
- `src/am/ec_spire/` coordinator, custom_scan, scan/relation
- `src/am/common/parallel.rs` production paths
- `src/am/ec_ivf/` scan/build/insert/vacuum
- `src/am/ec_diskann/scan_state.rs` and routine
- shared `src/quant/` and pgstat shims

Excluded — owned by adjacent tasks, do not duplicate:

- PG resource RAII (relation/buffer/snapshot/LWLock guards): owned by Task 41.
- Concurrency state machine lifts for Loom/Shuttle: owned by Task 40.
- Proof-side `unsafe` coverage (Miri/cargo-careful): owned by Task 43 / 43b.

This task may *consume* helpers introduced by those tasks (e.g.
`AccessShareIndexRelation`, lifted modules) but does not own them.

## Techniques

Each slice should pick one or more of the following patterns and apply it
across a coherent surface. Do not mix patterns within a single packet.

1. **Encapsulate at the FFI boundary.** Wrap a raw PG handle or opaque pointer
   in a newtype whose constructor is the only `unsafe fn`. All callers become
   safe.
2. **Lift invariants into references.** Replace `*mut T` threaded through a
   call chain with `&mut T` at the callback boundary. The borrow checker
   replaces the SAFETY comment that asserts "remains live for ...".
3. **Narrow the block.** Split an `unsafe { ... }` that wraps ten lines into
   the smallest expression that actually requires it. Block count may rise
   short-term; each remaining block is smaller and easier to remove later.
4. **Replace pointer-cached state with owned containers.** A `*mut Box<Map>`
   stored on a scan opaque becomes `Option<Map>` stored inline once the
   opaque itself is reached through a safe reference.
5. **Closure-style leaf APIs.** `with_X(handle, |view| ...)` moves the unsafe
   into one place per concern; callers stay safe.
6. **Delete dead unsafe.** Sites whose invariant is "this used to be needed
   because of \<thing fixed by a later refactor\>" should just go.

## Slice and Packet Rules

- Each packet must report the actual `unsafe { ... }` block count per affected
  file, before and after. Use a direct grep, not the Task 35 baseline file.
- Each packet must show net block-count reduction on the file it claims to
  reduce. Documentation-only changes are out of scope; route those back to a
  reopened Task 35 packet if any survive.
- Helpers introduced by a slice must themselves contain `unsafe` only where
  unavoidable, with `// SAFETY:` contracts and a one-line statement of the
  encoded invariant. Moving an `unsafe { ... }` into a helper does not count
  as reduction unless the helper is called from ≥ 2 sites.
- If a slice deletes a block whose invariant Task 35 had carefully documented,
  cite the Task 35 packet so the deletion of the comment is traceable.

## Performance Gate

Structural changes must not regress hot paths. Each packet that touches a
scoring, traversal, or cache hot path must include before/after measurements
from the relevant existing bench lane:

- HNSW: `bench` recall + QPS on the standard corpus profile.
- IVF: `bench` recall + QPS.
- DiskANN: `bench` low-L latency curves.
- SPIRE: read-efficiency bench from Task 30 phase 13d.
- Common build: parallel-build slot path latency where touched.

Acceptance: regression tolerance is the same as the corresponding M5
optimization tasks (Tasks 31-33). Any regression beyond noise blocks the
packet until investigated. Typical suspects when a regression appears:

- Accidental `.clone()` / `.to_vec()` introduced when migrating
  `*mut T` → owned `T`.
- Missing `#[inline]` on a small newtype constructor or accessor.
- New `Box::new` on a hot allocation site.
- Lost niche optimization growing a struct by a word (`Option<*mut T>` →
  `Option<&T>` is fine; `Option<HashMap>` inline may grow the parent).

Bench evidence must be packet-local under
`reviews/task-50/{ordinal-topic}/artifacts/` with a `manifest.md` recording
head SHA, lane, command, and isolated-vs-shared table choice per the
NFR-007 storage rule.

## Validation

Every packet must run, in addition to bench evidence above:

- `cargo fmt --all`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
  on touched modules
- direct unsafe-block count per touched file: before and after
- runtime tests on the touched module when behavior could plausibly drift
  (cache-hit semantics, error-path messages, callback ordering); skipped
  tests must be named in the request.

PG17 coverage remains optional unless the slice is PG17-facing.

## Exit Criteria

Task closes when:

- the densest residual modules (top-15 by block count at the time Task 35
  exits) have been processed at least once,
- each processed module's block count has dropped by at least 30% from its
  post-Task-35 state, or the request explains why a lower reduction is the
  structural ceiling,
- no bench lane regresses beyond its tolerance,
- a closing summary packet records the final per-module distribution and
  names the next-highest-density modules for a possible follow-on lane.

## Coordination

- Do not start while Task 35 is open. Coordinating two passes against the
  same files produces churn.
- Coordinate with Task 41 when touching PG resource sites; consume its
  wrappers rather than introducing parallel ones.
- Coordinate with Task 40 when touching shared-state modules it has lifted;
  reuse the lifted-module pattern rather than fighting it.
- Coordinate with the active M5 optimization tasks (31-33) for the bench
  windows; do not run a perf-sensitive packet against a corpus another
  packet is rebuilding.
