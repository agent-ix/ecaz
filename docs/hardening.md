# Hardening Lanes

Task 34 keeps new hardening local-first until each lane is reproducible and
low-noise. The Makefile is the entrypoint; optional tools are checked by
`scripts/hardening.sh` so missing tools fail with setup text instead of cargo
subcommand errors.

Reusable optional-tool setup lives outside the repository:

```sh
bash scripts/install_hardening_tools.sh --all
bash scripts/install_hardening_tools.sh --check
```

The installer keeps upstream source checkouts under `~/.ecaz/hardening-tools`
and puts reusable binaries or shims on the normal user tool path. Use
`--log-file reviews/task-{id}/001-topic/artifacts/name.log` when an install/update log needs to
be attached to a review packet.

## Aggregates

- `make hardening-local` runs stable local checks that do not need a live
  cluster: format check, PG18 Clippy with the current repository lint baseline,
  CLI unit tests, a standalone pure Rust extension harness, property tests,
  SIMD/scalar differential tests, layout assertions, unsafe comment audit, full `cargo-deny`, and
  `cargo-audit`.
- `make hardening-nightly-local` adds slower and toolchain-sensitive local
  lanes: expanded Miri, `cargo-careful`, fuzz smoke, PG fault-injection dry-run
  coverage, Kani, and pure Rust
  ASan/LSan. `cargo-geiger` remains a standalone reporting lane because it can
  force a large clean rebuild.
- `make hardening-validate` checks that local hardening crates import real
  ECAZ `src/` code and that retired synthetic-only lanes have not reappeared.
- `make hardening-tiers-report` prints the current lane tier inventory. See
  `docs/hardening-governance.md` for promotion and demotion rules.

## Baseline

- `make fmt-check`
- `make lint`
- `make lint-hardening`
- `make test`
- `make test-local`
- `make test-hardening-local`
- `make pg-test`
- `make proptest`
- `make simd-diff`
- `make layout-check`
- `make audit-unsafe`
- `make miri`
- `make fuzz-all-short`
- `make fault-full`
- `make deny-full`
- `make bench`
- `make bench-iai`

PG18 remains the primary validation target. PG17 compatibility checks stay
manual unless a change is PG17-facing.

## Supply Chain

- `make cargo-audit`: install with `cargo install cargo-audit`.
- `make deny-full`: install with `cargo install cargo-deny`.
- `make cargo-vet`: install with `cargo install cargo-vet`, then initialize
  `supply-chain/config.toml` with `cargo vet init`.

Promotion plan: `cargo-audit` and full `cargo-deny check` can become PR gates
after local burn-in. `cargo-vet` remains report/manual until third-party audit
imports and criteria are reviewed.

## Unsafe And Static Hygiene

- `make cargo-geiger`: install with `cargo install cargo-geiger`.
- `make mirai`: runs the archived MIRAI analyzer against
  `hardening/careful/` with its pinned nightly toolchain.

Policy: new unsafe blocks need a nearby `SAFETY` comment and, when the unsafe
surface is non-trivial, the review packet should call out why the boundary is
valid. MIRAI stays standalone/manual while its false-positive profile is
unknown for pgrx-heavy code. Rudra and Flux return only when they exercise real
ECAZ code instead of synthetic harness crates.

### Unsafe Quality Burndown

Task 34 introduced `scripts/unsafe_comment_baseline.txt` only as a temporary
grandfathering mechanism so new checks could land without hiding new debt. Task
35 owns burning that baseline down to zero.

Use `make unsafe-baseline-report` before and after each unsafe-burndown packet.
Review packets should cite the before/after counts, include the raw report logs
under packet-local artifacts when making count claims, and explain whether each
covered unsafe was removed, wrapped behind a safer boundary, or documented with
a specific invariant. Baseline growth is a blocker unless the packet calls out
the temporary exception and a reviewer accepts it.

## Miri And Cargo-Careful

- `make miri-expanded`: runs the expanded `miri_` pure-Rust test set through
  the repo hardening script with Miri's default aliasing model.
- `make miri-tree`: reruns the same `miri_` prefix with
  `-Zmiri-tree-borrows` so Tree Borrows and the default model can be compared
  in packet-local evidence.
- `make miri-many-seeds`: reruns the `miri_` prefix with
  `-Zmiri-many-seeds=0..128` by default. Override the range with
  `MIRI_MANY_SEEDS=FROM..TO` when a packet needs a smaller triage run or a
  deeper campaign.
- `make miri-full`: runs the default, Tree Borrows, and many-seeds Miri lanes.
  `hardening-nightly-local` uses this aggregate.
- `make careful`: runs a standalone pure-Rust harness under
  `hardening/careful/` so PostgreSQL callback symbols are kept out of the
  `cargo-careful` test binary. The harness path-lifts the storage page,
  DiskANN tuple/vacuum/Vamana graph, and HNSW search modules and currently
  runs 69 pure tests under cargo-careful.

