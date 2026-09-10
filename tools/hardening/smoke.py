#!/usr/bin/env python3
"""Fast ordinary-CI gates; explicitly not the full release campaign."""
from pathlib import Path
import os
import platform
import subprocess
import tempfile
from campaign import ROOT, TARGETS, seeds


def run(*args):
    subprocess.run(args, cwd=ROOT, check=True, env={**os.environ, "TERM": "xterm-256color"})


run("cargo", "build", "--locked", "--release", "--manifest-path", "tools/hardening/Cargo.toml", "--bin", "replay", "--bin", "bench")
suffix = ".exe" if platform.system() == "Windows" else ""
replay = str(ROOT / ("tools/hardening/target/release/replay"+suffix))
run(replay, "generated", "1000", "1")
run(replay, "faults", "3")
run(replay, "geometry", "tools/hardening/regressions/geometry")
with tempfile.TemporaryDirectory(prefix="replai-hardening-smoke-") as directory:
    seeds(Path(directory))
    for target in TARGETS:
        if target != "cabi" or platform.system() != "Windows":
            run(replay, target, directory)
run(str(ROOT / ("tools/hardening/target/release/bench"+suffix)), "3", "1")
