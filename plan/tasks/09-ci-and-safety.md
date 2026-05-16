# Task 09: CI Pipeline and Safety

Status: **mostly complete** — fuzz harness, CI pipeline, Makefile targets,
clippy.toml, property tests, miri tests all landed. Formal unsafe audit
enforcement is active for new unsafe sites; legacy unsafe debt is tracked under
Task 35.

## Scope

Wire existing build/test/lint targets into automated CI. Add fuzz harness and unsafe audit enforcement.

## Subtasks

- [x] **CI pipeline.** GitHub Actions running: cargo fmt --check, cargo clippy -D warnings, cargo test, layout assertions, property tests (256 cases). Criterion + benchmark-action (110% alert threshold) on main push. Miri on main push. Fuzz on nightly.
- [x] **Fuzz harness.** 4 fuzz targets: `parse_text`, `unpack_mse`, `element_tuple_decode`, `neighbor_tuple_decode`. Structure-aware input derivation.
- [ ] **Unsafe audit.** Review all legacy `unsafe` blocks for SAFETY comments
  through Task 35. `check_unsafe_comments.sh` currently blocks new
  undocumented unsafe sites against the grandfathered baseline.
- [x] **cargo deny in CI.** License compliance check wired into ci-quick Makefile target.
- [x] **Property tests.** 10 quantizer + 5 page codec properties via proptest. Run on every PR (256 cases).
- [x] **Miri tests.** 11 miri-prefixed tests covering pure-Rust quantizer and page codec paths at small dimensions.
- [x] **Size-of layout assertions.** 13 tests locking down payload sizes, struct sizes, compression ratio bounds.
- [x] **clippy.toml.** cognitive-complexity=30, too-many-arguments=8.

## Owns

- `NFR-004`
- `NFR-005`

## Dependencies

- None — can start immediately

## Unblocks

- Safe parallel development with merge gating
- Repeatable quality checks

## Deliverables

- ~~CI YAML configuration~~ **done**
- ~~Fuzz harness~~ **done**
- Unsafe comment audit report — **remaining under Task 35**
- ~~All gates green~~ **done** (fmt, clippy, test, proptest, layout-check pass)

## Primary Tests

- ~~`TC-035`: fuzz stability~~ **done** (4 fuzz targets)
- [ ] `TC-036`: unsafe comment audit
- ~~CI gates for `NFR-005`~~ **done**

## Notes

- This task ran on a **separate parallel agent** with no coordination required.
- The remaining unsafe audit (TC-036) is now the Task 35 unsafe quality
  burndown. It can proceed independently in reviewed subsystem packets.
