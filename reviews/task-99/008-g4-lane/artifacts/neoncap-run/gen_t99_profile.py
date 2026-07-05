#!/usr/bin/env python3
"""Generate the Task 99 index x quant x mode profile SuiteConfig.

Emits task99-profile-suite.json next to this script. The generator is the
reviewable source of truth for the cell matrix; the JSON is its output.
Run: python3 gen_t99_profile.py
"""
import json
import os

ART = "reviews/task-99/002-profile-suiteconfig/artifacts"

steps = []


def recall(name, tags, prefix, profile, sweep, gucs=None, ivf_soa=None,
           queries_limit=100):
    s = {
        "kind": "recall", "name": f"recall-{name}", "tags": tags,
        "prefix": prefix, "profile": profile, "k": 10, "sweep": sweep,
        "queries_limit": queries_limit,
        "truth_cache_dir": "${artifact_dir}/truth-cache",
        "log_output": "${artifact_dir}/recall-" + name + ".log",
    }
    if gucs:
        s["session_gucs"] = gucs
    if ivf_soa is not None:
        s["ivf_scratch_soa_batch_decode"] = ivf_soa
    return s


def latency(name, tags, prefix, profile, sweep, gucs=None, ivf_soa=None):
    s = {
        "kind": "latency", "name": f"latency-{name}", "tags": tags,
        "prefix": prefix, "profile": profile, "k": 10, "sweep": sweep,
        "iterations": 300, "concurrency": 1,
        "cache_state": f"t99_{name.replace('-', '_')}",
        "task87_candidate_batch_counters": True,
        "log_output": "${artifact_dir}/latency-" + name + ".log",
    }
    if gucs:
        s["session_gucs"] = gucs
    if ivf_soa is not None:
        s["ivf_scratch_soa_batch_decode"] = ivf_soa
    return s


def cell(name, tags, prefix, profile, sweep, gucs=None, ivf_soa=None,
         recall_too=True):
    out = []
    if recall_too:
        out.append(recall(name, tags, prefix, profile, sweep, gucs, ivf_soa))
    out.append(latency(name, tags, prefix, profile, sweep, gucs, ivf_soa))
    return out


def storage(prefix, tags):
    return {
        "kind": "storage", "name": f"storage-{prefix}",
        "tags": tags + ["storage"], "prefix": prefix,
        "log_file": "${artifact_dir}/storage-" + prefix + ".log",
    }


T99 = "task99"

# --------------------------------------------------------- QJL fixtures
# Synthetic 10k x 1024-dim (QJL lane only exists off the 1536 tile).
# Seeds 9901/9902; suite-driven so the fixture is reproducible per lane.
for kind, n, seed in (("corpus", 10000, 9901), ("queries", 100, 9902)):
    steps.append({
        "kind": "raw", "name": f"generate-qjl-1024-{kind}",
        "tags": [T99, "fixture", "generate", kind],
        "args": ["corpus", "generate",
                 "--output", f"{ART}/t99_qjl_1024_{kind}.tsv",
                 "--n", str(n), "--dim", "1024", "--seed", str(seed),
                 "--kind", kind],
        "expected_artifacts": [f"{ART}/t99_qjl_1024_{kind}.tsv"],
    })

for am, profile, extra in (
    ("hnsw", "ec_hnsw", {"m": [16], "ef_construction": 128}),
    ("ivf", "ec_ivf", {}),
    ("spire", "ec_spire", {}),
):
    load = {
        "kind": "load", "name": f"load-qjl-{am}-1024",
        "tags": [T99, "fixture", am, "turboquant_qjl", "dim1024"],
        "prefix": f"t99_qjl_{am}_1024", "profile": profile,
        "bits": 4, "dim": 1024,
        "corpus_file": f"{ART}/t99_qjl_1024_corpus.tsv",
        "queries_file": f"{ART}/t99_qjl_1024_queries.tsv",
        "log_file": "${artifact_dir}/load-qjl-" + am + "-1024.log",
    }
    load.update(extra)
    steps.append(load)

