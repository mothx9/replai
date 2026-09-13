#!/usr/bin/env python3
"""Pack and replay the retained COMPLETION.KEYMAP.SUGGESTION surfaces corpus."""
import argparse
import base64
import gzip
import hashlib
import json
from pathlib import Path
import platform
import subprocess
import time

ROOT = Path(__file__).resolve().parents[2]
MAX_ARCHIVE_BYTES = 32 * 1024 * 1024


def digest(inputs):
    hashes = [hashlib.sha256(value).hexdigest() for value in inputs]
    return hashlib.sha256("\n".join(sorted(hashes)).encode()).hexdigest()


def read(path):
    with gzip.open(path, "rb") as source:
        raw = source.read(MAX_ARCHIVE_BYTES + 1)
    assert len(raw) <= MAX_ARCHIVE_BYTES
    archive = json.loads(raw)
    assert archive["schema"] == 1 and archive["target"] == "surfaces"
    inputs = [base64.b64decode(value, validate=True) for value in archive["inputs"]]
    assert all(len(value) <= 4096 for value in inputs)
    assert archive["summary"]["budget_complete"] and archive["summary"]["cpu_seconds"] >= 1800
    assert archive["summary"]["final_corpus"] == {"files": len(inputs), "sha256": digest(inputs)}
    return archive, inputs


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="mode", required=True)
    pack = sub.add_parser("pack")
    pack.add_argument("--campaign", type=Path, required=True)
    pack.add_argument("--output", type=Path, required=True)
    replay = sub.add_parser("replay")
    replay.add_argument("--archive", type=Path, required=True)
    replay.add_argument("--work", type=Path, required=True)
    args = parser.parse_args()
    if args.mode == "pack":
        environment = json.loads((args.campaign / "environment.json").read_text())
        summary = json.loads((args.campaign / "surfaces" / "summary.json").read_text())
        assert not environment["dirty"] and summary["passed"] and summary["budget_complete"]
        inputs = [path.read_bytes() for path in sorted((args.campaign / "editor" / "corpus").iterdir()) if path.is_file()]
        archive = {"schema": 1, "target": "surfaces", "environment": environment,
                   "summary": summary, "inputs": [base64.b64encode(value).decode() for value in inputs]}
        args.output.parent.mkdir(parents=True, exist_ok=True)
        with args.output.open("xb") as output:
            output.write(gzip.compress(json.dumps(archive, sort_keys=True).encode(), mtime=0))
        read(args.output)
        print(json.dumps({"archive": str(args.output), "bytes": args.output.stat().st_size,
                          "sha256": hashlib.sha256(args.output.read_bytes()).hexdigest()}))
        return
    archive, inputs = read(args.archive)
    args.work.mkdir(parents=True, exist_ok=False)
    corpus = args.work / "corpus"
    corpus.mkdir()
    for index, value in enumerate(inputs):
        (corpus / str(index)).write_bytes(value)
    subprocess.run(["cargo", "build", "--locked", "--release", "--manifest-path",
                    "tools/hardening/Cargo.toml", "--bin", "replay"], cwd=ROOT, check=True)
    binary = ROOT / "tools/hardening/target/release" / ("replay.exe" if platform.system() == "Windows" else "replay")
    started = time.time()
    with (args.work / "stdout.txt").open("w") as stdout, (args.work / "stderr.txt").open("w") as stderr:
        result = subprocess.run([str(binary), "surfaces", str(corpus)], cwd=ROOT,
                                stdout=stdout, stderr=stderr, timeout=1800)
    receipt = {"head": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
               "tree": subprocess.check_output(["git", "rev-parse", "HEAD^{tree}"], cwd=ROOT, text=True).strip(),
               "runtime_tree": subprocess.check_output(["git", "rev-parse", "HEAD:src"], cwd=ROOT, text=True).strip(),
               "platform": platform.uname()._asdict(), "rustc": subprocess.check_output(["rustc", "-vV"], text=True).strip(),
               "archive_sha256": hashlib.sha256(args.archive.read_bytes()).hexdigest(),
               "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
               "corpus": archive["summary"]["final_corpus"], "start_unix": started,
               "end_unix": time.time(), "status": "PASS" if result.returncode == 0 else "FAIL"}
    (args.work / "summary.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print(json.dumps(receipt, indent=2))
    result.check_returncode()


if __name__ == "__main__":
    main()
