#!/usr/bin/env python3
"""Build a deterministic versioned REPLAI C SDK source archive from Git."""
import argparse
import gzip
import hashlib
import io
import json
from pathlib import Path, PurePosixPath
import subprocess
import tarfile
import tempfile
import tomllib

ROOT = Path(__file__).resolve().parents[1]
EXACT = {
    "Cargo.toml", "Cargo.lock", "LICENSE", "README.md",
    "include/replai.h", "api/c-abi.json", "examples/c/demo.c",
    "tests/c/contracts.c", "tests/c/layout.c", "tests/fixtures/presentation.tsv",
    "tests/support/terminal_state.rs", "tools/generate_abi.py", "tools/stage_c.py",
    "tools/qualify_c.py", "tools/c_pty.py",
    "docs/c-api.md", "docs/c-sdk.md",
}
PREFIXES = ("src/", "crates/replai-c/", "cmake/")


def git(*args, binary=False):
    return subprocess.check_output(["git", *args], cwd=ROOT, text=not binary)


def selected_files(revision):
    paths = git("ls-tree", "-r", "--name-only", revision).splitlines()
    selected = [p for p in paths if p in EXACT or p.startswith(PREFIXES)]
    missing = sorted(EXACT - set(selected))
    if missing:
        raise SystemExit(f"SDK source is missing required files: {missing}")
    return sorted(selected)


def blob(revision, path):
    return git("show", f"{revision}:{path}", binary=True)


def mode(revision, path):
    return 0o755 if git("ls-tree", revision, "--", path).split()[0] == "100755" else 0o644


def add_bytes(archive, name, data, file_mode):
    info = tarfile.TarInfo(name)
    info.size = len(data)
    info.mode = file_mode
    info.mtime = 0
    info.uid = info.gid = 0
    info.uname = info.gname = ""
    archive.addfile(info, io.BytesIO(data))


def build(output, revision):
    revision = git("rev-parse", revision).strip()
    tree = git("rev-parse", f"{revision}^{{tree}}").strip()
    manifest = tomllib.loads(blob(revision, "Cargo.toml").decode())
    version = manifest["package"]["version"]
    if version != "0.1.0":
        raise SystemExit(f"SDK requires the selected candidate version, found {version}")
    root = f"replai-c-sdk-{version}"
    metadata = {
        "schema": 1,
        "bundle": "replai-c-sdk-source",
        "version": version,
        "git_revision": revision,
        "git_tree": tree,
        "c_abi": 1,
        "archive_root": root,
    }
    files = selected_files(revision)
    output.parent.mkdir(parents=True, exist_ok=True)
    with output.open("wb") as raw:
        with gzip.GzipFile(filename="", mode="wb", fileobj=raw, mtime=0, compresslevel=9) as compressed:
            with tarfile.open(fileobj=compressed, mode="w", format=tarfile.PAX_FORMAT) as archive:
                for path in files:
                    add_bytes(archive, f"{root}/{path}", blob(revision, path), mode(revision, path))
                data = (json.dumps(metadata, indent=2, sort_keys=True) + "\n").encode()
                add_bytes(archive, f"{root}/SDK-METADATA.json", data, 0o644)
    digest = hashlib.sha256(output.read_bytes()).hexdigest()
    receipt = {
        **metadata,
        "filename": output.name,
        "bytes": output.stat().st_size,
        "sha256": digest,
        "file_count": len(files) + 1,
    }
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return receipt


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--revision", default="HEAD")
    parser.add_argument("--check-reproducible", action="store_true")
    args = parser.parse_args()
    first = build(args.output.resolve(), args.revision)
    if args.check_reproducible:
        with tempfile.TemporaryDirectory(prefix="replai-sdk-repro-") as directory:
            second_path = Path(directory) / args.output.name
            second = build(second_path, args.revision)
            if first["sha256"] != second["sha256"]:
                raise SystemExit("C SDK archive is not byte reproducible")
        print("C SDK byte reproducibility: PASS")


if __name__ == "__main__":
    main()
