#!/usr/bin/env python3
"""Derive the G4 NEON-capped supplemental suite from the main profile.

Takes every kernel-on bench cell from packet 002's
task99-profile-suite.json (excluding cells carrying a kernel_status
marker — retired / missing_kernel / structurally_absent have no NEON
kernel to measure) and re-emits it with `ecaz.isa_cap=neon`, so the
Graviton 4 lane can measure the NEON kernels that SVE2 otherwise always
out-dispatches. Run on the G4 lane only, AFTER the main profile run
(fixtures and truth caches already exist). Counter rows must report
isa=neon; that is the cap working, not a dispatch bug.

Run: python3 gen_t99_g4_neon_cap.py
"""
import copy
import json
import os

HERE = os.path.dirname(os.path.abspath(__file__))
MAIN = os.path.normpath(os.path.join(
    HERE, "..", "..", "002-profile-suiteconfig", "artifacts",
    "task99-profile-suite.json"))

with open(MAIN) as f:
    main_cfg = json.load(f)

steps = []
for step in main_cfg["steps"]:
    if step["kind"] not in ("recall", "latency"):
        continue
    tags = step.get("tags", [])
    if "kernel_on" not in tags:
        continue
    if any(t.startswith("kernel_status=") for t in tags):
        continue
    if any(t.startswith("no_kernel_") for t in tags):
        continue  # runnable no-kernel baselines: nothing to cap
    s = copy.deepcopy(step)
    s["name"] = s["name"] + "-neoncap"
    s["tags"] = tags + ["isa_cap=neon", "g4_only"]
    s["session_gucs"] = s.get("session_gucs", []) + ["ecaz.isa_cap=neon"]
    if "cache_state" in s:
        s["cache_state"] = s["cache_state"] + "_neoncap"
    if "log_output" in s:
        s["log_output"] = s["log_output"].replace(".log", "-neoncap.log")
    steps.append(s)

config = {
    "name": "task99-g4-neon-cap",
    "schema_version": 1,
    "artifact_dir": main_cfg["artifact_dir"],
    "defaults": main_cfg["defaults"],
    "steps": steps,
}

out = os.path.join(HERE, "t99-g4-neon-cap-suite.json")
with open(out, "w") as f:
    json.dump(config, f, indent=2)
    f.write("\n")
print(f"wrote {out} ({len(steps)} steps)")
