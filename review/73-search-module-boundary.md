# Request: Search Module Boundary

Commit: `0c61523`

Summary:
- Adds a dedicated `src/am/search.rs` module for pure traversal mechanics instead of continuing to grow beam/frontier logic inside the PostgreSQL-facing scan executor.
- Introduces a self-contained `BeamSearch` helper with best-first expansion, visited deduplication, bounded `ef_search` expansion, and pure Rust unit coverage.
- Keeps this slice structural only: the new search helper is not wired into `amgettuple` yet.

Files:
- `src/am/mod.rs`
- `src/am/search.rs`

Why this matters:
- This creates a durable parallel-work boundary between PostgreSQL scan plumbing and traversal algorithm work.
- It gives later beam-search integration a pure-Rust seam to evolve under test without forcing every algorithm change through scan executor state and pg-specific ownership.

Review focus:
- Whether the `scan` versus `search` boundary is the right long-horizon seam
- Whether the initial `BeamSearch` helper captures the right minimal traversal responsibilities without leaking pg concerns
- Whether the current tests are sufficient to lock in best-first, dedup, and bounded-expansion behavior before integration
