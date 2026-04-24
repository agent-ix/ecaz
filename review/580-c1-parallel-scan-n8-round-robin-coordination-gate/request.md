# Review Request: Parallel Scan N8 Round-Robin Coordination Gate

Current head: `c9a9e90c1b34bed8d215cb9681da6de079da14a9`

Scope:
- `src/lib.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- The staged parallel coordinator had `n=1`, `n=2`, `n=3`, and `n=4` coverage,
  but still lacked the planned `n=8` worker-count gate.
- A first `n=8` fixture exposed the intended aggregate-budget split rather than
  a coordinator ownership bug: with the default budget, every worker was capped
  at a six-candidate bootstrap frontier and the combined stream stopped early.

What changed:
- Added a PG18 `n=8` round-robin fixture with 16 scalar rows.
- Set `ec_hnsw.ef_search = 160` inside the gate so the test isolates shared
  ownership/drain behavior instead of the per-worker budget split.
- Reused the shared many-worker assertions to require serial-equivalent output,
  all eight workers contributing, no hidden/blocker/active stranded state, and
  no local-only/deferred-local fallback emits.
- Updated Task 18 notes with the explicit full-traversal-budget rationale.

Why this matters:
- The staged coordinator contract is now covered at the planned `n=8` scale.
- The test distinguishes coordinator correctness from the still-deferred
  aggregate-budget policy and recall validation work.

Still intentionally deferred:
- planner-visible enablement and `amcanparallel = true`
- live parallel-plan benchmark and recall measurements
- explicit validation of the default-budget `n=8` split policy

Validation:
- Passed:
  - `cargo fmt --check`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18 test_ech_parallel_n8_round_robin_matches_serial_scores`
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether `ef_search = 160` is the right way to isolate `n=8` coordinator
  behavior from aggregate-budget policy.
- Whether the 16-row fixture is enough to stress all-worker contribution and
  serial-order equivalence at eight participants.
- Whether the default-budget split deserves a separate diagnostic/negative gate
  before planner-visible enablement.
