#!/usr/bin/env python3
"""Optional README capture: Python + Pillow 11.3.0 + pyte 0.8.2, DejaVu Sans Mono.
Run on Linux after building the structured/demo examples; no library dependency.
"""
import os, pty, subprocess, termios, fcntl, struct, select, time
from pathlib import Path
import pyte
from PIL import Image, ImageDraw, ImageFont

root = Path(__file__).resolve().parents[2]
master, slave = pty.openpty()
fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 48, 76, 0, 0))
env = {
    **os.environ,
    "TERM": "xterm-256color",
    "PS1": "$ ",
    "PS2": "> ",
    "PROMPT_COMMAND": "",
}
env.pop("NO_COLOR", None)
p = subprocess.Popen(
    ["bash", "--noprofile", "--norc", "-i"],
    stdin=slave,
    stdout=slave,
    stderr=slave,
    env=env,
    cwd=root,
    start_new_session=True,
)
screen = pyte.Screen(76, 48)
stream = pyte.ByteStream(screen)
raw = bytearray()


def drain(seconds):
    until = time.monotonic() + seconds
    while time.monotonic() < until:
        if select.select([master], [], [], max(0, until - time.monotonic()))[0]:
            data = os.read(master, 65536)
            raw.extend(data)
            stream.feed(data)


def send(data, seconds=0.3):
    os.write(master, data)
    drain(seconds)


drain(0.2)
# Clear shell startup diagnostics using the shell's own redraw command.
raw.clear()
send(b"\x0c", 0.2)
send(b"cargo run --quiet --locked --example structured -- 68\r", 1)
send(b"cargo run --quiet --locked --example demo -- --notice\r", 1)
send(b"hello terminal\r", 0.3)
send(b"wor\t", 2.2)
# The screenshot is rasterized from the real PTY screen, never authored text.
rows = list(screen.display)
last = max(i for i, r in enumerate(rows) if r.strip())
rows = rows[: last + 1]
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
im = Image.new("RGB", (76 * cw + 2 * pad, top + len(rows) * ch + pad), (24, 27, 33))
d = ImageDraw.Draw(im)
d.text((pad, 17 * scale), "REPLAI", font=bold, fill="#dce2eb")
d.text(
    (pad + 100 * scale, 22 * scale),
    "structured output + live editing",
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
    for x in range(76):
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
if y < len(rows):
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
im.save(root / "assets/terminal-preview.png", optimize=True)
print("\n".join(rows))
print(im.size)
send(b"\x03", 0.1)
send(b"\x04", 0.1)
send(b"exit\r", 0.1)
p.terminate()
p.wait(timeout=5)
os.close(master)
os.close(slave)
