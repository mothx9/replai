#!/usr/bin/env python3
"""Capture real query-console and standalone report output: Pillow 11.3.0, pyte 0.8.2, DejaVu Sans Mono.
Optional documentation tooling; all terminal contents come from the executable.
"""
import copy
import fcntl
import os
from pathlib import Path
import pty
import select
import struct
import subprocess
import termios
import time

import pyte
from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parents[2]


class Session:
    def __init__(self, columns, *, plain=False, notice=False):
        self.master, self.slave = pty.openpty()
        fcntl.ioctl(self.slave, termios.TIOCSWINSZ, struct.pack("HHHH", 48, columns, 0, 0))
        self.before = termios.tcgetattr(self.slave)
        env = {**os.environ, "TERM": "xterm-256color"}
        env.pop("NO_COLOR", None)
        if plain:
            env["NO_COLOR"] = "1"
        self.process = subprocess.Popen(
            [str(ROOT / "target/debug/examples/query"), *(["--notice"] if notice else [])],
            stdin=self.slave, stdout=self.slave, stderr=self.slave, env=env, cwd=ROOT,
        )
        self.screen = pyte.Screen(columns, 48)
        self.stream = pyte.ByteStream(self.screen)
        self.raw = bytearray()

    def drain(self, seconds=0.2):
        until = time.monotonic() + seconds
        while time.monotonic() < until:
            if select.select([self.master], [], [], max(0, until - time.monotonic()))[0]:
                data = os.read(self.master, 65536)
                self.raw.extend(data)
                self.stream.feed(data)

    def send(self, data):
        assert os.write(self.master, data) == len(data)
        self.drain()

    def text(self):
        return "\n".join(row.rstrip() for row in self.screen.display).rstrip()

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


def screen_capture(columns, *, plain=False, editing=False):
    session = Session(columns, plain=plain, notice=editing)
    try:
        session.drain()
        if not editing:
            session.send(b"\x0c")  # Actual editor redraw clears introductory output.
        session.send(b"SEL\t")
        assert "SELECT*FROMdeployments;" in "".join(session.text().split())
        session.send(b"\r")
        assert "scheduler" in session.text() and "degraded" in session.text()
        if editing:
            session.send(b"\x1b[200~SELECT *\nFROM deployments;\x1b[201~")
            # History return and a non-end cursor, preserved through host output.
            session.send(b"\x1b[A\x1b[B")
            session.send(b"\x1b[D" * 12)
            cursor = session.screen.cursor.x
            session.drain(2.1)
            assert "Fixture notice:" in session.text()
            assert "... FROM deployments;" in session.text()
            assert session.screen.cursor.x == cursor
        if plain:
            assert all(cell.fg == "default" and not cell.bold
                       for row in session.screen.buffer.values() for cell in row.values())
        print(f"{columns} columns / NO_COLOR={plain}\n{session.text()}\n")
        return copy.deepcopy(session.screen)
    finally:
        session.close()


def report_capture(columns=100):
    master, slave = pty.openpty()
    try:
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 48, columns, 0, 0))
        before = termios.tcgetattr(slave)
        env = {**os.environ, "TERM": "xterm-256color"}
        env.pop("NO_COLOR", None)
        process = subprocess.Popen([str(ROOT / "target/debug/examples/report")],
                                   stdout=slave, stderr=slave, env=env, cwd=ROOT)
        screen = pyte.Screen(columns, 48)
        stream = pyte.ByteStream(screen)
        until = time.monotonic() + 5
        while time.monotonic() < until:
            if select.select([master], [], [], 0.1)[0]:
                stream.feed(os.read(master, 65536))
            elif process.poll() is not None:
                break
        assert process.wait(timeout=5) == 0
        assert termios.tcgetattr(slave) == before
        output = "\n".join(screen.display)
        assert all(value in output for value in ["Build console", "Pipeline summary", "Signing key", "Command help", "inspect"])
        screen.cursor.hidden = True  # Standalone output has no editable input surface.
        print(output.rstrip())
        return screen
    finally:
        os.close(master)
        os.close(slave)


def render(filename, panels):
    # Direct glyph rasterization at 2x. At README width this is approximately 14px.
    fontpath = "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf"
    font = ImageFont.truetype(fontpath, 28)
    bold = ImageFont.truetype(fontpath.replace(".ttf", "-Bold.ttf"), 28)
    label = ImageFont.truetype(fontpath, 24)
    cw, ch, pad, gap, top = 17, 40, 48, 40, 112
    heights = [max(i for i, row in enumerate(screen.display) if row.strip()) + 1
               for _, screen in panels]
    width = sum(screen.columns * cw for _, screen in panels) + 2 * pad + gap * (len(panels) - 1)
    canvas = Image.new("RGB", (width, top + max(heights) * ch + pad), "#181b21")
    draw = ImageDraw.Draw(canvas)
    colors = {"default": "#dce2eb", "black": "#17191e", "red": "#e58b8b",
              "green": "#99bf95", "brown": "#dfbd80", "blue": "#90b2df",
              "magenta": "#c5a2d7", "cyan": "#92c7ce", "white": "#e4e8ef",
              "brightblack": "#8893a4"}
    left = pad
    for (title, screen), height in zip(panels, heights):
        draw.text((left, 30), title, font=label, fill="#a5aebc")
        draw.line((left, 80, left + screen.columns * cw, 80), fill="#363d49", width=2)
        for y in range(height):
            for x in range(screen.columns):
                cell = screen.buffer[y][x]
                if cell.data.strip():
                    color = colors.get(cell.fg, "#" + cell.fg if len(cell.fg) == 6 else "#dce2eb")
                    draw.text((left + x * cw, top + y * ch), cell.data,
                              font=bold if cell.bold else font, fill=color)
        x, y = screen.cursor.x, screen.cursor.y
        if y < height and not screen.cursor.hidden:
            draw.rectangle((left + x * cw, top + y * ch + 2,
                            left + (x + 1) * cw - 1, top + (y + 1) * ch - 2),
                           outline="#dce2eb", width=2)
        left += screen.columns * cw + gap
    canvas.save(ROOT / "assets" / filename, optimize=True)
    print(filename, canvas.size)


if __name__ == "__main__":
    render("terminal-results.png", [("REPLAI / completion, results, history and multiline editing",
                                     screen_capture(100, editing=True))])
    render("terminal-report.png", [("REPLAI / standalone build report, status and command help",
                                    report_capture())])