Miri and Kani cover only pure Rust paths. pgrx, SPI, libpq, PostgreSQL memory
contexts, and C callback entrypoints are outside their model and must stay in
PG18 pgrx or live-cluster lanes.

Review packets for Miri depth work should keep the default, Tree Borrows, and
many-seeds logs separate. If Tree Borrows and the default model disagree, the
packet should identify the exact test, preserve both logs, and classify the
difference as likely UB, model-specific false positive, or a test harness
problem before promoting a new default.

Seeded Miri coverage now includes:

- storage `ItemPointer` and data-page chain behavior,
- DiskANN metadata encode/decode,
- DiskANN Vamana graph search/pruning helpers,
- DiskANN tuple/codebook serialization and vacuum tuple repair helpers,
- HNSW beam-search and visible-frontier traversal helpers,
- SPIRE routing and adaptive nprobe decisions,
- SPIRE top-k candidate dedupe/cursor helpers,
- SPIRE remote coordinator state summaries and remote payload cap validation,
- SPIRE top-graph object serialization,
- SPIRE leaf V2 object metadata and segment invariants through existing
  in-module tests with `miri_` prefixes.

## SIMD/Scalar Differential Validation

`make simd-diff` is the authoritative local lane. It runs the public
`tests/simd_diff.rs` harness and focused in-library differential suites for
RaBitQ arithmetic, `rabitq32`, `qjl32`, `lut32`, `grouped_pq_block`,
`int8_approx32`, the real `hamming32` SIMD implementation, and the
`ec_distann` codec binding. The focused commands are intentional: do not
replace them with an unfiltered `cargo test`. The lane prints the detected
host features and the ISA each family exercised. A host-reachable primary
backend (NEON on aarch64 or AVX2+FMA on x86) returning “unavailable” is a test
failure, not a skip. Production `prod` scoring is covered through the public
forced-hook stage; the focused in-library `prod` stage pins the tiled-LUT
query-shape and lane guards.

Current production inventory:

- `prod` TurboQuant split-score and code-to-code score: scalar,
  AVX2+FMA, and NEON. The `SimdBackend::Avx512` dispatch tier currently enters
  the AVX2/FMA product scorer; there is no distinct AVX-512 product scorer.
- FWHT: scalar, AVX2/FMA, and NEON. `rotation.rs` has no separate
  architecture-specific scoring kernel.
- RaBitQ arithmetic (`rabitq.rs`): NEON bits 1/4/8, AVX2+FMA bits 1/4/8, and
  AVX-512 bits 1/4/8; optional BF16 variants require their explicit cargo
  feature and hardware feature.
- `rabitq32`: bits=1 and multi-bit bits=2/4 full-block and partial kernels on
  AVX2+FMA and NEON. Its SVE module is a NEON-routing placeholder, not an SVE
  implementation.
- `qjl32`: AVX2 and NEON 32-candidate blocks and 8-candidate octets, scalar
  remainders below an octet, plus a real SVE/SVE2 block implementation.
- `lut32`: AVX2 and NEON block/octet/partial/tiled dispatch, plus real
  SVE/SVE2 block and predicated partial implementations.
- `grouped_pq_block`: AVX2 and NEON 32-candidate blocks with padded partial
  dispatch, plus a real SVE/SVE2 block implementation.
- `int8_approx32`: AVX2, NEON, and NEON SDOT/dotprod full-block and partial
  paths. SVE/SVE2 currently route through NEON.
- `hamming32`: NEON XOR/popcount block and partial paths are real SIMD. The
  AVX2 and SVE modules are scalar/NEON routing placeholders and must not be
  reported as distinct SIMD execution.
- HNSW and DiskANN source inner product: scalar, AVX2+FMA, and NEON. DistANN
  exact source scoring intentionally calls the shared DiskANN implementation.

The common `CandidateBatch` binding carries these kernels into the AMs:
HNSW uses TurboQuant LUT/tiled/int8/QJL, grouped-PQ, and RaBitQ bits=1; IVF
uses those families plus RaBitQ bits=2/4; DiskANN uses hamming, grouped-PQ,
TurboQuant LUT, and RaBitQ bits=1; SPIRE uses TurboQuant LUT/QJL and RaBitQ
bits=1; DistANN uses grouped-PQ, TurboQuant LUT, and RaBitQ bits=1. DistANN
has no private SIMD kernel, so its differential test checks direct codec
scoring against prepared/batch scoring, persisted stride slicing, widths
1/7/8/9/16/17/31/32/33, full-block plus tail, and IP-to-distance negation.

Equality contracts:

- Integer accumulators and lookup sums (`lut32`, `int8_approx32`, SDOT, and
  `hamming32`) are bit/integer exact.
- Grouped-PQ is bit-exact because scalar and vector paths retain group-order
  accumulation.
- RaBitQ block and arithmetic paths allow relative/absolute `1e-5`; vector
  FMA/reduction order differs from the scalar anchor.
