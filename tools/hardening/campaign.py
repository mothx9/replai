#!/usr/bin/env python3
"""Linux coverage-guided campaign. Budgets are child CPU seconds, never wall time."""
import argparse
import concurrent.futures
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import shutil
import subprocess
import time

ROOT = Path(__file__).resolve().parents[2]
TARGETS = ("protocol", "editor", "results", "geometry", "cabi")


def command(*args):
    return subprocess.check_output(args, cwd=ROOT, text=True).strip()


def digest(directory):
    hashes = sorted(hashlib.sha256(p.read_bytes()).hexdigest()
                    for p in directory.iterdir() if p.is_file())
    return {"files": len(hashes), "sha256": hashlib.sha256(
        "\n".join(hashes).encode()).hexdigest()}


def seeds(directory):
    values = [b"", b"\x1b[200~a\r\n\t\x1b[201~", b"\x1b[", b"\x1b]52;host\x07",
              "e\u0301界👩‍💻🇮🇹\n\t".encode(), bytes(range(256)) * 2]
    values.extend(bytes([255, choice, 0, 0]) for choice in range(7))
    for seed in range(1, 17):
        state, data = seed, bytearray()
        for _ in range(512):
            state = (state * 6364136223846793005 + 1442695040888963407) % 2**64
            data.append((state >> 32) % 256)
        values.append(bytes(data))
    for value in values:
        (directory / hashlib.sha256(value).hexdigest()).write_bytes(value)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--work", type=Path, required=True)
    parser.add_argument("--cpu-seconds", type=int, default=3600)
    parser.add_argument("--targets", nargs="+", choices=TARGETS, default=TARGETS)
    parser.add_argument("--workers", type=int, default=1, help="independent fuzz processes per target; CPU accounting is summed")
    parser.add_argument("--allow-dirty", action="store_true", help="smoke only; never release evidence")
    args = parser.parse_args()
    assert platform.system() == "Linux", "campaign accounting currently requires Linux wait4"
    assert 1 <= args.cpu_seconds <= 86400
    assert 1 <= args.workers <= 8
    status = command("git", "status", "--porcelain")
    assert not status or args.allow_dirty, "commit the instrumented source before qualification"
    args.work.mkdir(parents=True, exist_ok=False)
    host = command("rustc", "+nightly", "-vV").split("host: ")[1].splitlines()[0]
    metadata = {"head": command("git", "rev-parse", "HEAD"), "tree": command("git", "rev-parse", "HEAD^{tree}"),
                "dirty": bool(status), "uname": platform.uname()._asdict(), "libc": platform.libc_ver(),
                "rustc": command("rustc", "+nightly", "-vV"), "cargo_fuzz": command("cargo", "fuzz", "--version"),
                "instrumentation": "cargo-fuzz default address sanitizer + inline coverage + trace comparisons",
                "requested_cpu_seconds_per_target": args.cpu_seconds, "workers_per_target": args.workers, "start_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())}
    (args.work / "environment.json").write_text(json.dumps(metadata, indent=2)+"\n")
    with (args.work / "build.log").open("w") as log:
        for target in args.targets:
            subprocess.run(["cargo", "+nightly", "fuzz", "build", target, "--fuzz-dir", "tools/hardening", "--features", "fuzzing"],
                           cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, check=True)

    def campaign(target):
        work = args.work / target
        corpus = work / "corpus"
        corpus.mkdir(parents=True)
        seeds(corpus)
        summary = {"target": target, "initial_corpus": digest(corpus), "chunks": [], "cpu_seconds": 0.0}
        binary = work / "instrumented-target"
        shutil.copy2(ROOT / "tools/hardening/target" / host / "release" / target, binary)
        summary["binary_sha256"] = hashlib.sha256(binary.read_bytes()).hexdigest()
        def chunk(index, seconds):
            argv = [str(binary), str(corpus), "-max_len=8192" if target == "geometry" else "-max_len=4096",
                    "-timeout=10", "-rss_limit_mb=2048", "-print_final_stats=1", "-seed="+str(17011+index%args.workers),
                    "-max_total_time="+str(seconds), "-artifact_prefix="+str(work)+"/"]
            started = time.time()
            with (work / f"chunk-{index:04}.log").open("w") as log:
                child = subprocess.Popen(argv, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT,
                                         env={**os.environ, "TERM": "xterm-256color"})
                _, result, usage = os.wait4(child.pid, 0)
                child.returncode = os.waitstatus_to_exitcode(result)
            return {"argv": argv, "start_unix": started, "end_unix": time.time(),
                    "user_seconds": usage.ru_utime, "system_seconds": usage.ru_stime,
                    "peak_rss_kib": usage.ru_maxrss, "exit_code": child.returncode}

        with concurrent.futures.ThreadPoolExecutor(max_workers=args.workers) as workers:
            while summary["cpu_seconds"] < args.cpu_seconds:
                first = len(summary["chunks"])
                seconds = min(30, max(1, math.ceil((args.cpu_seconds-summary["cpu_seconds"])/args.workers)))
                results = list(workers.map(lambda index: chunk(index, seconds), range(first, first+args.workers)))
                summary["chunks"].extend(results)
                summary["cpu_seconds"] += sum(r["user_seconds"]+r["system_seconds"] for r in results)
                summary["final_corpus"] = digest(corpus)
                summary["budget_complete"] = summary["cpu_seconds"] >= args.cpu_seconds
                summary["passed"] = all(r["exit_code"] == 0 for r in results)
                (work / "summary.json").write_text(json.dumps(summary, indent=2)+"\n")
                print(f"{target}: {summary['cpu_seconds']:.2f}/{args.cpu_seconds} CPU s; pass {summary['passed']}", flush=True)
                if not summary["passed"]:
                    return False
        return True

    with concurrent.futures.ThreadPoolExecutor(max_workers=len(args.targets)) as pool:
        results = list(pool.map(campaign, args.targets))
    if not all(results):
        raise SystemExit("campaign finding: retain logs/input, minimize, classify and replay before continuing")


if __name__ == "__main__":
    main()
