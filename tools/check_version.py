#!/usr/bin/env python3
"""Check product version projections; Cargo.toml is the sole version owner."""
import json
from pathlib import Path
import re
import tomllib

ROOT = Path(__file__).resolve().parents[1]
SEMVER = re.compile(
    r"(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)"
    r"(?:-((?:0|[1-9][0-9]*|[0-9]*[A-Za-z-][0-9A-Za-z-]*)"
    r"(?:\.(?:0|[1-9][0-9]*|[0-9]*[A-Za-z-][0-9A-Za-z-]*))*))?"
    r"(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?"
)


def check(root=ROOT):
    """Return drift errors without changing versions, locks or release state."""
    manifest = tomllib.loads((root / "Cargo.toml").read_text())
    version = manifest["package"]["version"]
    errors = []
    if not isinstance(version, str) or not SEMVER.fullmatch(version):
        errors.append("Cargo.toml: product version must be Semantic Versioning")
    binding = tomllib.loads((root / "crates/replai-c/Cargo.toml").read_text())
    if binding["package"]["version"] != version:
        errors.append("replai-c package version differs from root authority")
    if binding["dependencies"]["replai"]["version"] != "=" + version:
        errors.append("replai-c dependency does not pin the root product version")
    lock = tomllib.loads((root / "Cargo.lock").read_text())
    for name in ("replai", "replai-c"):
        records = [p for p in lock["package"] if p["name"] == name and "source" not in p]
        if len(records) != 1 or records[0]["version"] != version:
            errors.append(f"Cargo.lock: {name} version differs from root authority")
    policy = json.loads((root / "tools/distribution_policy.json").read_text())
    expected = [f"replai-c-sdk-{version}.tar.gz"]
    if policy["profiles"]["c-sdk-source"]["payload_names"] != expected:
        errors.append("source SDK legal payload name differs from root authority")
    return errors


if __name__ == "__main__":
    failures = check()
    if failures:
        raise SystemExit("\n".join(failures))
    print("product version: root manifest, C crate, lock and SDK policy agree")
