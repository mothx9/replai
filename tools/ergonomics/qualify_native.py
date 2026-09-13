#!/usr/bin/env python3
"""Qualify the interaction-ergonomics delta on one exact native runner."""
import argparse
import json
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import time

ROOT = Path(__file__).resolve().parents[2]
def run(command, **kwargs):
    return subprocess.run(command, cwd=ROOT, check=True, text=True, **kwargs)


def pty_binary():
    result = run(
        ["cargo", "build", "--locked", "--release", "--manifest-path",
         "tools/hardening/Cargo.toml", "--bin", "ergonomics-pty", "--message-format", "json"],
        stdout=subprocess.PIPE,
        timeout=300,
    )
    artifacts = [json.loads(line) for line in result.stdout.splitlines() if line.startswith("{")]
    paths = [Path(item["executable"]) for item in artifacts
             if item.get("reason") == "compiler-artifact"
             and item.get("target", {}).get("name") == "ergonomics-pty"
             and item.get("executable")]
    assert len(paths) == 1, paths
    return paths[0]


def validate_valgrind(report):
    assert "definitely lost: 0 bytes" in report
    assert "indirectly lost: 0 bytes" in report
    for marker in ("Invalid read", "Invalid write", "Invalid free", "Mismatched free"):
        assert marker not in report, marker
    possible = re.findall(
        r"\n==\d+== \d+ bytes in \d+ blocks are possibly lost.*?"
        r"(?=\n==\d+== (?:\d+ bytes|LEAK SUMMARY:))",
        report,
        re.DOTALL,
    )
    for record in possible:
        # Rust's libtest runner leaves one process-lifetime TLS context reachable
        # from std::sync::mpmc. This exact non-product stack is not suppressed;
        # it remains visible in the retained log and every other possible-loss
        # record is rejected.
        assert "48 bytes in 1 blocks are possibly lost" in record, record
        assert "std::thread::current::init_current" in record, record
        assert "std::sync::mpmc::context::Context" in record, record
        assert "/src/core.rs" not in record and "/src/engine.rs" not in record
        assert "/src/history.rs" not in record and "/src/interaction.rs" not in record
    return len(possible)


def macos_leaks_example(work):
    """Drive a public blocking host under leaks from an external PTY owner."""
    import fcntl
    import select
    import struct
    import termios

    run(["cargo", "build", "--locked", "--example", "simple"], timeout=300)
    master, slave = os.openpty()
    before = termios.tcgetattr(slave)
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 12, 80, 0, 0))
    os.set_blocking(master, False)
    command = ["/usr/bin/leaks", "--atExit", "--", str(ROOT / "target/debug/examples/simple")]
    child = subprocess.Popen(
        command,
        cwd=ROOT,
        stdin=slave,
        stdout=slave,
        stderr=slave,
        env={**os.environ, "TERM": "xterm-256color"},
    )
    output = bytearray()

    def read_until(marker, start=0, timeout=15):
        deadline = time.monotonic() + timeout
        while marker not in output[start:]:
            assert time.monotonic() < deadline, (marker, bytes(output[-2000:]))
            ready, _, _ = select.select([master], [], [], 0.1)
            if ready:
                try:
                    output.extend(os.read(master, 65536))
                except OSError as error:
                    if error.errno != 5:  # EIO after the PTY peer exits.
                        raise
            assert child.poll() is None, (child.returncode, bytes(output[-2000:]))

    try:
        read_until(b"simple>")
        os.write(master, b"older alpha command\r")
        first = len(output)
        read_until(b"simple>", first)

        os.write(master, b"draft\x12alpha")
        read_until(b"? 'alpha' >")
        os.write(master, b"\x1b")
        time.sleep(0.35)
        os.write(master, b"\x12alpha\r\x1f\x17\x19\x03")
        second = len(output)
        read_until(b"simple>", second)
        os.write(master, b"\x04")
        # `leaks --atExit` writes a report large enough to fill a PTY. Drain it
        # while waiting so the instrumented child cannot block during cleanup.
        deadline = time.monotonic() + 30
        while child.poll() is None:
            assert time.monotonic() < deadline, bytes(output[-4000:])
            ready, _, _ = select.select([master], [], [], 0.1)
            if not ready:
                continue
            try:
                output.extend(os.read(master, 65536))
            except OSError as error:
                if error.errno != 5:
                    raise
                break
        child.wait(timeout=1)
        for _ in range(10):
            ready, _, _ = select.select([master], [], [], 0.05)
            if not ready:
                break
            try:
                output.extend(os.read(master, 65536))
            except OSError as error:
                if error.errno != 5:
                    raise
                break
        combined = bytes(output)
        assert child.returncode == 0, (child.returncode, combined[-4000:])
        assert b"? 'alpha' >" in output
        assert b"0 leaks for 0 total leaked bytes" in combined
        assert termios.tcgetattr(slave) == before
        (work / "leaks.txt").write_bytes(combined)
        return command
    finally:
        if child.poll() is None:
            child.kill()
            child.wait()
        os.close(master)
        os.close(slave)


