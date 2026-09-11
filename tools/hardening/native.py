#!/usr/bin/env python3
"""Run counted public Rust/C native PTY lifecycles; blocking input is host-driven."""
import argparse
import errno
import fcntl
import hashlib
import json
import os
from pathlib import Path
import platform
import pty
import select
import struct
import subprocess
import termios
import time

ROOT = Path(__file__).resolve().parents[2]


def blocking(binary, cycles, work, prefix):
    master, slave = pty.openpty()
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 80, 0, 0))
    before = termios.tcgetattr(slave)
    transcript = bytearray()
    cursor = 0
    with (work / "blocking.stderr").open("wb") as errors:
        child = subprocess.Popen([*prefix, str(binary), "blocking", str(cycles)],
                                 stdin=slave, stdout=slave, stderr=errors,
                                 env={**os.environ, "TERM": "xterm-256color", "NO_COLOR": "1"})

        def await_text(token):
            nonlocal cursor
            deadline = time.monotonic()+30
            while token not in transcript[cursor:]:
                assert time.monotonic() < deadline, ("blocking timeout", token, child.poll())
                if select.select([master], [], [], .1)[0]:
                    try:
                        data = os.read(master, 65536)
                    except OSError as error:
                        if error.errno == errno.EIO:
                            raise AssertionError("child exited before receipt") from error
                        raise
                    assert data, "EOF before receipt"
                    transcript.extend(data)
                assert token in transcript[cursor:] or child.poll() is None, "child exited before receipt"
            cursor = transcript.index(token, cursor)+len(token)

        try:
            for i in range(cycles):
                await_text(b"block> ")
                os.write(master, b"\x1b[200~"+f"case{i} 界\nline".encode()+b"\x1b[201~\r")
                await_text(f"RECEIPT {i}\r\n".encode())
            assert child.wait(timeout=30) == 0
            while select.select([master], [], [], .05)[0]:
                try:
                    part = os.read(master, 65536)
                except OSError as error:
                    if error.errno == errno.EIO:
                        break
                    raise
                if not part:
                    break
                transcript.extend(part)
            assert termios.tcgetattr(slave) == before, "parent restoration oracle"
        finally:
            if child.poll() is None:
                child.kill()
                child.wait()
            os.close(master)
            os.close(slave)
    (work / "blocking.pty").write_bytes(transcript)
    if prefix and platform.system() == "Darwin":
        assert b"0 leaks for 0 total leaked bytes" in transcript+(work / "blocking.stderr").read_bytes()
    return {"tier": "blocking", "cycles": cycles, "parent_termios_exact": True,
            "terminal_bytes": len(transcript)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--work", type=Path, required=True)
    parser.add_argument("--cycles", type=int, default=1000)
    parser.add_argument("--events", type=int, default=10000)
    parser.add_argument("--failure-repeats", type=int, default=100)
    parser.add_argument("--memory", action="store_true")
    args = parser.parse_args()
    assert platform.system() in ("Linux", "Darwin")
    assert 1 <= args.cycles <= 10000 and 1 <= args.events <= 100000
    args.work.mkdir(parents=True, exist_ok=False)
    subprocess.run(["cargo", "build", "--locked", "--release", "--manifest-path", "tools/hardening/Cargo.toml", "--bin", "native"], cwd=ROOT, check=True)
    binary = ROOT / "tools/hardening/target/release/native"
    prefix = []
    if args.memory:
        prefix = (["valgrind", "--leak-check=full", "--show-leak-kinds=all",
                   "--errors-for-leak-kinds=definite,indirect,possible", "--error-exitcode=97"]
                  if platform.system() == "Linux" else ["/usr/bin/leaks", "--atExit", "--"])
    summary = {"head": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
               "tree": subprocess.check_output(["git", "rev-parse", "HEAD^{tree}"], cwd=ROOT, text=True).strip(),
               "environment": platform.uname()._asdict(), "memory": args.memory,
               "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
               "rustc": subprocess.check_output(["rustc", "-vV"], text=True).strip(),
               "started_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
               "results": []}
    (args.work / "summary.json").write_text(json.dumps(summary, indent=2)+"\n")
    summary["results"].append(blocking(binary, args.cycles, args.work, prefix))
    for tier, count in [("session", args.cycles), ("driven", args.cycles), ("cabi", args.cycles), ("mixed", args.events), ("failures", args.failure_repeats), ("exhaustion", args.failure_repeats)]:
        with (args.work / f"{tier}.stderr").open("w") as errors:
            # The bounded NOFILE child has its own gate; a memory tool's private
            # descriptors must not be mistaken for product descriptors/limits.
            instrument = [] if tier == "exhaustion" else prefix
            result = subprocess.run([*instrument, str(binary), tier, str(count)], cwd=ROOT, text=True,
                                    stdout=subprocess.PIPE, stderr=errors, timeout=1800,
                                    env={**os.environ, "TERM": "xterm-256color"})
        (args.work / f"{tier}.stdout").write_text(result.stdout)
        assert result.returncode == 0, (tier, result.returncode)
        if instrument and platform.system() == "Darwin":
            assert "0 leaks for 0 total leaked bytes" in result.stdout+(args.work / f"{tier}.stderr").read_text()
        rows = [json.loads(line) for line in result.stdout.splitlines() if line.startswith("{")]
        assert len(rows) == 1
        summary["results"].append(rows[0])
        (args.work / "summary.json").write_text(json.dumps(summary, indent=2)+"\n")
        print(tier, summary["results"][-1], flush=True)


if __name__ == "__main__":
    main()