# ------------------------------------------------------------ HNSW @100k
HP = "t99_hnsw_tq_100k"
HS = [80, 160]
ISO = "ec_hnsw.disable_binary_prefilter=on"
BOFF = "ec_hnsw.candidate_batch_scoring=off"


# NOTE on markers: `kernel_status=` tags are SKIP directives to the suite
# runner (except the runnable `retired`). Real surfaces we want measured
# as no-kernel baselines carry plain `no_kernel_*` tags instead; the
# aggregate matrix documents their kernel absence.
def hnsw_mode_tags(mode, state, status="valid"):
    t = [T99, "hnsw", "turboquant", "real100k", f"mode={mode}",
         f"kernel_{state}"]
    if status == "retired":
        t.append("kernel_status=retired")
    elif status != "valid":
        t.append(status)
    return t


# Exact-mode isolation cells (prefilter disabled, Task 101/103 precedent)
for mode, status, both in (
    ("full_lut", "valid", True),
    ("int8_approx", "valid", True),
    ("exact", "no_kernel_baseline", True),    # runnable f32 baseline mode
    ("tiled_lut", "retired", False),          # runnable retired confirmation
):
    g_on = [ISO, f"ec_hnsw.turboquant_exact_score_mode={mode}"]
    steps += cell(f"hnsw-tq-{mode}-on", hnsw_mode_tags(mode, "on", status),
                  HP, "ec_hnsw", HS, gucs=g_on)
    if both:
        steps += cell(f"hnsw-tq-{mode}-off",
                      hnsw_mode_tags(mode, "off", status),
                      HP, "ec_hnsw", HS, gucs=g_on + [BOFF])

# Default production path (binary prefilter active, default mode)
steps += cell("hnsw-tq-default-on",
              [T99, "hnsw", "turboquant", "real100k", "default_path",
               "kernel_on"], HP, "ec_hnsw", HS)
steps += cell("hnsw-tq-default-off",
              [T99, "hnsw", "turboquant", "real100k", "default_path",
               "kernel_off"], HP, "ec_hnsw", HS, gucs=[BOFF])

# HNSW RaBitQ (bits-1 sidecar lane)
for state, gucs in (("on", None), ("off", [BOFF])):
    steps += cell(f"hnsw-rabitq-{state}",
                  [T99, "hnsw", "rabitq", "real100k", f"kernel_{state}"],
                  "t99_hnsw_rabitq_100k", "ec_hnsw", HS, gucs=gucs)

# HNSW QJL @1024
for state, gucs in (("on", None), ("off", [BOFF])):
    steps += cell(f"hnsw-qjl-1024-{state}",
                  [T99, "hnsw", "turboquant_qjl", "dim1024",
                   f"kernel_{state}"],
                  "t99_qjl_hnsw_1024", "ec_hnsw", [32, 80], gucs=gucs)

# ------------------------------------------------------------- IVF @100k
IS = [16, 64]
for quant, prefix, states, status in (
    ("turboquant", "t99_ivf_tq_100k", ("on", "off"), "valid"),
    ("rabitq1", "t99_ivf_rabitq1_100k", ("on", "off"), "valid"),
    # Real storage lane, no block kernel (Task 93 bits=1 scope) —
    # plain tag so the runner executes it as a baseline.
    ("rabitq4", "t99_ivf_rabitq4_100k", ("on",), "no_kernel_storage_lane"),
    ("pq_fastscan", "t99_ivf_pqfs_100k", ("on", "off"), "valid"),
):
    for state in states:
        tags = [T99, "ivf", quant, "real100k", f"kernel_{state}"]
        if status != "valid":
            tags.append(status)
        steps += cell(f"ivf-{quant}-{state}", tags, prefix, "ec_ivf", IS,
                      ivf_soa=(state == "on"))