- QJL candidates in the scalar remainder below an octet are bit-exact. SIMD
  8-candidate octets and 32-candidate blocks allow 4 ULP or relative `1e-6`
  because vector reductions change the accumulation order. The production
  candidate-batch differential covers widths 1/7/8/9/16/17/31/32/33 through
  the real block→octet→scalar cascade.
- FWHT and `prod` split/code-to-code scores allow relative/absolute `1e-5`.
- HNSW/DiskANN source inner product allows relative/absolute `1e-4` because
  the SIMD implementation may fuse multiply-add while scalar does not.

Tolerance changes require a review packet explaining the numeric reason.
Hardware-specific execution claims are host-local: Apple arm64 proves NEON
and, when detected, SDOT; Intel/x86 hosts prove AVX2/AVX-512; Graviton hosts
prove SVE/SVE2. Unavailable paths must be listed as unexecuted rather than
inferred from compilation. Repository CI is manual-dispatch-only; this lane is
not an automatic pull-request or scheduled gate. Until that policy changes,
pre-merge evidence is a task-scoped packet containing a local
`make simd-diff` run from every host class being claimed.

Every new production SIMD scoring path must land with an existing
scalar/reference entry point, a narrow test/bench-only forced-backend hook
when dispatch could hide it, boundary and realistic-dimension differential
fixtures, and a focused command added to `make simd-diff`. Every focused
command has an explicit expected test count; a renamed or empty filter fails
the lane. The Miri scalar
fallback remains useful for reference-path UB checks; it is not SIMD
execution evidence.

## Fuzzing

- `make fuzz-all-short`: runs each libFuzzer target for `FUZZ_SECONDS`, default
  30 seconds. Override without environment prefixes:
  `make fuzz-all-short FUZZ_SECONDS=5`.
- Individual targets: `make fuzz-parse-text`, `make fuzz-unpack`,
  `make fuzz-element-decode`, `make fuzz-neighbor-decode`,
  `make fuzz-diskann-metadata`, `make fuzz-item-pointer`, and
  `make fuzz-vector-normalize`.
- `make afl-decoders`: builds the DiskANN metadata and `ItemPointer` decoder
  targets with AFL.rs for longer manual campaigns.

SQLsmith is live-cluster only:

```sh
make sqlsmith-pg18 SQLSMITH_DSN='postgresql://localhost/postgres'
```

Use a PG18 cluster with `ecaz` installed. Capture crashes and raw SQLsmith logs
under the relevant review packet before citing findings.

### Engine Matrix

Task 46 introduces three orthogonal fuzz engines and a target-shape taxonomy.
Each engine and target shape has a distinct role and cadence; do not
substitute one for another without a packet recording why.

| Engine | Make lane | Cadence | Strengths | Notes |
|---|---|---|---|---|
| libFuzzer (cargo-fuzz) | `make fuzz-all-short`, `make fuzz-*` | per-PR (smoke) + nightly (long) | fast in-process, structure-aware via `arbitrary` | default engine; every target compiles under it |
| Honggfuzz | `make fuzz-honggfuzz` | weekly | persistent-mode, different mutators than libFuzzer | reuses same `fuzz_targets/*.rs`; requires `honggfuzz-rs` + system `honggfuzz` |
| AFL+ (`cargo-afl`) | `make afl-decoders`, `make fuzz-afl` | weekly | forkserver, deterministic stages | already wired for the decoder family; `make fuzz-afl` extends to the structured targets |
| ECAZ-grammar SQLsmith | `make sqlsmith-ecaz SQLSMITH_DSN=…` | nightly | biases toward `<-> `/`<#>` operators, CustomScan plan shapes, REINDEX/VACUUM interleavings | requires live PG18 + ecaz installed; complements upstream `make sqlsmith-pg18` |
| Cross-pollination | `make fuzz-cross-pollinate` | weekly | merges libFuzzer + Honggfuzz + AFL+ corpora | re-runs `make fuzz-corpus-minimize` over the merged corpus before committing |

#### Target shape taxonomy

Fuzz targets fall into two shapes; the choice is per-target and is documented
in the target's module doc-comment:

- **Decoder targets** consume raw bytes by definition — the input *is* bytes
  (`ItemPointer::decode`, `VamanaMetadataPage::decode`, the element/neighbor
  tuple decoders, `fuzz_item_pointer_decode`). Keep these on raw-byte
  `fuzz_target!(|data: &[u8]| ...)`. Wrapping a decoder in `Arbitrary` would
  obscure the bytes-in test it exists to perform.
- **Structured-input targets** consume a logical tuple/record (a
  `(dim, bits, indices)` triple for `unpack_mse_structured`, a
  `(dim, gamma, codes)` triple for `parse_text_structured`, a sorted-list
  pair for `fuzz_topk_merge_structured`). These use
  `#[derive(arbitrary::Arbitrary)]` on an input struct so the fuzzer mutates
  *inside* the valid shape rather than against the structural gates that
  would reject most random bytes. Per Task 46 §Why this trades exec/sec for
  coverage density; target the success path and pair with a separate
  error-path target if the surface has a non-trivial error tree.

