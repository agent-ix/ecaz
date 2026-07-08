# Review request — Task 164 M2: two-node loopback (TC-040/041 + H×RTT)

**Branch:** `task-164-ec-distann-m2` (stacked on M1).
**Milestone:** M2 read-path exit evidence + the transport rework that folds in
the two prior transport-review findings.

## What landed

- **Transport rework** (`remote_transport.rs`), addressing the transport
  pre-checkpoint review:
  - **Parallel per-node calls** (`remote_expand_batch` + `join_all`): a hop
    round now costs ~max remote RTT, not the sum (transport-review P2).
  - **Parameterized session setup** via `set_config($1,$2,$3)` — no more string
    interpolation of the roster spec (transport-review P2 injection).
  - `$1::text::regclass::oid` so the index name binds as text (transport-review
    P1); typed error classification by wire SQLSTATE; position-based reassembly.
- **`ec_distann_debug_expand_search`**: runs the FR-081 orchestration with the
  roster-selected expander, returning ranked hits — the TC-040/041 surface.

## Evidence (`artifacts/loopback-results.log`, `manifest.md`)

Against a committed DB with a real 10k/dim-1536 index (and a toy smoke),
release `.so`:

- **TC-040/041: 2-node top-k IDENTICAL to single-node** — same vec_id set,
  same rank→vec_id map, exact_dist within 1e-6, at both scales. The full remote
  path (group → parallel tokio-postgres transport → endpoint → reassemble) is
  result-identical to the single-node build.
- **Latency:** loopback transport costs ~2.8–3.0 ms/query end-to-end; on the
  small 10k corpus (compute ~0.7 ms) that's ~80% share. The gate-relevant D4
  evaluation (transport share vs 50%) is deferred to the M4 matrix on the real
  100k/dim-1536 corpus, where G0's ~12 ms compute puts the same transport at
  ~20% (under the trigger). See manifest for the honest read + a per-call
  `set_config`→connect-time optimization.

## Known-open (documented in the manifest)

- **Materialization** (transport-review P1): remote hits carry
  `heap_tid=INVALID`, so `RemoteNodeExpander` is deliberately not wired into
  `amgettuple` yet — a user-facing multi-node `ORDER BY … LIMIT` needs a
  materialization tier (SPIRE-CustomScan-style). Scoped next. M2 read-path
  correctness is proven at the orchestration level (this packet).
- FR-082 restart-on-mismatch loop is M3; M2 delivers the retriable
  classification it will consume.

## Ask

Please review the transport rework (parallelism, parameterized setup, error
classification) and the TC-040/041 evidence. Confirm whether the materialization
tier should land inside M2 or M3 before I proceed. The prior findings' fixes are
also summarized in `feedback/2026-07-08-02-coder.md` (packet 001). Not closing
any request.
