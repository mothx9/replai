#!/usr/bin/env python3
"""Run counted public Rust/C native PTY lifecycles; blocking input is host-driven."""
import argparse
import errno
import fcntl
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
            assert termios.tcgetattr(slave) == before, "parent restoration oracle"
        finally:
            if child.poll() is None:
                child.kill()
                child.wait()
            os.close(master)
            os.close(slave)
    (work / "blocking.pty").write_bytes(transcript)
    return {"tier": "blocking", "cycles": cycles, "parent_termios_exact": True,
            "terminal_bytes": len(transcript)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--work", type=Path, required=True)
    parser.add_argument("--cycles", type=int, default=1000)
    parser.add_argument("--events", type=int, default=10000)
    parser.add_argument("--memory", action="store_true")
    args = parser.parse_args()
    assert platform.system() in ("Linux", "Darwin")
    assert 1 <= args.cycles <= 10000 and 1 <= args.events <= 100000
    args.work.mkdir(parents=True, exist_ok=False)
    subprocess.run(["cargo", "build", "--locked", "--release", "--manifest-path", "tools/hardening/Cargo.toml", "--bin", "native"], cwd=ROOT, check=True)
    binary = ROOT / "tools/hardening/target/release/native"
    prefix = []
    if args.memory:
        assert platform.system() == "Linux", "macOS uses its native leaks campaign separately"
        prefix = ["valgrind", "--leak-check=full", "--show-leak-kinds=all",
                  "--errors-for-leak-kinds=definite,indirect,possible", "--error-exitcode=97"]
    summary = {"head": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
               "tree": subprocess.check_output(["git", "rev-parse", "HEAD^{tree}"], cwd=ROOT, text=True).strip(),
               "environment": platform.uname()._asdict(), "memory": args.memory, "results": []}
    summary["results"].append(blocking(binary, args.cycles, args.work, prefix))
    for tier, count in [("session", args.cycles), ("driven", args.cycles), ("cabi", args.cycles), ("mixed", args.events)]:
        with (args.work / f"{tier}.stderr").open("w") as errors:
            result = subprocess.run([*prefix, str(binary), tier, str(count)], cwd=ROOT, text=True,
                                    stdout=subprocess.PIPE, stderr=errors, timeout=1800,
                                    env={**os.environ, "TERM": "xterm-256color"})
        (args.work / f"{tier}.stdout").write_text(result.stdout)
        assert result.returncode == 0, (tier, result.returncode)
        summary["results"].append(json.loads(result.stdout))
        (args.work / "summary.json").write_text(json.dumps(summary, indent=2)+"\n")
        print(tier, summary["results"][-1], flush=True)


if __name__ == "__main__":
    main()
