#!/usr/bin/env python3
"""Generate the Task 105 full-scale sweep: fixtures SQL + suite configs.

Emits, next to this script:
  - t105-fixtures-{10k,50k,1m}.sql   per-scale per-variant fixture SQL
  - t105-sweep-{10k,50k,1m}.json     per-scale bench configs (same 45-cell
                                     matrix as the Task 99 profile, minus
                                     the scale-independent QJL cells)
  - t105-g4-100k-rerun.json          G4 confirmation column: the Task 99
                                     kernel-on cells (incl. QJL) under the
                                     flipped default dispatch — derived
                                     from packet 004's neon-cap config by
                                     stripping the ecaz.isa_cap GUC.

The 100k and QJL fixtures already exist in the post-Task-99 snapshots;
only 10k/50k/1m fixtures are new. Per-scale sources (snapshot tables):
10k=real_10k_ivf_tq, 50k=real_50k_ivf_tq, 1m=real_1m_ivf_rabitq1_rerank.
Iterations: 300 (10k/50k), 100 (1m — long queries; keeps the lane under
a day). Sweeps match the Task 99 profile for cross-scale comparability.

Run: python3 gen_t105_sweep.py
"""
import copy
import json
import os

HERE = os.path.dirname(os.path.abspath(__file__))
ART = "reviews/task-105/002-full-scale-sweep-configs/artifacts"

SCALES = {
    "10k": {"source": "real_10k_ivf_tq", "iterations": 300},
    "50k": {"source": "real_50k_ivf_tq", "iterations": 300},
    "1m": {"source": "real_1m_ivf_rabitq1_rerank", "iterations": 100},
}

# (slug, am, create-index WITH clause or None for bare ec_diskann)
VARIANTS = [
    ("hnsw_tq", "ec_hnsw",
     "m='16', ef_construction='128', storage_format=turboquant"),
    ("hnsw_rabitq", "ec_hnsw",
     "m='16', ef_construction='128', storage_format=rabitq"),
    ("ivf_tq", "ec_ivf",
     "nlists='{nlists}', nprobe='64', training_sample_rows='2000', "
     "storage_format=turboquant, rerank=heap_f32, rerank_width='25'"),
    ("ivf_rabitq1", "ec_ivf",
     "nlists='{nlists}', nprobe='64', training_sample_rows='2000', "
     "storage_format=rabitq, quant_bits='1', rerank=heap_f32, rerank_width='50'"),
    ("ivf_rabitq4", "ec_ivf",
     "nlists='{nlists}', nprobe='64', training_sample_rows='2000', "
     "storage_format=rabitq, quant_bits='4', rerank=heap_f32, rerank_width='50'"),
    ("ivf_pqfs", "ec_ivf",
     "nlists='{nlists}', nprobe='64', training_sample_rows='2000', "
     "storage_format=pq_fastscan, pq_group_size='8'"),
    ("spire_tq", "ec_spire",
     "nlists='{spire_nlists}', nprobe='24', rerank_width='25', "
     "local_store_count='1', storage_format=turboquant"),
    ("spire_rabitq", "ec_spire",
     "nlists='{spire_nlists}', nprobe='24', rerank_width='25', "
     "local_store_count='1', storage_format=rabitq"),
    ("diskann_pqfs", "ec_diskann", None),
    ("diskann_rabitq", "ec_diskann", "storage_format=rabitq"),
    ("diskann_tq", "ec_diskann", "storage_format=turboquant"),
]

# nlists scaling: keep the per-scale conventions used by prior fixtures
# (10k: 64 / task87; 50k: 64 ivf + 128 spire; 100k baseline used 64/128;
#  1m: 256 ivf / 512 spire — sqrt-ish scaling, recorded here as the
#  convention for this sweep).
NLISTS = {"10k": ("64", "32"), "50k": ("64", "128"), "1m": ("256", "512")}


def fixtures_sql(scale):
    src = SCALES[scale]["source"]
    nlists, spire_nlists = NLISTS[scale]
    out = [
        f"-- Task 105 fixtures @ {scale} (sources: {src}_corpus/_queries)",
        f"CREATE TABLE t105_src_{scale}_corpus AS SELECT id, source, embedding FROM {src}_corpus;",
        f"ALTER TABLE t105_src_{scale}_corpus ADD PRIMARY KEY (id);",
        f"CREATE TABLE t105_src_{scale}_queries AS SELECT id, source FROM {src}_queries;",
        f"ANALYZE t105_src_{scale}_corpus; ANALYZE t105_src_{scale}_queries;",
        "",
    ]
    for slug, am, withc in VARIANTS:
        p = f"t105_{slug}_{scale}"
        out += [
            f"CREATE TABLE {p}_corpus AS SELECT id, source, embedding FROM t105_src_{scale}_corpus;",
            f"ALTER TABLE {p}_corpus ADD PRIMARY KEY (id);",
            f"CREATE TABLE {p}_queries AS SELECT id, source FROM t105_src_{scale}_queries;",
        ]
        idx = f"CREATE INDEX {p}_idx ON {p}_corpus USING {am} (embedding)"
        if withc:
            idx += " WITH (" + withc.format(nlists=nlists, spire_nlists=spire_nlists) + ")"
        out += [idx + ";", f"ANALYZE {p}_corpus; ANALYZE {p}_queries;", ""]
    return "\n".join(out) + "\n"


