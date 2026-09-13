#!/usr/bin/env python3
"""Qualify the interaction-ergonomics delta on one exact native runner."""
import argparse
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import time

ROOT = Path(__file__).resolve().parents[2]
PTY_TEST = "pty_reverse_search_word_undo_kill_and_plain_narrow_restore_exactly"


def run(command, **kwargs):
    return subprocess.run(command, cwd=ROOT, check=True, text=True, **kwargs)


def test_binary():
    result = run(
        ["cargo", "test", "--locked", "--test", "pty", "--no-run", "--message-format", "json"],
        stdout=subprocess.PIPE,
    )
    artifacts = [json.loads(line) for line in result.stdout.splitlines() if line.startswith("{")]
    paths = [Path(item["executable"]) for item in artifacts
             if item.get("reason") == "compiler-artifact"
             and item.get("target", {}).get("name") == "pty" and item.get("executable")]
    assert len(paths) == 1, paths
    return paths[0]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--work", type=Path, required=True)
    parser.add_argument("--portable-only", action="store_true")
    args = parser.parse_args()
    args.work.mkdir(parents=True, exist_ok=False)
    started = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
    run(["cargo", "test", "--locked", "--test", "ergonomics"])
    run(["cargo", "run", "--locked", "--release", "--manifest-path",
         "tools/hardening/Cargo.toml", "--bin", "ergonomics-sequences", "--", "1000", "7632459182231841"])
    summary = {
        "head": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
        "tree": subprocess.check_output(["git", "rev-parse", "HEAD^{tree}"], cwd=ROOT, text=True).strip(),
        "runtime_tree": subprocess.check_output(["git", "rev-parse", "HEAD:src"], cwd=ROOT, text=True).strip(),
        "platform": platform.platform(), "machine": platform.machine(), "started_utc": started,
        "rustc": subprocess.check_output(["rustc", "-vV"], text=True).strip(),
        "portable_model_sequences": 1000, "portable_tests": "PASS",
    }
    if not args.portable_only:
        executable = test_binary()
        run([str(executable), PTY_TEST, "--exact", "--nocapture"])
        if platform.system() == "Linux":
            valgrind = shutil.which("valgrind")
            assert valgrind, "Valgrind is required on Linux"
            log = args.work / "valgrind.log"
            run([valgrind, "--leak-check=full", "--show-leak-kinds=all",
                 "--errors-for-leak-kinds=definite,indirect,possible", "--error-exitcode=99",
                 f"--log-file={log}", str(executable), PTY_TEST, "--exact", "--nocapture"])
            report = log.read_text()
            assert "ERROR SUMMARY: 0 errors" in report
            assert "definitely lost: 0 bytes" in report
            assert "indirectly lost: 0 bytes" in report
            assert "possibly lost: 0 bytes" in report
            summary["memory_tool"] = subprocess.check_output([valgrind, "--version"], text=True).strip()
        elif platform.system() == "Darwin":
            leaks = Path("/usr/bin/leaks")
            assert leaks.is_file(), "native leaks is required on macOS"
            result = run([str(leaks), "--atExit", "--", str(executable), PTY_TEST,
                          "--exact", "--nocapture"], stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
            (args.work / "leaks.txt").write_text(result.stdout)
            assert "0 leaks for 0 total leaked bytes" in result.stdout
            summary["memory_tool"] = "macOS leaks --atExit"
        else:
            raise AssertionError("native terminal qualification supports Linux/macOS only")
        summary.update(real_pty="PASS", memory="PASS", widths=[20, 40, 80, 132],
                       modes=["styled", "plain"])
    summary["ended_utc"] = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
    (args.work / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