#### Corpus management

`fuzz/corpus/` is **committed** (see `.gitignore` notes). Initial commit at
`5d84cedc9` ships the minimized spanning set across all registered targets;
re-minimize after every long campaign:

```sh
make fuzz-corpus-minimize    # cargo fuzz cmin over every registered target
```

`make fuzz-cross-pollinate` runs the multi-engine merge then calls
`fuzz-corpus-minimize` before committing. Cmin is deterministic for a fixed
target binary + input corpus; a no-op re-cmin signals the committed corpus is
already minimal.

`fuzz/target/` and `fuzz/artifacts/` stay gitignored — those are build
products and crash inputs, not curated test material.

#### When to add a new structured target

- New higher-level decoder or codec arrives in `src/` and would otherwise be
  fuzzed via raw bytes against a multi-stage validator.
- Existing raw target spends most of its budget rejecting at the first gate
  (cmin shows < 2× cov-per-corp-entry density delta versus a structured
  sibling).
- A property assertion (round-trip, monotonicity, equivalence) is available
  that the raw target cannot make.

New structured targets must:

1. Live next to their raw sibling under `fuzz/fuzz_targets/`.
2. Use `#[derive(arbitrary::Arbitrary)]` on a small input struct.
3. Land with a matched-run coverage comparison vs the raw sibling in the
   owning review packet (10 s `-max_total_time` is the established budget).
4. Update this engine matrix only if they introduce a new engine or lane.

## Test Quality

Task 39 adds measurement lanes for the quality of existing tests:

- `make coverage`: checks for `cargo-llvm-cov`, runs the local pure-Rust
  hardening subset currently safe outside a live PostgreSQL backend
  (`ecaz-cli` and `hardening/careful`), and writes `summary.txt` plus
  `coverage.json` under `target/quality/coverage`.
- `scripts/check_coverage_delta.sh`: compares `summary.txt` against
  `fixtures/quality/coverage-baseline.tsv`; per-file line coverage may drop at
  most 2 percentage points for changed baseline paths. Baseline raises are
  explicit commits: run the script with `--ratchet` only after inspecting a full
  coverage run, then cite the owning review packet in the TSV note.
- `make coverage-baseline-check`: fails when a critical Task 39 source file is
  missing from `fixtures/quality/coverage-baseline.tsv`.
- `make test-quality-ci-audit`: checks that CI still runs the Task 39 coverage
  lane on PRs, mutation lane weekly/manual, and flake-hunt lane nightly/manual
  with artifact uploads.
- `make coverage-report`: same lane with an HTML report under
  `target/quality/coverage/html`.
- `make mutants MUTANTS_MODULE=src/quant/prod.rs`: checks for
  `cargo-mutants` and runs a bounded mutation campaign for one critical module.
- `make mutants-full`: runs the initial critical-module target list from Task
  39. This is weekly/manual until survivor volume is triaged.
- `make flake-hunt`: re-runs proptest and short fuzz targets across multiple
  seeds. Override with `FLAKE_HUNT_SEEDS=N` and
  `FLAKE_HUNT_FUZZ_SECONDS=N`. The lane writes `manifest.txt` and
  `expanded-commands.txt` under `target/quality/flake-hunt` by default so
  nightly/manual CI runs preserve the exact seed budget and expanded commands.

Current interpretation:

| Lane | Gate Level | Artifact | Rule |
| --- | --- | --- | --- |
| `make coverage` | Report-first / PR candidate after burn-in | summary, JSON, optional HTML | No repository-wide threshold yet; touched production files should not drop by more than 2 percentage points once a baseline packet exists. |
| `make mutants` | Weekly/manual | cargo-mutants output directory plus triage note | Each survivor is triaged as killed by a new test, equivalent, or follow-up bug. |
| `make flake-hunt` | Nightly candidate | seed sweep log | Eight seeds run nightly by default; any nondeterministic failure or new fuzz crash becomes a follow-up with the seed and minimized input attached. |

The first coverage scope intentionally avoids claiming live pgrx callback
coverage. PG18 integration coverage is still a gap until instrumentation is
stable for `cargo pgrx test pg18` and the resulting logs are packeted.

Task 39 packet `reviews/task-39/013-pgrx-coverage-feasibility/` records the
current PG18 instrumentation decision. A probe with
`RUSTFLAGS="-C instrument-coverage"` can build the pgrx test profile far enough
to emit some `.profraw` files, but it does not reach live backend tests on the
current macOS PG18 setup: the lib test binary aborts before execution with
`dyld` failing to resolve `_BufferBlocks`. The coverage runner also needs an
absolute `LLVM_PROFILE_FILE` path; relative paths are resolved from child
process working directories and produce profile-write errors. Until both issues
are fixed and a packet shows merged backend coverage for callback-heavy files,
the supported Task 39 coverage surface is the shim-based subset exercised by
`make coverage`: `ecaz-cli` plus `hardening/careful`.

