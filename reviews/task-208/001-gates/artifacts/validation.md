# Validation

Head: `dece67f96a05e1b549c5cd384426b39bc101d260`

## Specification validation

Command:

```text
quire validate --scope /Users/peter/dev/tqvector \
  spec/non-functional/NFR-021-distann-distribution-invariant.md
```

Result: exit 0. Quire emitted only existing duplicate catalog-archetype and
inverse-edge notices; the edited NFR emitted no grammar or schema warning.

## Focused tests

Command:

```text
cargo test -p ecaz-cli commands::bench::suite::tests::distann_
```

Result:

```text
running 24 tests
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 465 filtered out
```

The focused set includes:

- pre-registration serialization and result labeling;
- pre-measurement rejection of a nonconforming candidate;
- fixed-roster normalized growth classification;
- unsharded derived-relation negative classification;
- all pre-existing DistANN suite parsing, expansion, topology, provenance, and
  metrics-mode tests.

Two pre-existing variant-expansion assertions were stale after the already
landed candidate-heap/traversal-replica tuple fields. Their expected strings
were updated to the current ten-field CLI encoding; no production behavior was
changed by that test repair.

## Clippy

Command:

```text
cargo clippy -p ecaz-cli --no-deps
```

Result: exit 0. Existing warnings remain in unrelated files and in pre-existing
suite code (`type_complexity` on the raw storage-growth map). The new
registration and conformance code emits no Clippy warning. The build also emits
the existing `clippy.toml` versus `Cargo.toml` MSRV notice.

## Real artifact replay

Commands and source hashes are recorded in `manifest.md`. Both
`bench suite report` invocations exited 0 and wrote normalized JSONL.

Observed decision rows:

```text
task205-owner-control actual=conforming normalized_growth_max=1.094675 raw_growth_max=11.117647 derived_o_n_bytes=0
task205-algorithm1-candidate actual=conforming normalized_growth_max=1.094675 raw_growth_max=11.117647 derived_o_n_bytes=0
task204-replica-negative actual=nonconforming derived_o_n_bytes=1659518976 preregistration_matches=true
```

The Task 204 owner context is intentionally `unavailable` from that replay
because its packet contains only 100k. It is not cited as a conforming result;
the three-scale Task 205 owner lane supplies the positive proof.
