# Review Request: Task 38 DistANN Fault Model Foundation

## Summary

This first Task 38 checkpoint adds `ec_distann` to the fault model without
aliasing it to `ec_diskann`. The model now has seven concrete fixtures:
`ec_hnsw`, `ec_ivf`, `ec_diskann`, `ec_spire`, and codec-specific
`ec_distann` fixtures for RaBitQ, TurboQuant, and grouped PQ.

`ecaz dev fault plan`, `prepare`, and `smoke` accept `--am distann` and an
optional `--distann-codec`. An unqualified DistANN selection expands to all
three codecs; aggregate selection expands to the four existing fixtures plus
all three DistANN fixtures. Each DistANN fixture has a distinct table/index
name and uses real local `ec_distann` DDL with the selected
`neighbor_code_format`.

The codec fixtures use dimensions supported by their real scorers:
TurboQuant uses the 1536-D no-QJL 4-bit lane, while RaBitQ and grouped PQ use
64-D deterministic vectors. DistANN now also participates in the existing
test-controlled palloc sweep at build, scan, insert, bulk-delete, and vacuum
callback boundaries.

This remains a pre-live checkpoint. It does not yet request review of live
cancellation, timeout, lifecycle, provider I/O, or remote transport evidence.
Those follow in later code checkpoints within this same packet.

## Historical Context

The prior four-AM smoke implementation and reviewer cycles remain in
`reviews/task-36/001-31145-task36-38-hardening-validation/`. In particular,
the final review and its corrected split between live SPIRE SQL transport and
nonexistent object-store reads are background for the next socket-provider
slice.

## Validation

See `artifacts/manifest.md` and the packet-local logs:

- `cargo test -p ecaz-fault-injection`
- `cargo check -p ecaz-cli`
- focused `ecaz-cli` DistANN fault parsing test
- focused DistANN cancel-plan dry run
- focused grouped-PQ timeout dry run

## Reviewer Focus

- Does the seven-fixture model make DistANN codec coverage explicit enough to
  prevent silent single-codec passes?
- Are the codec-specific relation names and local `ec_distann` DDL an
  appropriate foundation for the live fault lanes?
- Are 1536-D TurboQuant and 64-D RaBitQ/grouped-PQ deterministic fixtures the
  right small supported dimensions for interruption smoke?
- Are the DistANN palloc hooks narrow and safely disabled by the existing
  production-default-off GUC?
- Is rejecting `--distann-codec` outside `--am distann` the right operator
  contract?