def bench_steps(scale):
    it = SCALES[scale]["iterations"]
    steps = []

    def recall(name, tags, prefix, profile, sweep, gucs=None, ivf_soa=None):
        s = {"kind": "recall", "name": f"recall-{name}", "tags": tags,
             "prefix": prefix, "profile": profile, "k": 10, "sweep": sweep,
             "queries_limit": 100,
             "truth_cache_dir": "${artifact_dir}/truth-cache",
             "log_output": "${artifact_dir}/recall-" + name + ".log"}
        if gucs:
            s["session_gucs"] = gucs
        if ivf_soa is not None:
            s["ivf_scratch_soa_batch_decode"] = ivf_soa
        return s

    def latency(name, tags, prefix, profile, sweep, gucs=None, ivf_soa=None):
        s = {"kind": "latency", "name": f"latency-{name}", "tags": tags,
             "prefix": prefix, "profile": profile, "k": 10, "sweep": sweep,
             "iterations": it, "concurrency": 1,
             "cache_state": f"t105_{name.replace('-', '_')}",
             "task87_candidate_batch_counters": True,
             "log_output": "${artifact_dir}/latency-" + name + ".log"}
        if gucs:
            s["session_gucs"] = gucs
        if ivf_soa is not None:
            s["ivf_scratch_soa_batch_decode"] = ivf_soa
        return s

    def cell(name, tags, prefix, profile, sweep, gucs=None, ivf_soa=None):
        steps.append(recall(name, tags, prefix, profile, sweep, gucs, ivf_soa))
        steps.append(latency(name, tags, prefix, profile, sweep, gucs, ivf_soa))

    T = ["task105", f"scale={scale}"]
    HS, IS, DS = [80, 160], [16, 64], [64, 128]
    ISO = "ec_hnsw.disable_binary_prefilter=on"
    HOFF = "ec_hnsw.candidate_batch_scoring=off"
    SOFF = "ec_spire.candidate_batch_scoring=off"
    DOFF = "ec_diskann.candidate_batch_scoring=off"

    # HNSW TQ exact-mode isolation + default path
    for mode, status in (("full_lut", None), ("int8_approx", None),
                         ("exact", "no_kernel_baseline"),
                         ("tiled_lut", "retired")):
        base = [ISO, f"ec_hnsw.turboquant_exact_score_mode={mode}"]
        tags = T + ["hnsw", "turboquant", f"mode={mode}", "kernel_on"]
        if status == "retired":
            tags.append("kernel_status=retired")
        elif status:
            tags.append(status)
        cell(f"hnsw-tq-{mode}-on-{scale}", tags,
             f"t105_hnsw_tq_{scale}", "ec_hnsw", HS, gucs=base)
        if status != "retired":
            cell(f"hnsw-tq-{mode}-off-{scale}",
                 [t.replace("kernel_on", "kernel_off") for t in tags],
                 f"t105_hnsw_tq_{scale}", "ec_hnsw", HS, gucs=base + [HOFF])
    cell(f"hnsw-tq-default-on-{scale}", T + ["hnsw", "turboquant", "default_path", "kernel_on"],
         f"t105_hnsw_tq_{scale}", "ec_hnsw", HS)
    cell(f"hnsw-tq-default-off-{scale}", T + ["hnsw", "turboquant", "default_path", "kernel_off"],
         f"t105_hnsw_tq_{scale}", "ec_hnsw", HS, gucs=[HOFF])
    for state, gucs in (("on", None), ("off", [HOFF])):
        cell(f"hnsw-rabitq-{state}-{scale}", T + ["hnsw", "rabitq", f"kernel_{state}"],
             f"t105_hnsw_rabitq_{scale}", "ec_hnsw", HS, gucs=gucs)

    # IVF — batch axis via the explicit flag; note default is now ON
    # (ADR-077 §4). CAVEAT: the suite runner treats ivf_soa=False the
    # same as absent (it only appends --ivf-scratch-soa-batch-decode
    # when True), so post-flip the "off" cells inherit batch decode ON.
    # They are therefore SAME-CONFIG stability pairs, NOT a kernel A/B.
    # The IVF kernel differential is Task 99's pre-flip 100k A/B
    # (reviews/task-99/008|009); see packet 006 honest markers. A true
    # off arm would require the runner to emit an explicit `SET ... =
    # off` for the False case (suite.rs gap) plus a fresh rerun.
    for quant, prefix, states, marker in (
        ("turboquant", f"t105_ivf_tq_{scale}", ("on", "off"), None),
        ("rabitq1", f"t105_ivf_rabitq1_{scale}", ("on", "off"), None),
        ("rabitq4", f"t105_ivf_rabitq4_{scale}", ("on",), "no_kernel_storage_lane"),
        ("pq_fastscan", f"t105_ivf_pqfs_{scale}", ("on", "off"), None),
    ):
        for state in states:
            tags = T + ["ivf", quant, f"kernel_{state}"]
            if marker:
                tags.append(marker)
            cell(f"ivf-{quant}-{state}-{scale}", tags, prefix, "ec_ivf", IS,
                 ivf_soa=(state == "on"))

    # SPIRE
    for quant, prefix in (("turboquant", f"t105_spire_tq_{scale}"),
                          ("rabitq", f"t105_spire_rabitq_{scale}")):
        for state, gucs in (("on", None), ("off", [SOFF])):
            tags = T + ["spire", quant, f"kernel_{state}"]
            if quant == "rabitq":
                tags.append("attribution_check")
            cell(f"spire-{quant}-{state}-{scale}", tags, prefix, "ec_spire", IS, gucs=gucs)

    # DiskANN
    for name, prefix, quant, base in (
        ("diskann-pqfs-binary", f"t105_diskann_pqfs_{scale}", "binary",
         ["ec_diskann.prefilter_kind=binary_sidecar"]),
        ("diskann-pqfs-grouped-pq", f"t105_diskann_pqfs_{scale}", "grouped_pq",
         ["ec_diskann.prefilter_kind=grouped_pq"]),
        ("diskann-rabitq", f"t105_diskann_rabitq_{scale}", "rabitq", []),
        ("diskann-tq", f"t105_diskann_tq_{scale}", "turboquant", []),
    ):
        for state in ("on", "off"):
            gucs = list(base) + ([DOFF] if state == "off" else [])
            cell(f"{name}-{state}-{scale}", T + ["diskann", quant, f"kernel_{state}"],
                 prefix, "ec_diskann", DS, gucs=gucs or None)

    # storage per fixture
    for slug, _, _ in VARIANTS:
        p = f"t105_{slug}_{scale}"
        steps.append({"kind": "storage", "name": f"storage-{p}",
                      "tags": T + ["storage"], "prefix": p,
                      "log_file": "${artifact_dir}/storage-" + p + ".log"})
    return steps