### Coverage Ratchet

Coverage ratchets are manual and packet-backed. Do not update
`fixtures/quality/coverage-baseline.tsv` just because a local run improved.
The required sequence is:

1. Run the relevant coverage lane and inspect `summary.txt` for the touched
   files.
2. Run `scripts/check_coverage_delta.sh --ratchet` against the same summary
   only after confirming the increase is from intentional tests, not an
   accidental scope change.
3. Update the TSV note with the owning Task 39 packet path.
4. Include the raw coverage run, delta check, and ratchet log in that packet.

The delta gate allows a 2 percentage point drop from the recorded baseline.
That tolerance is for normal measurement noise and small line-count churn, not
for silently accepting untested code paths.

### Coverage Baseline

The versioned baseline lives in `fixtures/quality/coverage-baseline.tsv`.
Per-packet review requests and manifests cite the raw coverage summaries used
to raise individual rows; this policy doc intentionally does not duplicate that
ratchet history or carry a live baseline snapshot.

### Mutation Triage

Mutation packets must include `mutants.txt` or equivalent raw `cargo-mutants`
output plus a `triage.md` table with one row per mutant. Use the Task 39 packet
005 shape as the precedent:

| Column | Meaning |
| --- | --- |
| Mutant | File, line, and transformation. |
| Outcome | `caught`, `unviable`, `missed`, or `timed-out` from the raw run. |
| Verdict | `kill-with-test`, `equivalent`, or `follow-up-bug`. |
| Evidence | Killer test name, equivalence rationale, or follow-up issue/packet target. |

`missed` mutants are not silently acceptable. Either add a test in the packet
that kills the mutant, prove it is equivalent, or file a follow-up bug that
names the target module, mutant description, and why the current packet did not
kill it.

### Cross-Arch Mutation Pattern

SIMD and architecture-dispatched code should expose backend decision points as
small pure functions that can be unit-tested on every host. The intrinsic body
still needs host-specific validation through `make simd-diff`, but the policy
gate that decides whether a backend is eligible should be ordinary Rust logic
with direct tests.

The `src/quant/simd.rs` Task 39 packet 005 pattern is canonical:

- keep runtime CPU feature detection behind a narrow helper,
- make environment/backend override parsing testable without executing the
  intrinsic body,
- extract compound feature gates such as "x86 and AVX2 and FMA" into named
  predicates,
- add tests that kill boolean-gate mutations on ARM and x86 hosts.

When adding new SIMD paths, add this decision-point test shape with the path
rather than relying on a later ARM/x86 mutation sweep to discover the gap.

### Flake-Hunt Seeds

`make flake-hunt` defaults to 8 seeds and short fuzz runs. Nightly runs should
record the seed count, fuzz seconds, and every expanded seed command in
`target/quality/flake-hunt/manifest.txt` and
`target/quality/flake-hunt/expanded-commands.txt`; CI uploads that directory as
the `test-quality-flake-hunt` artifact. A nondeterministic failure packet must include:

- the failing seed,
- the lane and target name,
- the exact rerun command,
- any minimized fuzz input or proptest regression file,
- whether the failure reproduced on a second run with the same seed.

Changing the nightly seed count or fuzz-second budget requires a review packet.
Lowering either value is a demotion unless the packet shows equivalent coverage
through another lane.

## PG Fault Injection

Task 38 tracks five access methods. DistANN expands into three independent
fixtures: `ec_distann/rabitq` (64 dimensions),
`ec_distann/turboquant` (the supported 1536-dimensional no-QJL shape), and
`ec_distann/grouped_pq` (64 dimensions). `ecaz dev fault plan` and every
aggregate smoke lane include all seven fixtures; use
`--am distann --distann-codec <codec>` or the
`fault-distann-{plan,local-smoke}` Make targets for focused work.

Status terms below are evidence-sensitive: **executed-history** means the May
Linux Task 36/38 packet contains a live result; **implemented-current** means
the current runner has a real fixture/operator path but this branch still needs
live evidence; **unavailable-host** means the path requires Linux facilities
absent on the current macOS arm64 host; and **nonexistent** means the production
feature itself does not exist.