def macos_leaks_ergonomics(work, executable):
    """Run the self-contained new-surface PTY oracle under native leaks."""
    command = ["/usr/bin/leaks", "--atExit", "--", str(executable)]
    # `leaks --atExit` can spend substantially longer than the executable in
    # native heap analysis on hosted ARM64 runners. Keep the command bounded,
    # but give the memory tool the same budget as the Linux Valgrind pass.
    result = subprocess.run(command, cwd=ROOT, text=True, stdout=subprocess.PIPE,
                            stderr=subprocess.STDOUT, timeout=300)
    (work / "leaks-ergonomics.txt").write_text(result.stdout)
    assert result.returncode == 0, result.stdout[-4000:]
    assert "0 leaks for 0 total leaked bytes" in result.stdout, result.stdout[-4000:]
    assert '"suggestion":true' in result.stdout and '"large_completion":1000' in result.stdout
    return command


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--work", type=Path, required=True)
    parser.add_argument("--portable-only", action="store_true")
    args = parser.parse_args()
    args.work.mkdir(parents=True, exist_ok=False)
    started = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
    run(["cargo", "test", "--locked", "--test", "ergonomics"])
    run(["cargo", "test", "--locked", "--test", "configuration_helpers"])
    run(["cargo", "run", "--locked", "--release", "--manifest-path",
         "tools/hardening/Cargo.toml", "--bin", "ergonomics-sequences", "--", "1000", "7632459182231841"])
    run(["cargo", "run", "--locked", "--release", "--manifest-path",
         "tools/hardening/Cargo.toml", "--bin", "completion-sequences", "--", "1000", "15579347612220529"])
    summary = {
        "head": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
        "tree": subprocess.check_output(["git", "rev-parse", "HEAD^{tree}"], cwd=ROOT, text=True).strip(),
        "runtime_tree": subprocess.check_output(["git", "rev-parse", "HEAD:src"], cwd=ROOT, text=True).strip(),
        "platform": platform.platform(), "machine": platform.machine(), "started_utc": started,
        "rustc": subprocess.check_output(["rustc", "-vV"], text=True).strip(),
        "portable_model_sequences": 2000, "portable_tests": "PASS",
    }
    if not args.portable_only:
        executable = pty_binary()
        run([str(executable)], timeout=120)
        if platform.system() == "Linux":
            valgrind = shutil.which("valgrind")
            assert valgrind, "Valgrind is required on Linux"
            log = args.work / "valgrind.log"
            run([valgrind, "--leak-check=full", "--show-leak-kinds=all",
                 "--errors-for-leak-kinds=definite,indirect", "--error-exitcode=99",
                 f"--log-file={log}", str(executable)],
                timeout=300)
            report = log.read_text()
            possible_harness_records = validate_valgrind(report)
            summary["memory_tool"] = subprocess.check_output(
                [valgrind, "--version"], text=True
            ).strip()
            summary["non_attributable_harness_possible_records"] = possible_harness_records
        elif platform.system() == "Darwin":
            leaks = Path("/usr/bin/leaks")
            assert leaks.is_file(), "native leaks is required on macOS"
            memory_command = [
                macos_leaks_example(args.work),
                macos_leaks_ergonomics(args.work, executable),
            ]
            summary["memory_tool"] = "macOS leaks --atExit"
            summary["memory_command"] = memory_command
        else:
            raise AssertionError("native terminal qualification supports Linux/macOS only")
        summary.update(real_pty="PASS", memory="PASS", widths=[20, 40, 80, 132],
                       modes=["styled", "plain"])
    summary["ended_utc"] = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
    (args.work / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