def write(name, obj):
    path = os.path.join(HERE, name)
    with open(path, "w") as f:
        if name.endswith(".json"):
            json.dump(obj, f, indent=2)
            f.write("\n")
        else:
            f.write(obj)
    print("wrote", path)


for scale in SCALES:
    write(f"t105-fixtures-{scale}.sql", fixtures_sql(scale))
    steps = bench_steps(scale)
    write(f"t105-sweep-{scale}.json", {
        "name": f"task105-sweep-{scale}",
        "schema_version": 1,
        "artifact_dir": ART,
        "defaults": {"pg": 18, "bits": 4, "seed": 42, "queries_limit": 100,
                     "iterations": SCALES[scale]["iterations"],
                     "force_index": True, "sample_backend_memory": False},
        "steps": steps,
    })
    print(f"  {scale}: {len(steps)} steps")

# G4 100k confirmation column: neon-cap config minus the cap GUC.
NEONCAP = os.path.normpath(os.path.join(
    HERE, "..", "..", "..", "task-99", "004-isa-cap-dispatch", "artifacts",
    "t99-g4-neon-cap-suite.json"))
with open(NEONCAP) as f:
    cap_cfg = json.load(f)
steps = []
for s in cap_cfg["steps"]:
    s = copy.deepcopy(s)
    s["name"] = s["name"].replace("-neoncap", "-confirm")
    s["tags"] = [t for t in s["tags"] if t != "isa_cap=neon"] + ["dispatch_confirm"]
    s["session_gucs"] = [g for g in s.get("session_gucs", [])
                         if not g.startswith("ecaz.isa_cap")]
    if not s["session_gucs"]:
        del s["session_gucs"]
    if "cache_state" in s:
        s["cache_state"] = s["cache_state"].replace("_neoncap", "_confirm")
    if "log_output" in s:
        s["log_output"] = s["log_output"].replace("-neoncap", "-confirm")
    steps.append(s)
write("t105-g4-100k-rerun.json", {
    "name": "task105-g4-100k-dispatch-confirm",
    "schema_version": 1,
    "artifact_dir": ART,
    "defaults": cap_cfg["defaults"],
    "steps": steps,
})
print(f"  g4-100k-rerun: {len(steps)} steps")
