#!/usr/bin/env python3
"""Write or verify the narrow generated-asset provenance manifest."""
import argparse
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
MANIFEST = ROOT / "assets/readme/manifest.json"


def sha(path):
    return hashlib.sha256((ROOT / path).read_bytes()).hexdigest()


def record(file, generator, command, inputs, **extra):
    path = ROOT / file
    return {
        "file": file,
        "generator": generator,
        "command": command,
        "sha256": sha(file),
        "bytes": path.stat().st_size,
        "inputs": [{"file": value, "sha256": sha(value)} for value in inputs],
        **extra,
    }


def data():
    capture_inputs = [
        "examples/showcase.rs",
        "examples/support/showcase_host.rs",
        "tools/readme/capture_showcase.py",
        "tools/docs/capture-requirements.txt",
    ]
    vector = "tools/readme/render_vectors.py"
    evidence = "tools/hardening/evidence/q2-linux-aarch64.json"
    return {
        "schema": 1,
        "scope": "README generated assets only; not project or release status",
        "assets": [
            record("assets/readme/terminal-showcase.png", "tools/readme/capture_showcase.py",
                   "cargo build --locked --example showcase && python3 tools/readme/capture_showcase.py",
                   capture_inputs, dimensions=[1072, 891], format="PNG", real_pty=True),
            record("assets/readme/terminal-showcase.gif", "tools/readme/capture_showcase.py",
                   "cargo build --locked --example showcase && python3 tools/readme/capture_showcase.py",
                   capture_inputs, dimensions=[1072, 891], format="GIF", real_pty=True,
                   frames=35, duration_ms=11630),
            record("assets/readme/architecture-dark.svg", vector,
                   "python3 tools/readme/render_vectors.py", [vector], dimensions=[1200, 920], format="SVG"),
            record("assets/readme/architecture-light.svg", vector,
                   "python3 tools/readme/render_vectors.py", [vector], dimensions=[1200, 920], format="SVG"),
            record("assets/readme/benchmark-q2.svg", vector,
                   "python3 tools/readme/render_vectors.py", [vector, evidence], dimensions=[1200, 680],
                   format="SVG", evidence=evidence, evidence_head="6975c0979a1fd13f619f2d079b7494945ca18c5e"),
        ],
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    encoded = json.dumps(data(), indent=2, sort_keys=True) + "\n"
    if args.check:
        assert MANIFEST.read_text() == encoded, "README asset manifest drifted"
    else:
        MANIFEST.parent.mkdir(parents=True, exist_ok=True)
        MANIFEST.write_text(encoded)
    print(f"PASS {len(data()['assets'])} README asset records")


if __name__ == "__main__":
    main()
