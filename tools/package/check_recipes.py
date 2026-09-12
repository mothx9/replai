#!/usr/bin/env python3
"""Check that the canonical cookbook retains each selected v0.1 recipe."""
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
text = (ROOT / "docs/use-cases.md").read_text()
required = {
    "simple-loop", "history-admission", "history-persistence", "rich-completion",
    "validated-multiline", "delayed-analysis", "spans-hints", "reactor",
    "finite-notices", "coalesced-output", "closed-chat-stream", "structured-report",
    "c-pkg-config", "cmake-static", "cmake-shared", "cpp", "no-color",
    "secret-input", "concurrent-writers",
}
observed = {line.split("recipe:", 1)[1].split(" -->", 1)[0] for line in text.splitlines() if "<!-- recipe:" in line}
missing = sorted(required - observed)
extra = sorted(observed - required)
if missing or extra:
    raise SystemExit(f"cookbook recipe identity mismatch: missing={missing}, extra={extra}")
for path in [
    "examples/simple.rs", "examples/completion.rs", "examples/validation.rs",
    "examples/analysis-presentation.rs", "examples/driven.rs", "examples/query.rs",
    "examples/report.rs", "tools/qualify_c.py", "docs/c-sdk.md",
]:
    if not (ROOT / path).is_file():
        raise SystemExit(f"recipe carrier missing: {path}")
print("PASS cookbook: 19 selected supported/deferred recipe contracts")
