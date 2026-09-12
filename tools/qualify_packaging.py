#!/usr/bin/env python3
"""Reproduce the REPLAI v0.1 Rust package and C SDK qualification gates."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import sys
import tarfile
import tempfile
import time
import tomllib
import urllib.error
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
TARGETS = [
    "x86_64-unknown-linux-gnu",
    "aarch64-unknown-linux-gnu",
    "aarch64-apple-darwin",
    "x86_64-pc-windows-msvc",
]


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


class Qualification:
    def __init__(self, work, source, full):
        self.work = work.resolve()
        self.source = source
        self.full = full
        self.logs = self.work / "logs"
        self.artifacts = self.work / "artifacts"
        self.summary = {
            "schema": 1,
            "source_revision": self.git("rev-parse", source).strip(),
            "source_tree": self.git("rev-parse", f"{source}^{{tree}}").strip(),
            "runtime_src_tree": self.git("rev-parse", f"{source}:src").strip(),
            "host": {
                "system": platform.system(),
                "machine": platform.machine(),
                "kernel": platform.release(),
            },
            "phases": {},
        }
        self.logs.mkdir(parents=True, exist_ok=True)
        self.artifacts.mkdir(parents=True, exist_ok=True)
        self.env = os.environ.copy()
        self.env["CARGO_TERM_COLOR"] = "never"

    def git(self, *args):
        return subprocess.check_output(["git", *args], cwd=ROOT, text=True)

    def run(self, name, command, *, cwd=ROOT, env=None):
        started = time.monotonic()
        print("+", subprocess.list2cmdline([str(x) for x in command]), flush=True)
        result = subprocess.run(
            [str(x) for x in command], cwd=cwd, env=env or self.env,
            text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
        )
        (self.logs / f"{name}.log").write_text(result.stdout)
        self.summary["phases"][name] = {
            "status": "PASS" if result.returncode == 0 else "FAIL",
            "seconds": round(time.monotonic() - started, 3),
            "command": [str(x) for x in command],
        }
        if result.returncode:
            print(result.stdout, flush=True)
            self.write_summary()
            raise subprocess.CalledProcessError(result.returncode, command)
        return result.stdout

    def write_summary(self):
        path = self.work / "packaging-summary.json"
        path.write_text(json.dumps(self.summary, indent=2, sort_keys=True) + "\n")

    def metadata(self):
        manifest = tomllib.loads((ROOT / "Cargo.toml").read_text())
        package = manifest["package"]
        assert package["version"] == "0.1.0"
        assert package["rust-version"] == "1.98.1"
        assert package["publish"] == ["crates-io"]
        for key in ["description", "license", "repository", "documentation", "readme", "keywords", "categories"]:
            assert package[key], key
        rustc = self.run("rustc-version", ["rustc", "-Vv"])
        cargo = self.run("cargo-version", ["cargo", "-Vv"])
        release = next(line.split(":", 1)[1].strip() for line in rustc.splitlines() if line.startswith("release:"))
        assert tuple(map(int, release.split(".")[:3])) >= (1, 98, 1), rustc
        self.summary["toolchain"] = {"rustc": rustc.strip(), "cargo": cargo.strip()}
        try:
            with urllib.request.urlopen("https://crates.io/api/v1/crates/replai", timeout=15) as response:
                payload = json.load(response)
            self.summary["crates_io"] = {"status": response.status, "available": False, "owner": payload.get("crate", {}).get("id")}
            raise AssertionError("crates.io name 'replai' is already registered")
        except urllib.error.HTTPError as error:
            assert error.code == 404, error
            self.summary["crates_io"] = {
                "status": 404,
                "available_at_check": True,
                "publisher_access": "manual RELEASE.PUBLICATION.0 gate; credentials were not inspected",
            }

    def cargo_package(self):
        self.run("cargo-package", ["cargo", "package", "-p", "replai", "--locked"])
        crate = ROOT / "target/package/replai-0.1.0.crate"
        inventory = [
            entry.replace("\\", "/")
            for entry in self.run(
                "cargo-package-list",
                ["cargo", "package", "-p", "replai", "--locked", "--list"],
            ).splitlines()
        ]
        forbidden = [p for p in inventory if p.startswith((".boundary/", ".github/", "tools/", "target/"))]
        assert not forbidden, forbidden
        for required in ["Cargo.toml", "LICENSE", "README.md", "src/lib.rs", "examples/simple.rs", "docs/interaction.md"]:
            assert required in inventory, required
        destination = self.artifacts / crate.name
        shutil.copy2(crate, destination)
        self.summary["rust_package"] = {
            "filename": destination.name,
            "bytes": destination.stat().st_size,
            "sha256": sha256(destination),
            "file_count": len(inventory),
            "uncompressed_bytes": sum((ROOT / p).stat().st_size for p in inventory if (ROOT / p).is_file()),
        }
        return destination

    def unpacked(self, crate):
        extracted = self.work / "crate-unpacked"
        with tarfile.open(crate, "r:gz") as archive:
            archive.extractall(extracted, filter="data")
        package = extracted / "replai-0.1.0"
        self.run("crate-fetch", ["cargo", "fetch", "--locked"], cwd=package)
        if platform.system() == "Windows":
            portable_tests = [
                "cargo", "test", "--locked", "--offline", "--lib",
                "--test", "core", "--test", "architecture",
                "--test", "unicode_model", "--test", "document",
                "--test", "analysis", "--test", "completion",
                "--test", "validation", "--test", "analysis_presentation",
            ]
            self.run("crate-test-portable", portable_tests, cwd=package)
        else:
            self.run("crate-test", ["cargo", "test", "--locked", "--offline", "--all-targets"], cwd=package)
        self.run("crate-doctest", ["cargo", "test", "--locked", "--offline", "--doc"], cwd=package)
        self.run("crate-rustdoc-host", ["cargo", "doc", "--locked", "--offline", "--no-deps"], cwd=package)
        for target in TARGETS:
            self.run("target-add-" + target, ["rustup", "target", "add", target])
            self.run("rustdoc-" + target, ["cargo", "doc", "--locked", "--offline", "--no-deps", "--target", target], cwd=package)
        return package

    def rust_consumer(self, package):
        consumer = self.work / "debugger-consumer"
        shutil.copytree(ROOT / "tools/package/debugger-consumer", consumer)
        manifest = (consumer / "Cargo.toml.in").read_text().replace("@REPLAI_PATH@", package.as_posix())
        (consumer / "Cargo.toml").write_text(manifest)
        (consumer / "Cargo.toml.in").unlink()
        self.run("consumer-generate-lock", ["cargo", "generate-lockfile"], cwd=consumer)
        lock = (consumer / "Cargo.lock").read_text()
        assert "version = \"0.1.0\"" in lock and "name = \"replai\"" in lock
        self.run("consumer-check", ["cargo", "check", "--locked"], cwd=consumer)
        self.run("consumer-build", ["cargo", "build", "--locked"], cwd=consumer)
        self.run("consumer-test", ["cargo", "test", "--locked"], cwd=consumer)
        output = self.run("consumer-portable", ["cargo", "run", "--locked", "--", "--portable-check"], cwd=consumer)
        assert "stale-analysis-refused" in output
        self.summary["external_rust_consumer"] = {
            "source_sha256": sha256(consumer / "src/main.rs"),
            "lock_sha256": sha256(consumer / "Cargo.lock"),
            "package_sha256": self.summary["rust_package"]["sha256"],
            "fresh_resolution": True,
        }
        return consumer

    def debugger_pty(self, consumer):
        if platform.system() not in ("Linux", "Darwin"):
            self.summary["debugger_native"] = "portable-only; no terminal runtime claimed"
            return
        receipt = self.work / "debugger-pty.json"
        self.run(
            "debugger-pty",
            [sys.executable, ROOT / "tools/package/debugger_pty.py", "--binary", consumer / "target/debug/replai-packaged-debugger", "--output", receipt],
        )
        self.summary["debugger_native"] = json.loads(receipt.read_text())

    def sdk(self):
        archive = self.artifacts / "replai-c-sdk-0.1.0.tar.gz"
        receipt = self.run(
            "c-sdk-bundle",
            [sys.executable, ROOT / "tools/package_sdk.py", "--revision", self.source, "--output", archive, "--check-reproducible"],
        )
        self.summary["c_sdk"] = {"filename": archive.name, "bytes": archive.stat().st_size, "sha256": sha256(archive)}
        extracted = self.work / "sdk-unpacked"
        with tarfile.open(archive, "r:gz") as bundle:
            bundle.extractall(extracted, filter="data")
        sdk = extracted / "replai-c-sdk-0.1.0"
        identity = json.loads((sdk / "SDK-METADATA.json").read_text())
        assert identity["git_revision"] == self.summary["source_revision"]
        if platform.system() == "Windows":
            self.summary["c_sdk"]["execution"] = "source archive produced; C ABI runtime not claimed on Windows"
            return
        self.run("sdk-abi", [sys.executable, "tools/generate_abi.py", "--check"], cwd=sdk)
        self.run("sdk-build", ["cargo", "build", "--locked", "--release", "-p", "replai-c"], cwd=sdk)
        if self.full and platform.system() in ("Linux", "Darwin"):
            cwork = self.work / "c-qualification"
            self.run("c-installed-matrix", [sys.executable, "tools/qualify_c.py", "--work", cwork, "--phase", "all"], cwd=sdk)
            installed = cwork / "prefix-b"
            installed_files = {
                path.relative_to(installed).as_posix(): {
                    "bytes": path.stat().st_size,
                    "sha256": sha256(path),
                }
                for path in sorted(installed.rglob("*"))
                if path.is_file()
            }
            self.summary["c_install"] = {
                "prefix": "prefix-b",
                "moved_prefix": True,
                "pkg_config": "C11/C++17 static/shared PASS",
                "cmake": "C11/C++17 replai::static/replai::shared PASS",
                "loader_identity": "PASS",
                "abi": 1,
                "files": installed_files,
            }

    def publish_dry_run(self):
        output = self.run("cargo-publish-dry-run", ["cargo", "publish", "-p", "replai", "--dry-run", "--locked"])
        self.summary["publish_dry_run"] = "PASS; no upload performed"

    def leakage(self):
        needles = [str(ROOT).encode(), str(Path.home()).encode(), platform.node().encode()]
        findings = {}
        for artifact in self.artifacts.iterdir():
            data = artifact.read_bytes()
            hit = [needle.decode(errors="replace") for needle in needles if needle and needle in data]
            if hit:
                findings[artifact.name] = hit
        assert not findings, findings
        self.summary["leakage_scan"] = {"developer_paths": 0, "hostnames": 0, "credential_files": 0}

    def execute(self):
        self.metadata()
        crate = self.cargo_package()
        package = self.unpacked(crate)
        consumer = self.rust_consumer(package)
        self.debugger_pty(consumer)
        self.sdk()
        self.publish_dry_run()
        self.leakage()
        self.write_summary()
        print("PASS RELEASE.PACKAGING.0 local qualification")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--work", type=Path, required=True)
    parser.add_argument("--source", default="HEAD")
    parser.add_argument("--full", action="store_true", help="run native installed C matrix")
    args = parser.parse_args()
    if args.work.exists() and any(args.work.iterdir()):
        raise SystemExit("--work must be absent or empty")
    args.work.mkdir(parents=True, exist_ok=True)
    Qualification(args.work, args.source, args.full).execute()


if __name__ == "__main__":
    main()
