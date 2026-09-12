#!/usr/bin/env python3
"""Drive the packaged debugger consumer through a real PTY."""
import argparse
import fcntl
import hashlib
import json
import os
from pathlib import Path
import pty
import select
import struct
import subprocess
import termios
import time


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    master, slave = pty.openpty()
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 80, 0, 0))
    process = subprocess.Popen(
        [str(args.binary.resolve())],
        stdin=slave,
        stdout=slave,
        stderr=subprocess.PIPE,
        close_fds=True,
        env={**os.environ, "NO_COLOR": "1", "TERM": "xterm-256color"},
    )
    os.close(slave)
    terminal = bytearray()
    receipts = bytearray()

    def pump(until=None, timeout=4.0):
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            if until is not None and until in terminal + receipts:
                return
            readers = [master]
            if process.stderr is not None:
                readers.append(process.stderr.fileno())
            ready, _, _ = select.select(readers, [], [], 0.05)
            for fd in ready:
                try:
                    data = os.read(fd, 65536)
                except OSError:
                    data = b""
                if fd == master:
                    terminal.extend(data)
                else:
                    receipts.extend(data)
            if process.poll() is not None and not ready:
                break
        if until is not None and until not in terminal + receipts:
            raise AssertionError(f"timeout waiting for {until!r}\n{receipts[-4000:]!r}")

    def send(data, marker=None, pause=0.06):
        os.write(master, data)
        time.sleep(pause)
        pump(marker, timeout=4.0 if marker is not None else 0.15)

    try:
        pump(b"debug> ")
        send(b"b", b"delayed-analysis-captured")
        send(b"r", b"delayed-stale-refused")
        assert b"fresh-analysis-applied" in receipts
        send(b"\x7f")  # return to a prefix with two ordered candidates
        send(b"\t", b"completion-presented")
        pump(b"module batch 2/2", timeout=3)
        assert receipts.count(b"serialized-notice-preserved") == 2
        send(b"\t")  # navigate to the second host-ordered candidate
        send(b"\r")  # completion acceptance only
        send(b"\x7f" * 9)
        send(b"watch }")
        send(b"\r", b"disposition-Invalid")
        assert process.poll() is None
        send(b"\x7f{")
        send(b"\r", b"disposition-Incomplete")
        send(b"counter > 0}")
        send(b"\r", b"disposition-Complete")
        pump(b"Fixture debugger result")
        pump(timeout=0.5)
        status = process.wait(timeout=4)
        assert status == 0
        assert b"ASSERT submitted watch {\\ncounter > 0}" in receipts
        assert b"reak main.rs:42" not in receipts.split(b"ASSERT submitted", 1)[1]
        result = {
            "schema": 1,
            "exit_status": status,
            "assertions": {
                "portable_stale_refusal": b"stale-analysis-refused" in receipts,
                "delayed_stale_refusal": b"delayed-stale-refused" in receipts,
                "fresh_analysis": b"fresh-analysis-applied" in receipts,
                "rich_completion": b"completion-presented" in receipts,
                "finite_serialized_notices": receipts.count(b"serialized-notice-preserved") == 2,
                "invalid": b"disposition-Invalid" in receipts,
                "incomplete": b"disposition-Incomplete" in receipts,
                "complete": b"disposition-Complete" in receipts,
                "structured_document": b"Fixture debugger result" in terminal,
            },
            "terminal_sha256": hashlib.sha256(terminal).hexdigest(),
            "receipts_sha256": hashlib.sha256(receipts).hexdigest(),
        }
        assert all(result["assertions"].values())
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")
        print("PASS packaged debugger: I0/I1/I2/I3, finite notices, document")
    finally:
        if process.poll() is None:
            process.kill()
            process.wait()
        os.close(master)


if __name__ == "__main__":
    main()
