#!/usr/bin/env python3
"""Rasterize real query-example PTY screens using Pillow 11.3.0 and pyte 0.8.2.
Optional documentation tooling; no generated application text or library dependency.
"""
import os, pty, subprocess, termios, fcntl, struct, select, time
from pathlib import Path
import pyte
from PIL import Image, ImageDraw, ImageFont

root = Path(__file__).resolve().parents[2]
master, slave = pty.openpty()
fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 32, 72, 0, 0))
before = termios.tcgetattr(slave)
env = {**os.environ, "TERM": "xterm-256color"}
env.pop("NO_COLOR", None)
p = subprocess.Popen([str(root / "target/debug/examples/query"), "--notice"],
    stdin=slave, stdout=slave, stderr=slave, env=env, cwd=root)
screen = pyte.Screen(72, 32)
stream = pyte.ByteStream(screen)
raw = bytearray()


def drain(seconds):
    until = time.monotonic() + seconds
    while time.monotonic() < until:
        if select.select([master], [], [], max(0, until - time.monotonic()))[0]:
            data = os.read(master, 65536)
            raw.extend(data)
            stream.feed(data)


def send(data, seconds=0.15):
    assert os.write(master, data) == len(data)
    drain(seconds)


def capture(filename, title, show_cursor=True):
    rows = list(screen.display)
    last = max(i for i, row in enumerate(rows) if row.strip())
    rows = rows[:last + 1]
    fontpath = "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf"
    boldpath = fontpath.replace(".ttf", "-Bold.ttf")
    # Render glyphs directly at 2x density; never enlarge an existing bitmap.
    scale = 2
    font = ImageFont.truetype(fontpath, 20 * scale)
    bold = ImageFont.truetype(boldpath, 20 * scale)
    caption = ImageFont.truetype(fontpath, 14 * scale)
    cw = 12 * scale
    ch = 28 * scale
    pad = 24 * scale
    top = 72 * scale
    im = Image.new("RGB", (72 * cw + 2 * pad, top + len(rows) * ch + pad), (24, 27, 33))
    d = ImageDraw.Draw(im)
    d.text((pad, 17 * scale), "REPLAI", font=bold, fill="#dce2eb")
    d.text(
        (pad + 100 * scale, 22 * scale),
        title,
        font=caption,
        fill="#a5aebc",
    )
    d.line((pad, 53 * scale, im.width - pad, 53 * scale), fill="#363d49", width=scale)
    colors = {
        "default": "#dce2eb",
        "black": "#17191e",
        "red": "#e58b8b",
        "green": "#99bf95",
        "brown": "#dfbd80",
        "blue": "#90b2df",
        "magenta": "#c5a2d7",
        "cyan": "#92c7ce",
        "white": "#e4e8ef",
        "brightblack": "#8893a4",
    }
    for y, row in enumerate(rows):
        for x in range(72):
            cell = screen.buffer[y][x]
            if cell.data.strip():
                color = colors.get(
                    cell.fg, "#" + cell.fg if len(cell.fg) == 6 else "#dce2eb"
                )
                d.text(
                    (pad + x * cw, top + y * ch),
                    cell.data,
                    font=bold if cell.bold else font,
                    fill=color,
                )
    x, y = screen.cursor.x, screen.cursor.y
    if show_cursor and y < len(rows):
        d.rectangle(
            (
                pad + x * cw,
                top + y * ch + 3 * scale,
                pad + (x + 1) * cw - 2 * scale,
                top + (y + 1) * ch - 2 * scale,
            ),
            outline="#dce2eb",
            width=scale,
        )
    im.save(root / "assets" / filename, optimize=True)
    print("\n".join(rows))
    print(im.size)

try:
    drain(0.2)
    send(b"SEL\t")
    assert "SELECT * FROM deployments;" in "\n".join(screen.display)
    send(b"\r")
    assert "worker" in "\n".join(screen.display)
    capture("terminal-results.png", "structured results")
    send(b"\x1b[200~SELECT *\nFROM deployments;\x1b[201~")
    # Recall history, then return to the exact unfinished multiline draft.
    send(b"\x1b[A\x1b[B")
    send(b"\x1b[D" * 12 + b"\x0c")
    cursor = (screen.cursor.x, screen.cursor.y)
    drain(2.1)
    visible = "\n".join(screen.display)
    assert "Fixture notice:" in visible and "FROM deployments;" in visible
    # External output adds one row; the same input column must survive.
    assert screen.cursor.x == cursor[0]
    capture("terminal-editing.png", "multiline editing + host output")
    send(b"\x03")
    send(b"\x04")
    assert p.wait(timeout=5) == 0
    assert termios.tcgetattr(slave) == before
    assert raw.rfind(b"\x1b[?2004l") > raw.rfind(b"\x1b[?2004h")
finally:
    if p.poll() is None:
        p.terminate()
        p.wait(timeout=5)
    os.close(master)
    os.close(slave)