| Access method / fixture | Build, scan, insert, delete-vacuum, DDL | Cancel, terminate, statement/idle/lock timeout | palloc/process memory; I/O; WAL/temp | Local / remote transport | Cleanup and evidence |
| --- | --- | --- | --- | --- | --- |
| `ec_hnsw` | Real AM lifecycle and HNSW-specific DDL | All generic interrupt/timeout probes | palloc, RLIMIT/SIGKILL proxy, matched EIO/ENOSPC/slow disk, accumulator and WAL/temp accounting | Local | Shared probes; **executed-history** |
| `ec_ivf` | Real AM lifecycle and IVF-specific DDL | All generic probes | Same resource/provider inventory | Local | Shared probes; **executed-history** |
| `ec_diskann` | Real AM lifecycle and DiskANN traversal | All generic probes | Same resource/provider inventory | Local | Shared probes; **executed-history** |
| `ec_spire` | Real local lifecycle; Stage E fixtures are separate | Generic local probes plus Stage E fault cases | Same local inventory | Real remote SQL transport exists over libpq/Unix sockets; exact-peer reset/slow is **implemented-current**. Object-store reads are **nonexistent** | Local probes **executed-history**; live socket provider **unavailable-host** here |
| `ec_distann/rabitq` | Real 64-D physical build, traversal/owner scoring/payload, insert/tombstone, vacuum and DDL | Repeated real KNN work for all probes | Codec-specific palloc and local relation provider/resource lanes | Real owner/payload libpq loopback transport | **implemented-current**; live evidence pending |
| `ec_distann/turboquant` | Real supported 1536-D no-QJL lifecycle | Same, inside TurboQuant batch scoring | Distinct fixture paths and markers | Same DistANN transport | **implemented-current**; live evidence pending |
| `ec_distann/grouped_pq` | Real grouped-PQ lifecycle. An operation requiring an absent future codebook-rehydration path must emit a supported skip | Same, inside grouped-PQ batch scoring | Distinct fixture paths and markers | Same DistANN transport | **implemented-current**; live evidence pending |

Every live lane uses one index per fixture table and prints AM, codec, phase,
and fault markers. Shared postconditions check surviving `ecaz-fault-*`
sessions, relation/advisory locks, prepared transactions, optional
`pg_buffercache` fixture pins, and readable/non-decreasing `pg_stat_io` and
`pg_stat_wal` counters. Provider cases additionally require a matching
`fault=1` event; provider-load/configuration markers alone cannot pass.

- `ecaz dev fault plan`: prints the required Task 38 fault matrix for every
  ECAZ AM and every lane.
- `make fault-io-smoke`, `make fault-mem-smoke`, `make fault-cancel-smoke`,
  `make fault-timeout-smoke`, `make fault-lock-smoke`,
  `make fault-resource-smoke`, and `make fault-slow-disk-smoke`: run the
  operator smoke entry points. They default to `FAULT_SMOKE_FLAGS=--dry-run` so
  local hardening can verify matrix coverage without a live injection
  provider. This is not a CI or nightly execution claim.
- To run a live probe, clear the dry-run flag, for example:
  `make fault-timeout-smoke FAULT_SMOKE_FLAGS=`.
- `ecaz dev fault provider-env` prints the LD_PRELOAD environment for the
  built-in Linux provider. That provider can inject matched-path `EIO` reads,
  matched-path `ENOSPC` writes/creates/fsyncs, slow-disk latency, and
  exact-peer socket reset/latency faults once the PG postmaster is started
  with the printed environment. Socket peers use `tcp:HOST:PORT`,
  `tcp:[IPv6]:PORT`, or an absolute named `unix:/path`; socket modes reject a
  missing or unstable identity. Unnamed and abstract `AF_UNIX` peers never
  match because they do not have a stable pathname.
- `--arm-file <absolute-path>` starts the provider disarmed and makes file
  existence the injection gate. This lets a long-lived coordinator complete
  real SPIRE/DistANN topology setup before the operator creates the arm file
  for the targeted remote query; removing the file disarms subsequent traffic
  so recovery can be checked without another postmaster restart.
- `ecaz dev fault provider-restart` and `ecaz dev fault provider-restore`
  wrap the local pgrx `pg_ctl restart` step so provider-backed lanes do not
  require hand-assembled `LD_PRELOAD` commands. Marker paths passed to
  `provider-restart` are made absolute before they are exported to the
  postmaster, so backend workers append to the same marker even after
  PostgreSQL changes its working directory.
- `ecaz dev fault prepare --rows N` creates the AM-specific fixtures before
  destructive provider modes are enabled. Live I/O smoke then runs with
  `--assume-prepared --provider-marker <marker>` against an `eio-read` or
  `enospc-write` provider-backed postmaster.
  Provider-backed smoke lanes require the same marker path via
  `--provider-marker` so they cannot pass against a normal postmaster.
- Live memory smoke uses the extension GUC `ecaz.fault_palloc_nth` and
  `ecaz_fault_reset_palloc_counter()` to raise a clean ERROR at instrumented
  AM memory-fault boundaries. The current smoke covers each AM's build,
  insert, and vacuum callback boundary, and sweeps the first few Nth allocation
  points for each AM scan workload. The runner attempts to disable the GUC and
  reset its counter after every workload result, including unexpected errors;
  a workload error plus reset error is a hard failure.

