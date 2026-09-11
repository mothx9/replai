#!/usr/bin/env python3
"""Pre-register Q2 thresholds; evaluate two independent alternating control/candidate runs."""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import shutil
import statistics
import subprocess
import time

ROOT = Path(__file__).resolve().parents[2]


def hashed(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def distribution(samples):
    assert samples and all(isinstance(x, (int, float)) and math.isfinite(x) and x >= 0 for x in samples)
    values = sorted(samples)
    return statistics.median(values), values[math.ceil(.95*len(values))-1]


def mad(values):
    median = statistics.median(values)
    return statistics.median(abs(x-median) for x in values)


def thresholds(samples, resolution):
    assert len(samples) == 155
    controls = [distribution(samples[i:i+31]) for i in range(0, 155, 31)]
    median, p95 = distribution(samples)
    median_mad = mad([x[0] for x in controls])
    p95_mad = mad([x[1] for x in controls])
    return {"median_ns": median, "p95_ns": p95, "control_median_mad_ns": median_mad,
            "control_p95_mad_ns": p95_mad,
            "median_allowance_ns": max(.15*median, 5*median_mad, 2*resolution),
            "p95_allowance_ns": max(.25*p95, p95_mad, 2*resolution)}


def read(path):
    data = [json.loads(s) for s in path.read_text().splitlines()]
    header, rows = data[0], data[1:]
    assert len({r["id"] for r in rows}) == len(rows) == 32
    assert header["samples"] == 31 and header["batches"] == 5
    return header, {r["id"]: r for r in rows}


def run(binary, path):
    with path.open("x") as out:
        subprocess.run([str(binary), "31", "5"], stdout=out, check=True, timeout=1800)
    return read(path)


def identity(binary):
    def cmd(*args):
        return subprocess.check_output(args, cwd=ROOT, text=True).strip()
    governors = {str(p): p.read_text().strip() for p in Path("/sys/devices/system/cpu").glob("cpu*/cpufreq/scaling_governor")}
    return {"head": cmd("git", "rev-parse", "HEAD"), "tree": cmd("git", "rev-parse", "HEAD^{tree}"),
            "runtime_tree": cmd("git", "rev-parse", "HEAD:src"), "binary_sha256": hashed(binary),
            "rustc": cmd("rustc", "-vV"), "host": platform.uname()._asdict(),
            "affinity": sorted(os.sched_getaffinity(0)) if hasattr(os, "sched_getaffinity") else None,
            "governors": governors, "load": os.getloadavg() if hasattr(os, "getloadavg") else None,
            "utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())}


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("mode", choices=["register", "compare"])
    p.add_argument("--work", type=Path, required=True)
    p.add_argument("--binary", type=Path, required=True)
    p.add_argument("--allocation-binary", type=Path)
    p.add_argument("--registration", type=Path)
    p.add_argument("--control-binary", type=Path)
    p.add_argument("--cpu", type=int)
    a = p.parse_args()
    if a.cpu is not None:
        os.sched_setaffinity(0, {a.cpu})
    a.work.mkdir(parents=True, exist_ok=False)
    assert not subprocess.check_output(["git", "status", "--porcelain"], cwd=ROOT), "qualification requires committed source"
    # Keep the exact measured executables; Cargo may replace its output paths.
    for field in ("binary", "allocation_binary", "control_binary"):
        source = getattr(a, field)
        if source is not None:
            destination = a.work / field
            shutil.copy2(source, destination)
            setattr(a, field, destination.resolve())
    metadata = identity(a.binary)
    if a.mode == "register":
        assert a.allocation_binary
        header, rows = run(a.binary, a.work / "controls.jsonl")
        assert not header["instrumented_allocations"]
        alloc_header, allocations = run(a.allocation_binary, a.work / "allocations.jsonl")
        assert alloc_header["instrumented_allocations"]
        gates = {}
        for key, row in rows.items():
            gate = thresholds(row["samples_ns"], header["timer_resolution_ns"])
            assert len(set(row["encoded_bytes"])) == 1
            values = allocations[key]["allocations"]
            assert all(v == values[0] for v in values), (key, "allocation oracle is not deterministic")
            gate.update(encoded_bytes=row["encoded_bytes"][0], allocations=values[0])
            gates[key] = gate
        for key in ["append/1024/0", "cursor/1024/0"]:
            assert gates[key]["allocations"]["allocation_calls"] == 0
        registration = {"identity": metadata, "allocation_binary_sha256": hashed(a.allocation_binary),
                        "controls_sha256": hashed(a.work / "controls.jsonl"), "timer_resolution_ns": header["timer_resolution_ns"],
                        "formula": "median max(15%,5*MAD,2*timer); p95 max(25%,p95-MAD,2*timer)", "gates": gates}
        with (a.work / "registered.json").open("x") as f:
            json.dump(registration, f, indent=2)
            f.write("\n")
        print("REGISTERED", len(gates), "workloads; candidate evaluation has not run")
    else:
        assert a.registration and a.control_binary and a.allocation_binary
        registered = json.loads(a.registration.read_text())
        assert hashed(a.control_binary) == registered["identity"]["binary_sha256"]
        comparisons = []
        for i in range(2):
            _, control = run(a.control_binary, a.work / f"control-{i}.jsonl")
            _, candidate = run(a.binary, a.work / f"candidate-{i}.jsonl")
            assert set(control) == set(candidate) == set(registered["gates"])
            exceedances = []
            for key, row in candidate.items():
                gate = registered["gates"][key]
                median, p95 = distribution(row["samples_ns"])
                # Contemporaneous controls are retained for investigation; they
                # never silently raise the already registered absolute bound.
                med_fail = median > gate["median_ns"]+gate["median_allowance_ns"]
                p95_fail = p95 > gate["p95_ns"]+gate["p95_allowance_ns"]
                if med_fail or p95_fail:
                    exceedances.append(key)
                assert all(n == gate["encoded_bytes"] for n in row["encoded_bytes"]), (key, "encoded-byte regression")
            comparisons.append(exceedances)
        _, allocations = run(a.allocation_binary, a.work / "candidate-allocations.jsonl")
        for key, row in allocations.items():
            assert all(v == registered["gates"][key]["allocations"] for v in row["allocations"]), (key, "allocation regression")
        blocked = sorted(set(comparisons[0]) & set(comparisons[1]))
        summary = {"identity": metadata, "registration_sha256": hashed(a.registration),
                   "alternating_exceedances": comparisons, "blocking_reproduced_exceedances": blocked,
                   "allocation_and_byte_gates": "PASS", "passed": not blocked}
        (a.work / "summary.json").write_text(json.dumps(summary, indent=2)+"\n")
        print(json.dumps(summary, indent=2))
        if blocked:
            raise SystemExit("reproduced threshold exceedance: investigation required; do not overwrite baseline")


if __name__ == "__main__":
    main()
