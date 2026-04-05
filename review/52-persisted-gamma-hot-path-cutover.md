# Review Request: Persisted Gamma Hot-Path Cutover

Scope:
- `src/am/mod.rs`
- `src/am/scan.rs`
- `src/quant/prod.rs`

What changed:
- Switched scan scoring to use persisted `(gamma, code_bytes)` directly from the index element tuple instead of fetching a representative heap row.
- Switched duplicate lookup to compare persisted element gamma directly instead of reading gamma from the representative heap tuple.
- Added `ProdQuantizer::score_ip_from_parts(prepared, gamma, code_bytes)` so the scan path can score without rebuilding a synthetic `[gamma][code_bytes]` payload buffer.
- Added focused quantizer coverage that the parts-based scorer matches the encoded-payload scorer and that the supplied gamma term actually influences the score.

Review focus:
- Whether the scan and duplicate-detection hot paths are now fully off the representative-heap gamma dependency
- Whether the new parts-based scorer is the right stable seam for future ordered traversal work
- Whether any remaining path still consults heap-derived gamma unnecessarily in scan or live insert

Questions to answer:
- Is the removal of representative-heap gamma reads complete for scan scoring and duplicate lookup?
- Is `score_ip_from_parts` the right long-term scorer API for candidate traversal work?
- Are there any remaining correctness or lifecycle edges around persisted gamma usage that should be covered before candidate-heap work starts?