The current live CLI smoke creates AM-specific fixtures for `ec_hnsw`, `ec_ivf`,
`ec_diskann`, `ec_spire`, and all three `ec_distann` codec shapes, then directly exercises cancellation and
backend termination with repeated AM KNN scans, statement timeout with repeated
AM KNN scans, `idle_in_transaction_session_timeout` after each AM fixture is
touched inside an open transaction, lock timeout with
`REINDEX INDEX CONCURRENTLY`, `CREATE INDEX`, and `VACUUM (FULL)`, and
scan/insert/vacuum/resource settings on those fixtures.
Slow-disk runs the same AM-specific scan/insert/vacuum smoke against a
provider-backed postmaster, requires a `fault=1` marker, and requires
`--slow-disk-baseline-ms` measured from the matched provider-off workload.
The lane prints and asserts provider elapsed time is greater than baseline. I/O smoke
uses prebuilt fixtures and checks one provider mode at a time: `eio-read`
expects clean ERROR from AM scan reads, while `enospc-write` expects clean
ERROR from AM writes. When the provider marker records `match=pg_wal`, the I/O
lane treats WAL-path ENOSPC as a crash-recovery surface: it records the backend
disconnect, prints `wal_enospc_provider_restore_required=true`, and expects the
operator to run `ecaz dev fault provider-restore`, whose fallback path performs
an immediate stop/start if fast restart cannot shut down the faulting
postmaster. Resource smoke prepares pressure-sized AM fixtures, runs high-limit
KNN scans under `work_mem = '64kB'` and `effective_cache_size = '1MB'`, emits
`resource_accumulator_pressure` markers with the prepared row count, requested
limit, actual returned high-water, and returned fraction. The gate requires at
least 95% of the requested pressure target so approximate AMs remain valid
without falling back to the weak historical `count >= 64` assertion. It then runs AM scan/insert/vacuum under tiny
`work_mem`/`maintenance_work_mem` settings and forces a temp-spill failure with
`temp_file_limit = '64kB'`, verifying the backend remains usable. When the
postmaster is restarted with an `enospc-write` provider whose marker records
`match=pgsql_tmp`, the resource lane instead disables `temp_file_limit` and
expects the temp-spill failure to come from provider-backed ENOSPC. The
provider appends `fault=1` marker lines with mode, operation, errno, count, and
target path when it actually injects EIO/ENOSPC, and provider-backed smoke
asserts `provider_fault_events ... count>0` for the configured match. The same
resource lane now performs AM-backed writes, forces `pg_switch_wal()`, and
emits `wal_rotation_accounting` markers proving WAL LSN advancement plus
non-decreasing `pg_stat_wal` record/byte counters after stats flush. Memory smoke
injects palloc failures at the
instrumented AM build/scan/insert/vacuum boundaries. Build, scan, insert, and
vacuum probes sweep `ecaz.fault_palloc_nth` from 1 through the smoke cap and
stop at the first successful Nth value, emitting `memory_palloc_sweep_fault`
and `memory_palloc_sweep_completed` markers so the log shows how many currently
instrumented palloc boundaries were covered. The lane verifies the backend
remains usable after each ERROR. Every lane
uses the shared post-condition probe inventory from `ecaz-fault-injection`:
leftover fault sessions, surviving locks, prepared transactions, optional
`pg_buffercache` fixture pin counts, optional `pg_stat_io` non-decreasing
operation counters, and optional `pg_stat_wal` non-decreasing WAL record/byte
counters. Resource temp-spill probes also print
`resource_temp_spill_accounting` markers from `pg_stat_database.temp_bytes` for
readable before/after accounting; temp-file-limit failures may abort before the
database temp-byte total advances, so the smoke asserts readability and
non-decreasing totals rather than byte-perfect attribution. Memory smoke also
caps a warmed backend's `RLIMIT_AS` during AM build work, expecting an
OOM-class ERROR or backend disconnect followed by a usable postmaster, then
SIGKILLs worker backends during AM build/scan/insert as an OOM-kill proxy and
waits for postmaster recovery. The 25 ms delay is probability tuning, not
proof that SIGKILL landed inside an AM critical section. Those subcases are crash-recovery checks; lower
post-run `pg_stat_io` or `pg_stat_wal` totals are recorded as stats resets
after recovery rather than treated as monotonicity failures.

SPIRE Stage E SQLSTATE faults reuse
`ecaz dev spire-multicluster fault-pg18`; this is distinct from provider-level
socket faults. SPIRE loopback remote SQL uses Unix sockets, while DistANN
multicluster owner/payload SQL uses loopback TCP. On Linux, start only the
coordinator with the exact named-Unix or TCP peer filter, require a reset/slow
`fault=1` marker, restore the provider, and run the shared postconditions.
`make fault-distann-remote-socket-smoke` automates that sequence for a real
two-owner physical DistANN fixture. The coordinator starts with the provider
disarmed, proves the baseline remote owner query, creates the arm file for one
exact-peer query, requires the reset/error or measured delay plus a matching
`fault=1` marker, removes the arm file, and requires the next remote-owner
query to recover successfully. Set `FAULT_SOCKET_PROVIDER_MODE` to
`socket-reset` or `socket-slow`.
`make fault-spire-remote-socket-smoke` applies the same armed sequence to the
native one-coordinator/three-worker SPIRE fixture. Only the coordinator loads
the provider; the exact peer is remote worker 1's named Unix socket. The probe
runs the production read profile before, during, and after the fault, requires
the exact-peer marker, accepts SPIRE's documented clean degraded result or
clean ERROR for reset, and requires successful recovery after disarm.
Unnamed and abstract Unix peers are deliberately non-matchable. This macOS
host cannot load the Linux provider, so no live socket result is claimed. SPIRE
object-store reads remain **nonexistent**, not an unavailable transport test.

