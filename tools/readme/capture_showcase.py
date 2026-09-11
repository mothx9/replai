#!/usr/bin/env python3
"""Capture the public showcase through a real PTY and render PNG/GIF assets."""
import argparse
import copy
import fcntl
import hashlib
import os
from pathlib import Path
import pty
import select
import struct
import subprocess
import tempfile
import termios
import time

import pyte
from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "assets/readme"
COLUMNS = 84
ROWS = 27
FONT_PATH = Path("/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf")


class Showcase:
    def __init__(self):
        self.master, self.slave = pty.openpty()
        fcntl.ioctl(self.slave, termios.TIOCSWINSZ, struct.pack("HHHH", ROWS, COLUMNS, 0, 0))
        self.before = termios.tcgetattr(self.slave)
        env = {**os.environ, "TERM": "xterm-256color"}
        env.pop("NO_COLOR", None)
        self.process = subprocess.Popen(
            [str(ROOT / "target/debug/examples/showcase")],
            stdin=self.slave,
            stdout=self.slave,
            stderr=self.slave,
            env=env,
            cwd=ROOT,
        )
        self.screen = pyte.Screen(COLUMNS, ROWS)
        self.stream = pyte.ByteStream(self.screen)
        self.raw = bytearray()

    def drain(self, seconds=0.08):
        until = time.monotonic() + seconds
        while time.monotonic() < until:
            ready = select.select([self.master], [], [], max(0, until - time.monotonic()))[0]
            if ready:
                data = os.read(self.master, 65536)
                self.raw.extend(data)
                self.stream.feed(data)

    def send(self, data, seconds=0.08):
        assert os.write(self.master, data) == len(data)
        self.drain(seconds)

    def until(self, text, timeout=5):
        deadline = time.monotonic() + timeout
        while text not in self.text():
            assert time.monotonic() < deadline, self.text()
            assert self.process.poll() is None, self.text()
            self.drain(0.03)

    def text(self):
        return "\n".join(row.rstrip() for row in self.screen.display).rstrip()

    def snap(self):
        return copy.deepcopy(self.screen)

    def close(self):
        try:
            if self.process.poll() is None:
                self.send(b"\x03")
                self.send(b"\x04")
            assert self.process.wait(timeout=5) == 0
            assert termios.tcgetattr(self.slave) == self.before
            assert self.raw.rfind(b"\x1b[?2004l") > self.raw.rfind(b"\x1b[?2004h")
        finally:
            if self.process.poll() is None:
                self.process.terminate()
                self.process.wait(timeout=5)
            os.close(self.master)
            os.close(self.slave)


def type_bytes(session, value, frames, delay=150):
    for byte in value:
        session.send(bytes([byte]))
        frames.append((session.snap(), delay))


def static_session():
    session = Showcase()
    try:
        session.until("ops[local]>")
        session.send("deploy {".encode())
        session.send(b"\r\t")
        session.send("service café".encode())
        session.send(b"\r}\r")
        session.until("Host accepted the validated draft")
        session.send(b"de\t\t")
        session.until("> describe")
        cursor_column = session.screen.cursor.x
        session.until("fixture inventory refreshed")
        assert "> describe" in session.text()
        assert session.screen.cursor.x == cursor_column
        assert "Local fixture result" in session.text()
        return session.snap()
    finally:
        session.close()


def animated_session():
    session = Showcase()
    frames = []
    try:
        session.until("ops[local]>")
        frames.append((session.snap(), 650))
        # A complete UTF-8 grapheme is inserted/deleted before the visible prefix.
        type_bytes(session, "dé".encode(), frames)
        session.send(b"\x7f")
        frames.append((session.snap(), 180))
        session.send(b"e")
        frames.append((session.snap(), 800))
        assert "[~ploy · Tab for operations]" in session.text()
        session.send(b"\t")
        session.until("> deploy")
        frames.append((session.snap(), 800))
        session.send(b"\t")
        session.until("> describe")
        frames.append((session.snap(), 750))
        cursor_column = session.screen.cursor.x
        session.until("fixture inventory refreshed")
        assert session.screen.cursor.x == cursor_column
        frames.append((session.snap(), 950))
        session.send(b"\r")
        frames.append((session.snap(), 350))
        session.send(b"\r")
        session.until("Host accepted the validated draft")
        frames.append((session.snap(), 1100))

        type_bytes(session, b"deploy {", frames, 90)
        session.send(b"\r\t")
        frames.append((session.snap(), 420))
        type_bytes(session, "service café".encode(), frames, 70)
        session.send(b"\r}")
        frames.append((session.snap(), 650))
        session.send(b"\r")
        session.until("Host accepted the validated draft")
        frames.append((session.snap(), 950))
        session.send(b"\x1b[A")
        session.until("service café")
        frames.append((session.snap(), 750))
        session.send(b"\x03", seconds=0.2)
        session.send(b"}\r")
        session.until("Unmatched closing brace in local plan")
        frames.append((session.snap(), 1200))
        return frames
    finally:
        session.close()


