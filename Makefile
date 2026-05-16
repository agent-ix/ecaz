.PHONY: fmt fmt-check lint lint-pg17 lint-hardening test test-hardening-local pg-test pg-test-pg17 deny deny-full audit cargo-audit cargo-vet audit-unsafe cargo-geiger rudra mirai flux build install clean
.PHONY: bench bench-iai dhat-encode dhat-score proptest layout-check miri miri-expanded careful
.PHONY: fuzz-parse-text fuzz-unpack fuzz-element-decode fuzz-neighbor-decode fuzz-diskann-metadata fuzz-item-pointer fuzz-vector-normalize fuzz-all-short afl-decoders
.PHONY: kani loom shuttle sanitizer-asan sanitizer-lsan sanitizer-tsan sanitizer-msan sanitizer-pg18-asan sanitizer-pg18-tsan sqlsmith-pg18
.PHONY: hardening-local hardening-nightly-local
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

CLIPPY_HARDENING_BASELINE_ALLOW = \
	-A unknown-lints \
	-A unused-imports \
	-A clippy::clone-on-copy \
	-A clippy::derivable-impls \
	-A clippy::enum-variant-names \
	-A clippy::field-reassign-with-default \
	-A clippy::if-same-then-else \
	-A clippy::int-plus-one \
	-A clippy::manual-contains \
	-A clippy::manual-range-contains \
	-A clippy::needless-lifetimes \
	-A clippy::needless-return \
	-A clippy::op-ref \
	-A clippy::question-mark \
	-A clippy::redundant-closure-call \
	-A clippy::too-many-arguments \
	-A clippy::type-complexity \
	-A clippy::unnecessary-cast \
	-A clippy::unnecessary-sort-by \
	-A clippy::useless-conversion \
	-A clippy::useless-format \
	-A clippy::useless-vec \
	-A clippy::vec-init-then-push

lint-hardening:
	cargo clippy --all-targets --no-default-features --features pg18,bench -- -D warnings $(CLIPPY_HARDENING_BASELINE_ALLOW)

## Run unit tests (no Postgres required)
test: test-hardening-local

## Run local unit lanes that avoid pgrx callback symbol loading
test-hardening-local:
	cargo test -p ecaz-cli
	cargo test --manifest-path hardening/careful/Cargo.toml --lib

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

deny-full:
	bash scripts/hardening.sh cargo-deny-full

cargo-audit:
	bash scripts/hardening.sh cargo-audit

audit: cargo-audit deny-full

cargo-vet:
	bash scripts/hardening.sh cargo-vet

## Verify all unsafe blocks have nearby SAFETY comments
audit-unsafe:
	bash scripts/check_unsafe_comments.sh

cargo-geiger:
	bash scripts/hardening.sh cargo-geiger

RUDRA_MANIFEST ?= hardening/rudra/Cargo.toml
RUDRA_FLAGS ?= --manifest-path $(RUDRA_MANIFEST)

rudra:
	bash scripts/hardening.sh rudra $(RUDRA_FLAGS)

mirai:
	bash scripts/hardening.sh mirai

flux:
	bash scripts/hardening.sh flux

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

miri-expanded:
	bash scripts/hardening.sh miri-expanded

careful:
	bash scripts/hardening.sh cargo-careful

# --- Fuzzing (requires cargo-fuzz + nightly) ---

FUZZ_SECONDS ?= 30

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

fuzz-diskann-metadata:
	cd fuzz && cargo +nightly fuzz run fuzz_diskann_metadata_decode -- -max_total_time=600

fuzz-item-pointer:
	cd fuzz && cargo +nightly fuzz run fuzz_item_pointer_decode -- -max_total_time=600

fuzz-vector-normalize:
	cd fuzz && cargo +nightly fuzz run fuzz_vector_normalize -- -max_total_time=600

fuzz-all-short:
	bash scripts/hardening.sh fuzz-all-short --seconds $(FUZZ_SECONDS)

afl-decoders:
	bash scripts/hardening.sh afl-decoders

# --- Formal / concurrency pilots ---

kani:
	bash scripts/hardening.sh kani

loom:
	bash scripts/hardening.sh loom

shuttle:
	bash scripts/hardening.sh shuttle

# --- Sanitizers / live-cluster hardening ---

SQLSMITH_DSN ?=
SQLSMITH_FLAGS ?= $(if $(SQLSMITH_DSN),--dsn $(SQLSMITH_DSN),)

sanitizer-asan:
	bash scripts/hardening.sh sanitizer-asan

sanitizer-lsan:
	bash scripts/hardening.sh sanitizer-lsan

sanitizer-tsan:
	bash scripts/hardening.sh sanitizer-tsan

sanitizer-msan:
	bash scripts/hardening.sh sanitizer-msan

sanitizer-pg18-asan:
	bash scripts/hardening.sh sanitizer-pg18-asan

sanitizer-pg18-tsan:
	bash scripts/hardening.sh sanitizer-pg18-tsan

sqlsmith-pg18:
	bash scripts/hardening.sh sqlsmith-pg18 $(SQLSMITH_FLAGS)

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

# --- Hardening aggregates ---

hardening-local: fmt-check lint-hardening test-hardening-local proptest layout-check audit-unsafe deny-full cargo-audit

hardening-nightly-local: hardening-local miri-expanded careful fuzz-all-short kani loom shuttle sanitizer-asan sanitizer-lsan