`ecaz dev fault cgroup-plan` reports Linux, cgroup-v2, and `systemd-run`
capability and prints the isolated one-index-per-table MemoryMax plan. A live
run uses `make fault-cgroup-smoke` to place a fresh isolated PG18 postmaster,
the selected AM workload, and a resident-memory pressure task in one user
`systemd-run --scope`. Each of the seven fixtures runs separately with
`MemoryMax` and `OOMPolicy=kill`. The outer operator requires systemd
`Result=oom-kill` after the repeated real AM-build marker, restarts the killed
cluster outside the scope, verifies SQL usability and zero invalid ECAZ
indexes, and stops it cleanly. Scope and recovery logs land below
`FAULT_CGROUP_ARTIFACT_DIR`; transient data directories live below
`FAULT_CGROUP_RUNTIME_DIR` and are removed only after successful recovery, so
PostgreSQL data files cannot accidentally enter a review packet. Direct
`/sys/fs/cgroup` writes are forbidden. The current macOS host reports this lane
unavailable and cannot supply live evidence.

Provider ENOSPC can surface PostgreSQL checkpoint failures as `XX000`. The
allowance is restricted to messages containing `checkpoint request failed` or
`No space left on device`; arbitrary internal errors still fail.

Current interrupt inventory:

- DiskANN build/scan paths call `maybe_check_for_interrupts()` from
  `src/am/ec_diskann/mod.rs`, including the scan loop and build/import loops in
  `src/am/ec_diskann/scan.rs` and `src/am/ec_diskann/routine.rs`.
- SPIRE remote candidate dispatch polls PostgreSQL interrupt and statement
  timeout flags in `src/am/ec_spire/coordinator/remote_candidates/dispatch.rs`.
- DistANN build and scan paths poll around physical generation, traversal,
  owner scoring, and payload work. The codec fixtures keep interrupt workloads
  in repeated real scoring rather than substituting `pg_sleep`.
- HNSW parallel build calls `pg_sys::ProcessInterrupts()` in
  `src/am/ec_hnsw/build_parallel.rs`.

Missing or newly discovered long-running loops should be added to this list
with either an interrupt check or a follow-up task.

## Concurrency And Formal Pilots

- `make loom-real`: runs the standalone Loom harness in `hardening/loom/`.
  The harness path-lifts real ECAZ code instead of reimplementing a synthetic
  example. Current coverage targets the parallel scan worker-slot state machine
  from `src/am/common/parallel_slot.rs`: exclusive claim, live claim count,
  publish/release interleavings, and stale-epoch rejection. New Loom targets
  should first lift the production protocol into a pgrx-free helper and then
  model that helper from `hardening/loom/`.
- `make shuttle-real`: runs the standalone Shuttle harness in
  `hardening/shuttle/`. Current coverage targets SPIRE remote candidate merge
  order invariance and epoch-publish visibility using path-lifted helpers under
  `src/am/ec_spire/`.
- `make sim-spire-remote`: runs the standalone Turmoil-based SPIRE remote
  simulation in `hardening/sim-spire/`. Current coverage drives path-lifted
  remote transport request/response state through deterministic UDP delivery,
  network partition handling, degraded skip behavior, stale served-epoch
  rejection, and candidate merge selection. Set `SIM_SPIRE_SEEDS=N` to sweep
  multiple Turmoil seeds per network scenario while keeping the default local
  lane at one seed.
- `make sim-spire-remote-deep`: pre-release / post-refactor budget knob for the
  same simulation lane. It currently runs with
  `SIM_SPIRE_REMOTE_DEEP_SEEDS=1000` by default and is intentionally not part
  of the standard local hardening rollup.
- `make kani`: bounded proof for `ItemPointer` decode length behavior.

Kani is intentionally separate from normal `cargo test` so the repo does not
acquire heavyweight model-checking dependencies on the default path. Flux
remains deferred until Task 44 can point it at real ECAZ invariants.

## Sanitizers

Pure Rust:

- `make sanitizer-asan`
- `make sanitizer-lsan`
- `make sanitizer-tsan`
- `make sanitizer-msan`

PG18/pgrx:

- `make sanitizer-pg18-asan`
- `make sanitizer-pg18-tsan`

Sanitizer runs require nightly Rust and platform support. PG18 sanitizer lanes
also require a pgrx-ready cluster; keep them nightly/manual until the cluster
setup is stable.
