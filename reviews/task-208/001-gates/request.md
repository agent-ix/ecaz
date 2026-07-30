# Task 208 review request: mechanical NFR-021/NFR-022 gates

Please review commit `dece67f96a05e1b549c5cd384426b39bc101d260`.

This checkpoint implements Task 208 phases 1-3 and removes the NFR-021
contradiction that blocked Task 172's decision-bearing matrix.

## What changed

- NFR-021 now measures graph-side bytes per owned graph record. Raw
  fixed-roster byte growth remains visible but is not a conformance threshold:
  a valid O(N) shard necessarily grows with corpus cardinality when roster size
  is fixed.
- A `distann-local-multinode` arm can pre-register a stable NFR-021 id, decision
  role, admissibility verdict, and rationale. Scale-specific steps reuse the
  same id.
- Singular and multi-variant arms are supported. If any benchmark variant is
  registered, every variant in that step must be registered.
- A nonconforming control or candidate is rejected during config validation,
  before measurement. Nonconforming context lanes remain measurable.
- Registrations are serialized into `suite-manifest.json`, and every matching
  physical-arm result row carries the id, role, and preregistered verdict.
- The suite derives a `physical_benchmark_nfr_021_conformance` row after all
  selected steps complete. It checks:
  - 10k/50k/100k storage and published-topology coverage;
  - graph-side bytes per owned record, with a 100k/10k threshold of 2.0;
  - zero non-owned graph records and orphan vectors;
  - constant head capacity;
  - zero unsharded O(N) derived-relation bytes.
- Missing evidence is `unavailable`, not a pass. An observed verdict that does
  not match pre-registration fails the suite after writing `results.jsonl`.
- `bench suite report --results-output` regenerates the same derived rows from
  a completed manifest.

## Real-evidence replay

The new gate was replayed over the existing Task 205 three-scale physical-owner
artifacts and Task 204's 100k FR-084 negative fixture.

| arm | raw 100k/10k growth | normalized max | unsharded derived bytes | verdict |
|---|---:|---:|---:|---|
| Task 205 owner control | 11.117647 | 1.094675 | 0 | conforming |
| Task 205 Algorithm 1 candidate | 11.117647 | 1.094675 | 0 | conforming |
| Task 204 coordinator replica | n/a (100k negative) | n/a | 1,659,518,976 | nonconforming |

This is the intended discrimination: the owner arms grow linearly because they
hold their own shards, while the coordinator replica fails because it retains a
complete O(N) derived relation.

The exact derived JSONL rows are in
`artifacts/nfr021-replay-results.jsonl`; provenance and commands are recorded in
`artifacts/manifest.md` and `artifacts/validation.md`.

## Validation

- `quire validate` for NFR-021: passed (catalog duplicate notices only).
- focused DistANN suite tests: 24 passed.
- `cargo clippy -p ecaz-cli --no-deps`: exit 0; existing repository warnings
  remain and are listed in the validation artifact.
- real artifact replay: both owner arms conform; the known replica negative is
  classified nonconforming.

Task 208's retrospective packet sweep remains open for
`reviews/task-208/002-retrospective-sweep/`. This request is limited to the
mechanical gates needed by Task 172.