# IVF QJL @1024
for state in ("on", "off"):
    steps += cell(f"ivf-qjl-1024-{state}",
                  [T99, "ivf", "turboquant_qjl", "dim1024",
                   f"kernel_{state}"],
                  "t99_qjl_ivf_1024", "ec_ivf", [8, 16],
                  ivf_soa=(state == "on"))

# ----------------------------------------------------------- SPIRE @100k
SOFF = "ec_spire.candidate_batch_scoring=off"
for quant, prefix in (("turboquant", "t99_spire_tq_100k"),
                      ("rabitq", "t99_spire_rabitq_100k")):
    for state, gucs in (("on", None), ("off", [SOFF])):
        tags = [T99, "spire", quant, "real100k", f"kernel_{state}"]
        if quant == "rabitq":
            # M5 finding: counters not batch-attributed on this surface;
            # this cell re-checks attribution on the production lanes.
            tags.append("attribution_check")
        steps += cell(f"spire-{quant}-{state}", tags, prefix, "ec_spire",
                      IS, gucs=gucs)

# SPIRE QJL @1024
for state, gucs in (("on", None), ("off", [SOFF])):
    steps += cell(f"spire-qjl-1024-{state}",
                  [T99, "spire", "turboquant_qjl", "dim1024",
                   f"kernel_{state}"],
                  "t99_qjl_spire_1024", "ec_spire", [8, 16], gucs=gucs)

# --------------------------------------------------------- DiskANN @100k
DS = [64, 128]
DOFF = "ec_diskann.candidate_batch_scoring=off"
for name, prefix, quant, base_gucs in (
    ("diskann-pqfs-binary", "t99_diskann_pqfs_100k", "binary",
     ["ec_diskann.prefilter_kind=binary_sidecar"]),
    ("diskann-pqfs-grouped-pq", "t99_diskann_pqfs_100k", "grouped_pq",
     ["ec_diskann.prefilter_kind=grouped_pq"]),
    ("diskann-rabitq", "t99_diskann_rabitq_100k", "rabitq", []),
    ("diskann-tq", "t99_diskann_tq_100k", "turboquant", []),
):
    for state in ("on", "off"):
        gucs = list(base_gucs) + ([DOFF] if state == "off" else [])
        steps += cell(f"{name}-{state}",
                      [T99, "diskann", quant, "real100k",
                       f"kernel_{state}"],
                      prefix, "ec_diskann", DS, gucs=gucs or None)

# --------------------------------------------------------------- storage
for prefix, am in (
    ("t99_hnsw_tq_100k", "hnsw"), ("t99_hnsw_rabitq_100k", "hnsw"),
    ("t99_ivf_tq_100k", "ivf"), ("t99_ivf_rabitq1_100k", "ivf"),
    ("t99_ivf_rabitq4_100k", "ivf"), ("t99_ivf_pqfs_100k", "ivf"),
    ("t99_spire_tq_100k", "spire"), ("t99_spire_rabitq_100k", "spire"),
    ("t99_diskann_pqfs_100k", "diskann"),
    ("t99_diskann_rabitq_100k", "diskann"),
    ("t99_diskann_tq_100k", "diskann"),
    ("t99_qjl_hnsw_1024", "hnsw"), ("t99_qjl_ivf_1024", "ivf"),
    ("t99_qjl_spire_1024", "spire"),
):
    steps.append(storage(prefix, [T99, am]))

config = {
    "name": "task99-index-quant-mode-profile",
    "schema_version": 1,
    "artifact_dir": ART,
    "defaults": {
        "pg": 18, "bits": 4, "seed": 42, "queries_limit": 100,
        "iterations": 300, "force_index": True,
        "sample_backend_memory": False,
    },
    "steps": steps,
}

out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                   "task99-profile-suite.json")
with open(out, "w") as f:
    json.dump(config, f, indent=2)
    f.write("\n")
print(f"wrote {out} ({len(steps)} steps)")
