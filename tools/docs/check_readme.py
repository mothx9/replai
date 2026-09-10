#!/usr/bin/env python3
"""Compile README Rust/C code; run its blocking host and exact plain transcript.

Documentation control only. Build the checkout's library, never an adjacent copy.
The functions without main are embedding excerpts compiled with an empty main.
"""
import fcntl
import json
import os
from pathlib import Path
import pty
import re
import select
import struct
import subprocess
import tempfile
import termios
import time

ROOT = Path(__file__).resolve().parents[2]


def blocking(binary):
    master, slave = pty.openpty()
    before = termios.tcgetattr(slave)
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 80, 0, 0))
    child = subprocess.Popen([binary], stdin=slave, stdout=slave, stderr=slave,
                             env={**os.environ, "TERM": "xterm-256color", "NO_COLOR": "1"})
    output = bytearray()

    def until(needle, start=0):
        deadline = time.monotonic() + 8
        while needle not in output[start:]:
            assert time.monotonic() < deadline, output.decode(errors="replace")
            if select.select([master], [], [], 0.05)[0]:
                output.extend(os.read(master, 65536))

    try:
        until(b"demo> ")
        os.write(master, b"ping\r")
        until(b"received: ping")
        until(b"demo> ", output.index(b"received: ping"))
        mark = len(output)
        os.write(master, b"\x03")
        until(b"demo> ", mark)
        os.write(master, b"\x04")
        assert child.wait(timeout=8) == 0
        assert termios.tcgetattr(slave) == before
        # Drain final mode cleanup; the slave remains ours until after observation.
        while select.select([master], [], [], 0)[0]:
            output.extend(os.read(master, 65536))
        assert output.rfind(b"\x1b[?2004l") > output.rfind(b"\x1b[?2004h")
    finally:
        if child.poll() is None:
            child.kill()
            child.wait()
        os.close(master)
        os.close(slave)


def main():
    built = subprocess.check_output([
        "cargo", "build", "--locked", "-p", "replai", "--message-format=json"
    ], cwd=ROOT, text=True)
    artifacts = [json.loads(line) for line in built.splitlines()]
    libraries = [Path(f) for a in artifacts if a.get("reason") == "compiler-artifact"
                 and a["target"]["name"] == "replai"
                 for f in a["filenames"] if f.endswith(".rlib")]
    assert len(libraries) == 1, libraries
    library = libraries[0]
    dependencies = library.parent / "deps"
    blocks = re.findall(r"^```(\w+)\n(.*?)^```", (ROOT / "README.md").read_text(), re.M | re.S)
    counts = {"rust": 0, "c": 0}
    with tempfile.TemporaryDirectory(prefix="replai-readme-") as directory:
        work = Path(directory)
        for index, (language, code) in enumerate(blocks):
            if language not in counts:
                continue
            counts[language] += 1
            source = work / f"snippet_{index}.{ 'rs' if language == 'rust' else 'c'}"
            if language == "c":
                source.write_text(code)
                subprocess.run(["cc", "-std=c11", "-Wall", "-Wextra", "-Werror",
                                "-I", str(ROOT / "include"), "-c", str(source),
                                "-o", str(work / "consumer.o")], check=True)
                continue
            has_main = "fn main(" in code
            source.write_text("#![allow(dead_code)]\n" + code + ("" if has_main else "\nfn main() {}\n"))
            binary = work / f"snippet_{index}"
            subprocess.run(["rustc", "--edition=2024", "-D", "warnings", str(source),
                            "--extern", f"replai={library}", "-L", f"dependency={dependencies}",
                            "-o", str(binary)], check=True)
            if "input.read_line(" in code:
                blocking(binary)
            elif has_main:
                actual = subprocess.check_output([binary], text=True)
                assert blocks[index + 1][0] == "text"
                assert actual.strip() == blocks[index + 1][1].strip(), repr(actual)
    assert counts == {"rust": 4, "c": 1}, counts
    print("PASS README: 4 Rust snippets, 1 C snippet, blocking PTY (submit/interrupt/EOF/restoration), exact Document output")


if __name__ == "__main__":
    main()