def render_screen(screen):
    font = ImageFont.truetype(str(FONT_PATH), 20)
    bold = ImageFont.truetype(str(FONT_PATH).replace(".ttf", "-Bold.ttf"), 20)
    title = ImageFont.truetype(str(FONT_PATH), 17)
    cw, ch, pad, top = 12, 29, 32, 76
    canvas = Image.new("RGB", (COLUMNS * cw + 2 * pad, ROWS * ch + top + pad), "#181b21")
    draw = ImageDraw.Draw(canvas)
    draw.text((pad, 22), "REPLAI / PUBLIC API SHOWCASE", font=title, fill="#b69af8")
    draw.line((pad, 56, COLUMNS * cw + pad, 56), fill="#393f4b", width=1)
    colors = {
        "default": "#dce2eb", "black": "#17191e", "red": "#e58b8b",
        "green": "#99bf95", "brown": "#dfbd80", "blue": "#90b2df",
        "magenta": "#c5a2d7", "cyan": "#92c7ce", "white": "#e4e8ef",
        "brightblack": "#8893a4",
    }
    for y in range(ROWS):
        for x in range(COLUMNS):
            cell = screen.buffer[y][x]
            if cell.data.strip():
                color = colors.get(cell.fg, "#" + cell.fg if len(cell.fg) == 6 else "#dce2eb")
                draw.text((pad + x * cw, top + y * ch), cell.data,
                          font=bold if cell.bold else font, fill=color)
    if not screen.cursor.hidden:
        x, y = screen.cursor.x, screen.cursor.y
        draw.rectangle((pad + x * cw, top + y * ch + 2,
                        pad + (x + 1) * cw - 1, top + (y + 1) * ch - 2),
                       outline="#dce2eb", width=2)
    return canvas


def build(destination):
    destination.mkdir(parents=True, exist_ok=True)
    static = render_screen(static_session())
    static.save(destination / "terminal-showcase.png", optimize=True)
    recorded = animated_session()
    frames = [render_screen(screen).quantize(colors=48, method=Image.Quantize.MEDIANCUT,
                                             dither=Image.Dither.NONE)
              for screen, _ in recorded]
    durations = [duration for _, duration in recorded]
    frames[0].save(destination / "terminal-showcase.gif", save_all=True,
                   append_images=frames[1:], duration=durations, loop=0,
                   optimize=True, disposal=2)
    encoded = Image.open(destination / "terminal-showcase.gif")
    return {
        "frames": encoded.n_frames,
        "duration_ms": sum(durations),
        "dimensions": list(static.size),
    }


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    if not FONT_PATH.is_file():
        raise SystemExit(f"required capture font is missing: {FONT_PATH}")
    if args.check:
        with tempfile.TemporaryDirectory(prefix="replai-showcase-") as directory:
            details = build(Path(directory))
            for name in ("terminal-showcase.png", "terminal-showcase.gif"):
                candidate = Path(directory) / name
                committed = OUT / name
                assert candidate.read_bytes() == committed.read_bytes(), f"{name} drifted"
    else:
        details = build(OUT)
    for name in ("terminal-showcase.png", "terminal-showcase.gif"):
        path = OUT / name
        print(f"{name}: {path.stat().st_size} bytes sha256={digest(path)}")
    print(f"PASS real PTY frames={details['frames']} duration_ms={details['duration_ms']} dimensions={details['dimensions']}")


if __name__ == "__main__":
    main()
