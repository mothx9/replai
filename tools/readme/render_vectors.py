#!/usr/bin/env python3
"""Generate the README architecture and Q2 evidence graphics as deterministic SVG."""
import argparse
import hashlib
import html
import json
from pathlib import Path
import tempfile

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "assets/readme"
EVIDENCE = ROOT / "tools/hardening/evidence/q2-linux-aarch64.json"


def write(path, value):
    path.write_text(value.rstrip() + "\n")


def architecture(dark):
    bg = "#181b21" if dark else "#f6f7fa"
    panel = "#22262e" if dark else "#ffffff"
    text = "#e9edf4" if dark else "#20242c"
    muted = "#a8b0bd" if dark else "#5d6572"
    line = "#535b69" if dark else "#c7ccd5"
    purple = "#9b76f7" if dark else "#7046d8"
    boxes = [
        (70, 84, 1060, 116, "YOUR APPLICATION", "parser · semantics · execution · state · scheduler"),
        (70, 250, 1060, 118, "PUBLIC CONTRACT", "blocking  ·  session  ·  driven  ·  C ABI 1"),
        (70, 418, 1060, 142, "ONE INTERACTION ENGINE", "editor + revision provenance  ·  analysis contracts", "presentation + renderer  ·  safe coordinated output"),
        (70, 610, 1060, 116, "TERMINAL CONTRACT", "capabilities · decoder · readiness · resources · restoration"),
        (250, 776, 700, 92, "NATIVE RUNTIME", "Linux GNU  ·  macOS"),
    ]
    parts = [f'<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="920" viewBox="0 0 1200 920" role="img" aria-labelledby="title desc">',
             '<title id="title">REPLAI architecture</title>',
             '<desc id="desc">The application calls blocking, session, driven, or C ABI contracts over one interaction engine, terminal contract, and Linux or macOS runtime.</desc>',
             f'<rect width="1200" height="920" fill="{bg}" rx="20"/>']
    for index, box in enumerate(boxes):
        x, y, width, height, heading, *body = box
        parts.append(f'<rect x="{x}" y="{y}" width="{width}" height="{height}" rx="14" fill="{panel}" stroke="{purple if index == 2 else line}" stroke-width="{3 if index == 2 else 2}"/>')
        parts.append(f'<text x="600" y="{y + 39}" text-anchor="middle" font-family="ui-monospace,SFMono-Regular,Consolas,monospace" font-size="20" font-weight="700" fill="{purple}">{heading}</text>')
        for n, content in enumerate(body):
            parts.append(f'<text x="600" y="{y + 75 + n * 28}" text-anchor="middle" font-family="ui-monospace,SFMono-Regular,Consolas,monospace" font-size="17" fill="{text if index == 2 else muted}">{content}</text>')
        if index < len(boxes) - 1:
            next_y = boxes[index + 1][1]
            parts.append(f'<path d="M600 {y + height} V{next_y - 20}" stroke="{purple}" stroke-width="3"/>')
            parts.append(f'<path d="M592 {next_y - 30} L600 {next_y - 20} L608 {next_y - 30}" fill="none" stroke="{purple}" stroke-width="3"/>')
    parts.append('</svg>')
    return "\n".join(parts)


def human_ns(value):
    if value < 1000:
        return f"{value} ns"
    if value < 1_000_000:
        return f"{value / 1000:.1f} µs"
    return f"{value / 1_000_000:.2f} ms"


def benchmark():
    evidence = json.loads(EVIDENCE.read_text())
    selected = [
        ("append/1024/0", "Warmed append"),
        ("burst/0/0", "1,000-byte edit + submit"),
        ("menu-show/1024/0", "Completion menu show"),
        ("validation/1024/0", "Validation result"),
        ("analysis/65536/0", "Analysis · 64 KiB draft"),
        ("analysis/1048576/0", "Analysis · 1 MiB draft"),
    ]
    values = [evidence["gates"][key]["median_ns"] for key, _ in selected]
    # Log scale makes nanosecond and millisecond workloads legible together.
    import math
    minimum, maximum = math.log10(min(values)), math.log10(max(values))
    rows = []
    for index, ((key, label), value) in enumerate(zip(selected, values)):
        y = 145 + index * 73
        width = 80 + 610 * (math.log10(value) - minimum) / (maximum - minimum)
        rows.extend([
            f'<text x="55" y="{y + 5}" font-family="ui-monospace,SFMono-Regular,Consolas,monospace" font-size="18" fill="#dce2eb">{html.escape(label)}</text>',
            f'<rect x="390" y="{y - 19}" width="{width:.1f}" height="28" rx="5" fill="#8d5cf5"/>',
            f'<text x="{405 + width:.1f}" y="{y + 3}" font-family="ui-monospace,SFMono-Regular,Consolas,monospace" font-size="17" fill="#dce2eb">{human_ns(value)}</text>',
        ])
    identity = evidence["identity"]
    return "\n".join([
        '<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="680" viewBox="0 0 1200 680" role="img" aria-labelledby="title desc">',
        '<title id="title">Selected REPLAI Q2 median latencies</title>',
        '<desc id="desc">Six selected median latency workloads from the qualified Q2 Linux ARM64 evidence, displayed on a logarithmic scale.</desc>',
        '<rect width="1200" height="680" fill="#181b21" rx="20"/>',
        '<text x="55" y="55" font-family="ui-monospace,SFMono-Regular,Consolas,monospace" font-size="25" font-weight="700" fill="#b69af8">Q2 / RECORDED MEDIAN LATENCY</text>',
        '<text x="55" y="89" font-family="ui-monospace,SFMono-Regular,Consolas,monospace" font-size="16" fill="#a8b0bd">logarithmic scale · preparation and verification excluded</text>',
        *rows,
        '<line x1="55" y1="597" x2="1145" y2="597" stroke="#393f4b"/>',
        f'<text x="55" y="625" font-family="ui-monospace,SFMono-Regular,Consolas,monospace" font-size="14" fill="#a8b0bd">source {identity["head"][:12]} · {identity["host"]["node"]} · {identity["host"]["machine"]} · Rust 1.98.1 · CPU 19</text>',
        '<text x="55" y="650" font-family="ui-monospace,SFMono-Regular,Consolas,monospace" font-size="14" fill="#a8b0bd">Recorded workload, not a universal library ranking. Full methodology and 32 gates are linked below.</text>',
        '</svg>',
    ])


def build(destination):
    destination.mkdir(parents=True, exist_ok=True)
    write(destination / "architecture-dark.svg", architecture(True))
    write(destination / "architecture-light.svg", architecture(False))
    write(destination / "benchmark-q2.svg", benchmark())


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    if args.check:
        with tempfile.TemporaryDirectory(prefix="replai-vectors-") as directory:
            candidate = Path(directory)
            build(candidate)
            for name in ("architecture-dark.svg", "architecture-light.svg", "benchmark-q2.svg"):
                assert (candidate / name).read_bytes() == (OUT / name).read_bytes(), f"{name} drifted"
    else:
        build(OUT)
    for name in ("architecture-dark.svg", "architecture-light.svg", "benchmark-q2.svg"):
        path = OUT / name
        print(f"{name}: {path.stat().st_size} bytes sha256={hashlib.sha256(path.read_bytes()).hexdigest()}")


if __name__ == "__main__":
    main()
