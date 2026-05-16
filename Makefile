.PHONY: fmt fmt-check lint lint-pg17 test pg-test pg-test-pg17 deny audit-unsafe build install clean
.PHONY: bench bench-iai dhat-encode dhat-score proptest layout-check miri
.PHONY: fuzz-parse-text fuzz-unpack fuzz-element-decode fuzz-neighbor-decode
.PHONY: ci-quick ci-nightly spire-multicluster-smoke spire-multicluster-transport-overlap

## Format all source files
fmt:
	cargo fmt --all

## Check formatting without modifying files
fmt-check:
	cargo fmt --all -- --check

## Run Clippy (deny warnings)
lint:
	cargo clippy --all-targets --no-default-features --features pg18,bench -- -D warnings

lint-pg17:
	cargo clippy --all-targets --no-default-features --features pg17,bench -- -D warnings

## Run unit tests (no Postgres required)
test:
	cargo test

## Run pgrx integration tests (requires: cargo pgrx init)
pg-test:
	cargo pgrx test pg18

pg-test-pg17:
	cargo pgrx test pg17

SPIRE_MULTICLUSTER_SMOKE_FLAGS ?= --skip-install
SPIRE_MULTICLUSTER_TRANSPORT_OVERLAP_FLAGS ?= --skip-install

spire-multicluster-smoke:
	scripts/run_spire_multicluster_pg18_smoke.sh $(SPIRE_MULTICLUSTER_SMOKE_FLAGS)

spire-multicluster-transport-overlap:
	bash scripts/run_spire_multicluster_transport_overlap_pg18.sh $(SPIRE_MULTICLUSTER_TRANSPORT_OVERLAP_FLAGS)

## Check dependency licenses
deny:
	cargo deny check licenses

## Verify all unsafe blocks have nearby SAFETY comments
audit-unsafe:
	bash scripts/check_unsafe_comments.sh

## Build release shared library
build:
	cargo build --release

## Install into local Postgres (requires sudo)
install:
	cargo pgrx install --sudo --release

## Remove build artifacts
clean:
	cargo clean

# --- Benchmarks ---

## Run all criterion benchmarks
bench:
	cargo bench --features bench

## Run a specific criterion benchmark (e.g., make bench-quant_score)
bench-%:
	cargo bench --features bench --bench $*

## Run iai-callgrind instruction-count benchmarks (requires valgrind)
bench-iai:
	cargo bench --features bench --bench iai_quant_score
	cargo bench --features bench --bench iai_hadamard
	cargo bench --features bench --bench iai_bitpack

## Generate line-level flamegraph for the quant_score criterion bench.
## Requires `cargo install flamegraph` and Linux `perf` (or DTrace on macOS).
## Output: flamegraph.svg in the repo root.
flamegraph-quant_score:
	cargo flamegraph --features bench --bench quant_score -- --bench
	@echo "Open flamegraph.svg"

## Generate flamegraph for an end-to-end ecaz bench latency run against a
## given corpus prefix. Pass PREFIX=... PROFILE=... ITERATIONS=... K=... .
## Example: make flamegraph-bench PREFIX=ec_hnsw_real_10k PROFILE=ec_ivf
flamegraph-bench:
	cargo flamegraph --bin ecaz -- bench latency \
		--prefix $(PREFIX) --profile $(PROFILE) \
		--k $${K:-10} --iterations $${ITERATIONS:-2000} --concurrency 1
	@echo "Open flamegraph.svg"

## Run dhat heap profiler on encode path
dhat-encode:
	cargo run --release --features bench,dhat-heap --bin dhat_encode
	@echo "Open dhat-heap.json at https://nnethercote.github.io/dh_view/dh_view.html"

## Run dhat heap profiler on score path
dhat-score:
	cargo run --release --features bench,dhat-heap --bin dhat_score
	@echo "Open dhat-heap.json at https://nnethercote.github.io/dh_view/dh_view.html"

# --- Property Testing ---

## Run proptest suite
proptest:
	cargo test --features bench --test proptest_quant --test proptest_page -- --test-threads=1

# --- Layout ---

## Run struct layout and payload size assertions
layout-check:
	cargo test --features bench --test size_of_assertions

# --- Safety ---

## Run Miri on pure-Rust paths
miri:
	cargo +nightly miri test --lib -- miri_

# --- Fuzzing (requires cargo-fuzz + nightly) ---

## Run parse_text fuzzer (10 min)
fuzz-parse-text:
	cd fuzz && cargo +nightly fuzz run fuzz_parse_text -- -max_total_time=600

## Run MSE unpack fuzzer (10 min)
fuzz-unpack:
	cd fuzz && cargo +nightly fuzz run fuzz_unpack_mse -- -max_total_time=600

## Run element tuple decode fuzzer (10 min)
fuzz-element-decode:
	cd fuzz && cargo +nightly fuzz run fuzz_element_tuple_decode -- -max_total_time=600

## Run neighbor tuple decode fuzzer (10 min)
fuzz-neighbor-decode:
	cd fuzz && cargo +nightly fuzz run fuzz_neighbor_tuple_decode -- -max_total_time=600

# --- Recall ---

## Run pure-Rust recall benchmark (slow, ~5 min for 10K vectors)
recall:
	cargo test --features bench --release --test recall_integration -- --ignored --nocapture

# --- SQL Benchmarks (requires running PG with extension installed) ---

bench-sql-latency:
	bash scripts/bench_sql_latency.sh

bench-storage:
	bash scripts/bench_storage.sh

bench-recall-sql:
	python3 scripts/bench_recall.py

# --- CI Aggregates ---

## Quick checks (< 5 min, for every PR)
ci-quick: fmt-check lint test layout-check audit-unsafe

## Full benchmark suite (nightly)
ci-nightly: ci-quick bench bench-iai proptest miri
