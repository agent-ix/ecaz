# Review Request: ReadStream Pure Callback Functions

Scope:
- `src/am/stream.rs`
- `spec/functional/FR-019-async-io-read-stream.md`
- `spec/tests.md`
- `plan/tasks/11-planner.md`

What changed:
- Added a pure `ReadStreamCallbackResult` enum in `src/am/stream.rs` so the eventual PG18 binding
  has a concrete result contract to translate into a block number or `InvalidBlockNumber`.
- Added `graph_prefetch_callback(...)` and `linear_prefetch_callback(...)`, which consume the
  existing planner-owned callback state and return either the next block or `EndOfStream`.
- Added unit coverage for both callback helpers so the D1 ReadStream work is now more than just
  signatures and state carriers.
- Updated FR-019, the test matrix, and Task 11 notes to record that pure callback behavior now
  exists and the remaining PG18 work is the actual PostgreSQL callback binding and runtime wiring.

Review focus:
- Whether `ReadStreamCallbackResult` is the right pure representation for the PG18 binding seam
- Whether the new pure callback helpers strike the right balance between real D1 behavior and
  avoiding premature runtime or PostgreSQL integration
- Whether this is a better D1 endpoint for FR-019 than expanding the existing SQL snapshot surface
- Whether the callback helpers leave the eventual PG18 glue in the right state: just result
  translation and `read_stream` registration remain

Questions to answer:
- Should `ReadStreamCallbackResult` stay as a local enum, or be brought closer to PostgreSQL naming
  before real PG18 bindings exist?
- Are the current callback helpers and state carriers sufficient for the later scan-lane wiring, or
  is there another obvious pure seam still missing?
- Does this make the FR-019 D1 boundary clear enough that further stream work should wait for the
  runtime/PG18 lanes?
