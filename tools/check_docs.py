#!/usr/bin/env python3
"""Read-only documentation structure, ABI prose and Mermaid qualification."""
import argparse
from html import unescape
from html.parser import HTMLParser
import json
from pathlib import Path
import re
import struct
import subprocess
from urllib.parse import unquote, urlsplit

ROOT = Path(__file__).resolve().parents[1]
RETIRED = {"docs/baseline.md", "docs/archaeology.md"}
OWNERS = {
    "README.md", "CONTRIBUTING.md", "AGENTS.md", "ROADMAP.md", "CHANGELOG.md",
    "docs/README.md", "docs/repl.md", "docs/architecture.md",
    "docs/interaction.md", "docs/presentation.md", "docs/c-api.md",
    "docs/development.md", "docs/release-scope.md",
}


def inventory(root):
    """Include concurrent/untracked additions, omit ignored and deleted paths."""
    output = subprocess.check_output(
        ["git", "ls-files", "-z", "--cached", "--others", "--exclude-standard"],
        cwd=root,
    )
    return {p for p in output.decode().split("\0") if p and (root / p).is_file()}


class HtmlLinks(HTMLParser):
    def __init__(self):
        super().__init__()
        self.links = []
        self.ids = set()

    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        for key in ("href", "src"):
            if key in attrs and attrs[key] is not None:
                self.links.append(attrs[key])
        for key in ("id", "name"):
            if attrs.get(key):
                self.ids.add(attrs[key])


def parse_markdown(text):
    """Parse the repository's Markdown links/headings and fenced diagrams.

    Supports inline/reference links, HTML href/src/id, ATX/setext headings,
    duplicate heading slugs and backtick/tilde fences. Not a general GFM renderer.
    """
    visible, diagrams, errors = [], [], []
    fence, info, body, start = None, "", [], 0
    for number, line in enumerate(text.splitlines(), 1):
        marker = re.match(r"^ {0,3}(`{3,}|~{3,})(.*)$", line)
        if fence:
            if marker and marker[1][0] == fence[0] and len(marker[1]) >= len(fence) and not marker[2].strip():
                if info == "mermaid":
                    diagrams.append({"line": start, "code": "\n".join(body)})
                fence = None
            else:
                body.append(line)
            visible.append("")
        elif marker:
            fence, info, body, start = marker[1], marker[2].strip(), [], number
            visible.append("")
        else:
            visible.append(line)
    if fence:
        errors.append(f"line {start}: unclosed code fence")
    # A pipe header without its delimiter renders as prose, even when control
    # rows are otherwise valid. Check visible tables, excluding fenced examples.
    for i, line in enumerate(visible):
        if not line.startswith("|") or (i and visible[i - 1].startswith("|")):
            continue
        header = re.split(r"(?<!\\)\|", line.strip().strip("|"))
        following = visible[i + 1] if i + 1 < len(visible) else ""
        delimiter = following.strip().strip("|").split("|")
        if (not following.startswith("|") or len(delimiter) != len(header)
                or not all(re.fullmatch(r":?-{3,}:?", cell.strip()) for cell in delimiter)):
            errors.append(f"line {i + 1}: table header requires a matching Markdown delimiter row")
    prose = "\n".join(visible)
    html = HtmlLinks()
    html.feed(prose)
    ids, seen, headings = html.ids, set(), []
    for i, line in enumerate(visible):
        match = re.match(r"^ {0,3}#{1,6}\s+(.+?)\s*#*\s*$", line)
        title = match[1] if match else None
        if i and re.fullmatch(r" {0,3}(?:=+|-+)\s*", line) and visible[i - 1].strip():
            title = visible[i - 1].strip()
        if title:
            headings.append(title)
            plain = re.sub(r"<[^>]*>", "", title)
            plain = re.sub(r"\[([^]]+)\]\([^)]*\)", r"\1", plain)
            slug = re.sub(r"[^\w\- ]", "", unescape(plain).lower()).replace(" ", "-")
            candidate, n = slug, 0
            while candidate in seen:
                n += 1
                candidate = f"{slug}-{n}"
            seen.add(candidate)
            ids.add(candidate)
    # Code examples are not links; inline code is not a reference label.
    prose = re.sub(r"(`+).*?\1", "", prose)
    definitions = {}
    for match in re.finditer(r"^ {0,3}\[([^]]+)\]:\s*(\S+)", prose, re.M):
        definitions[match[1].casefold()] = match[2].strip("<>")
    prose = re.sub(r"^ {0,3}\[[^]]+\]:.*$", "", prose, flags=re.M)
    links = html.links + re.findall(r"\[[^]\n]*\]\(<?([^\s)>]+)>?(?:\s+[^)]*)?\)", prose)
    for match in re.finditer(r"\[([^]\n]+)\](?:\[([^]\n]*)\])?(?!\()", prose):
        label = (match[2] or match[1]).casefold()
        if label in definitions:
            links.append(definitions[label])
        elif match[2] is not None:
            errors.append(f"undefined reference [{label}]")
    return ids, links, diagrams, headings, errors


MATURITY = {"ESTABLISHED": "🟢", "PARTIAL": "🟡", "OPEN": "🔴", "LATER": "⚪"}


def asset_dimensions(path, format_name):
    data = path.read_bytes()
    if format_name == "PNG" and data.startswith(b"\x89PNG\r\n\x1a\n"):
        return list(struct.unpack(">II", data[16:24]))
    if format_name == "GIF" and data.startswith((b"GIF87a", b"GIF89a")):
        return list(struct.unpack("<HH", data[6:10]))
    if format_name == "SVG":
        text = data.decode()
        match = re.search(r'<svg[^>]*\bwidth="(\d+)"[^>]*\bheight="(\d+)"', text)
        if match:
            return [int(match[1]), int(match[2])]
    return None


def check_public_surface(root, paths):
    """Validate narrow README asset provenance without becoming status authority."""
    errors = []
    manifest_path = root / "assets/readme/manifest.json"
    if not manifest_path.is_file():
        return errors
    try:
        manifest = json.loads(manifest_path.read_text())
    except (ValueError, OSError) as error:
        return [f"assets/readme/manifest.json: invalid manifest: {error}"]
    if manifest.get("schema") != 1 or "not project or release status" not in manifest.get("scope", ""):
        errors.append("assets/readme/manifest.json: invalid narrow scope/schema")
    readme = (root / "README.md").read_text()
    import hashlib
    for record in manifest.get("assets", []):
        name = record.get("file", "")
        if name not in paths or name not in readme:
            errors.append(f"assets/readme/manifest.json: unreferenced or missing asset: {name}")
            continue
        path = root / name
        actual = hashlib.sha256(path.read_bytes()).hexdigest()
        if actual != record.get("sha256") or path.stat().st_size != record.get("bytes"):
            errors.append(f"assets/readme/manifest.json: asset digest/size drift: {name}")
        if asset_dimensions(path, record.get("format")) != record.get("dimensions"):
            errors.append(f"assets/readme/manifest.json: asset dimensions drift: {name}")
        if record.get("format") == "GIF" and path.stat().st_size > 5 * 1024 * 1024:
            errors.append(f"assets/readme/manifest.json: GIF exceeds 5 MiB: {name}")
        generator = record.get("generator", "")
        if generator not in paths:
            errors.append(f"assets/readme/manifest.json: missing generator: {generator}")
        for source in record.get("inputs", []):
            source_name = source.get("file", "")
            source_path = root / source_name
            if source_name not in paths or not source_path.is_file():
                errors.append(f"assets/readme/manifest.json: missing input: {source_name}")
            elif hashlib.sha256(source_path.read_bytes()).hexdigest() != source.get("sha256"):
                errors.append(f"assets/readme/manifest.json: input digest drift: {source_name}")
        if evidence := record.get("evidence"):
            evidence_path = root / evidence
            if not evidence_path.is_file():
                errors.append(f"assets/readme/manifest.json: missing evidence: {evidence}")
            else:
                identity = json.loads(evidence_path.read_text()).get("identity", {}).get("head")
                if identity != record.get("evidence_head"):
                    errors.append(f"assets/readme/manifest.json: evidence identity drift: {name}")
    required = [
        "docs/release-scope.md", "ROADMAP.md",
        "assets/replai-lockup-dark.svg", "assets/replai-lockup-light.svg",
    ]
    for value in required:
        if value not in readme:
            errors.append(f"README.md: missing public-surface invariant: {value}")
    for marker in ("🟢", "🟡", "⚪", "🔴"):
        matches = [line for line in readme.splitlines() if marker in line]
        if not matches:
            errors.append(f"README.md: missing public status marker: {marker}")
        for line in matches:
            if re.search(re.escape(marker) + r"\s+\*{0,2}[A-Za-z]", line) is None:
                errors.append(f"README.md: color-only public status marker: {marker}")
    return errors


def check_roadmap(text):
    """Validate the Markdown control tables, never infer maturity from prose."""
    errors = []

    def reject(message):
        errors.append("ROADMAP.md: " + message)

    def section(name):
        start, end = f"<!-- {name}:start -->", f"<!-- {name}:end -->"
        if text.count(start) != 1 or text.count(end) != 1 or text.index(start) >= text.index(end):
            reject(f"expected one ordered {name} section")
            return ""
        return text.split(start, 1)[1].split(end, 1)[0]

    def rows(body):
        return [
            [cell.strip() for cell in line.strip().strip("|").split("|")]
            for line in body.splitlines() if line.startswith("|")
            and not line.startswith(("| ID |", "| Program |", "| ---"))
        ]

    programs = set()
    for cells in rows(section("programs")):
        if len(cells) != 7 or not all(cells):
            reject("program rows require seven nonempty fields")
            continue
        program = cells[0]
        if not re.fullmatch(r"[FPIOUXQEVDA]", program) or program in programs:
            reject(f"invalid or duplicate program: {program}")
        programs.add(program)
        if cells[2] not in {f"{icon} {state}" for state, icon in MATURITY.items()}:
            reject(f"invalid program maturity: {program}")
    if programs != set("FPIOUXQEVDA"):
        reject("missing strategic program")

    counts, ids, maturity = dict.fromkeys(MATURITY, 0), set(), {}
    for cells in rows(section("maturity")):
        if len(cells) != 7 or not all(cells):
            reject("maturity rows require seven nonempty fields")
            continue
        identity, _, state, _, _, owners, evidence = cells
        if not re.fullmatch(r"[a-z][a-z0-9_]*(?:\.[a-z][a-z0-9_]*)+", identity) or identity in ids:
            reject(f"invalid or duplicate maturity ID: {identity}")
        ids.add(identity)
        maturity[identity] = state
        valid = [name for name, icon in MATURITY.items() if state == f"{icon} {name}"]
        if not valid:
            reject(f"invalid maturity state: {identity}")
        else:
            counts[valid[0]] += 1
        refs = owners.split(" / ")
        if len(refs) != len(set(refs)) or not set(refs) <= programs:
            reject(f"invalid program reference: {identity}")
        if not re.search(r"\[[^]]+\](?:\[[^]]+\]|\([^)]+\))", evidence):
            reject(f"missing evidence/owner link: {identity}")
    if not ids:
        reject("missing maturity rows")
    expected = " ".join(f"{state}={count}" for state, count in counts.items()) + f" TOTAL={sum(counts.values())}"
    if section("maturity-counts").strip() != expected:
        reject(f"maturity counts differ; expected {expected}")

    # Release inclusion is independent of maturity, but cannot omit an open gap.
    classes = dict.fromkeys(("MUST_V0_1", "SHOULD_V0_1", "LATER", "OUT_OF_SCOPE"), 0)
    remaining = {key for key, state in maturity.items() if state != "🟢 ESTABLISHED"}
    classified = set()
    for cells in rows(section("release-scope")):
        if cells[0] == "Capability":
            continue
        if len(cells) != 5 or not all(cells):
            reject("release scope requires five nonempty fields")
            continue
        identity, state, category, _, evidence = cells
        if identity in classified:
            reject(f"duplicate release capability: {identity}")
        classified.add(identity)
        if maturity.get(identity) != state:
            reject(f"release maturity differs from canonical row: {identity}")
        if category not in classes:
            reject(f"invalid release class: {identity}")
        else:
            classes[category] += 1
        if not re.search(r"\[[^]]+\]\([^)]+\)", evidence):
            reject(f"missing release evidence link: {identity}")
    if classified != remaining:
        reject(f"release scope coverage differs; missing={sorted(remaining - classified)}, extra={sorted(classified - remaining)}")
    expected = " ".join(f"{key}={value}" for key, value in classes.items()) + f" TOTAL={sum(classes.values())}"
    if section("release-counts").strip() != expected:
        reject(f"release counts differ; expected {expected}")

    # Deliberate horizon and ownership exclusions are explicit without becoming
    # adopted maturity rows or a second release-status authority.
    scope_topics, scope_classes = set(), set()
    for cells in rows(section("scope-boundaries")):
        if cells[0] == "Topic":
            continue
        if len(cells) != 3 or not all(cells):
            reject("scope boundary requires three nonempty fields")
            continue
        topic, category, _ = cells
        if topic in scope_topics:
            reject(f"duplicate scope boundary: {topic}")
        scope_topics.add(topic)
        if category not in {"LATER", "OUT_OF_SCOPE"}:
            reject(f"invalid scope boundary class: {topic}")
        else:
            scope_classes.add(category)
    if scope_classes != {"LATER", "OUT_OF_SCOPE"}:
        reject("scope boundaries must cover LATER and OUT_OF_SCOPE")

    selected = re.findall(
        r"^\| Current selected engineering boundary \| \*\*([A-Z0-9.]+)(?: — (SELECTED_NOT_STARTED|ACTIVE))?\*\*",
        text, re.M,
    )
    markers = re.findall(r"\*\*(?:[A-Z0-9.]+ — (?:SELECTED_NOT_STARTED|ACTIVE)|NONE)\*\*", text)
    valid = len(selected) == 1 and len(markers) == 1
    if valid:
        identity, state = selected[0]
        valid = (identity == "NONE" and not state) or (identity != "NONE" and bool(state))
    if not valid:
        reject("expected exactly one selected boundary or explicit NONE in the snapshot")
    elif (
        "## Current Execution Sequence" not in text
        or selected[0][0] not in text.split("## Current Execution Sequence", 1)[1].split("\n## ", 1)[0]
    ):
        reject("selected boundary missing from dependency sequence")

    sequence = (
        text.split("## Current Execution Sequence", 1)[1].split("\n## ", 1)[0]
        if "## Current Execution Sequence" in text else ""
    )
    boundaries = []
    for line in sequence.splitlines():
        if re.match(r"^\| \d+ \|", line):
            boundaries.append(line.strip().strip("|").split("|")[1].strip())
    expected_boundaries = [
        "INTERACTION.ERGONOMICS.0", "COMPLETION.KEYMAP.SUGGESTION.0",
        "OUTPUT.LONG_LIVED.0", "WINDOWS.RUNTIME.0", "SENSITIVE.INPUT.0",
        "PRESENTATION.UX.0", "LARGE.DRAFT.PERFORMANCE.0",
        "DEVELOPER.EXPERIENCE.0", "AGENTIC.INTEGRATION.0",
        "DOCUMENTATION.CLOSURE.0", "RELEASE.HARDENING.1",
        "RELEASE.CANDIDATE.0", "RELEASE.PUBLICATION.0",
    ]
    if boundaries != expected_boundaries:
        reject("expanded v0.1 dependency sequence differs")
    return errors


def check_release_scope(text):
    """Keep the active release authority explicit without inferring its truth."""
    errors = []
    if text.count("This is the active REPLAI v0.1 product contract.") != 1:
        errors.append("docs/release-scope.md: active v0.1 authority is ambiguous")
    if "supersedes the earlier “minimum\nqualified kernel” definition" not in text:
        errors.append("docs/release-scope.md: missing explicit scope supersession")
    if "../ROADMAP.md#first-release-scope" not in text:
        errors.append("docs/release-scope.md: missing canonical ROADMAP authority link")
    if re.search(r"\bV0_2\b", text):
        errors.append("docs/release-scope.md: stale V0_2 classification")
    return errors


def check_local(root, paths):
    """Return actionable errors and diagrams; no writes or network calls."""
    root = root.resolve()
    errors, parsed, diagrams = [], {}, []
    for missing in sorted(OWNERS - paths):
        errors.append(f"{missing}: missing documentation owner")
    for path in sorted(paths):
        if path in RETIRED or (path.endswith(".md") and any(
            part.lower() in {"archive", "archives"} for part in Path(path).parts
        )):
            errors.append(f"{path}: retired/archive surface must remain in Git history")
        if not path.endswith(".md"):
            continue
        parsed[path] = parse_markdown((root / path).read_text())
        _, _, blocks, headings, problems = parsed[path]
        diagrams.extend(dict(block, file=path) for block in blocks)
        errors.extend(f"{path}: {problem}" for problem in problems)
        # Enforce the explicit status surface, not guesses about semantic prose.
        if "Project status" in headings and path != "ROADMAP.md":
            errors.append(f"{path}: project-status owner must be ROADMAP.md")
    if "ROADMAP.md" in parsed and "Project status" not in parsed["ROADMAP.md"][3]:
        errors.append("ROADMAP.md: missing Project status heading")
    if "ROADMAP.md" in parsed:
        errors.extend(check_roadmap((root / "ROADMAP.md").read_text()))
    if "docs/release-scope.md" in parsed:
        errors.extend(check_release_scope((root / "docs/release-scope.md").read_text()))
    errors.extend(check_public_surface(root, paths))
    graph = {path: set() for path in parsed}
    for path, (_, links, _, _, _) in parsed.items():
        for link in links:
            url = urlsplit(link)
            if url.scheme or url.netloc:
                continue
            target = ((root / path).parent / unquote(url.path)).resolve() if url.path else root / path
            if not target.is_relative_to(root):
                errors.append(f"{path}: link escapes repository: {link}")
                continue
            if target.is_dir():
                target /= "README.md"
            relative = target.relative_to(root).as_posix()
            if relative not in paths:
                errors.append(f"{path}: missing link target: {link}")
                continue
            graph[path].add(relative)
            if url.fragment and relative in parsed and unquote(url.fragment) not in parsed[relative][0]:
                errors.append(f"{path}: missing anchor: {link}")
    reached, pending = set(), ["README.md"]
    while pending:
        path = pending.pop()
        if path not in reached:
            reached.add(path)
            pending.extend(graph.get(path, ()))
    for path in sorted(set(parsed) | {p for p in paths if p.startswith("assets/")}):
        if path not in reached:
            errors.append(f"{path}: unreachable from README.md")
    abi_path = root / "api/c-abi.json"
    if abi_path.is_file() and "docs/c-api.md" in parsed:
        schema = json.loads(abi_path.read_text())
        api = (root / "docs/c-api.md").read_text()
        for name, _, value in schema["constants"]:
            if name == "REPLAI_C_ABI_VERSION":
                versions = re.findall(r"\bABI (\d+)\b", api)
                if not versions or any(int(v) != value for v in versions):
                    errors.append(f"docs/c-api.md: ABI identity differs from schema ({value})")
            else:
                label = re.sub(r"^REPLAI_(?:EVENT_|ROLE_)?", "", name)
                values = re.findall(rf"^\| {re.escape(label)} \| (\d+) \|", api, re.M)
                if values != [str(value)]:
                    errors.append(f"docs/c-api.md: {name} must have one table value {value}; found {values}")
    else:
        errors.append("api/c-abi.json: missing ABI authority or C contract")
    return errors, diagrams


def check_mermaid(diagrams):
    """Run the real pinned Mermaid parser; missing tools fail the gate."""
    try:
        result = subprocess.run(
            ["node", str(ROOT / "tools/docs/check_mermaid.mjs")],
            input=json.dumps(diagrams), text=True, capture_output=True, timeout=60,
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        return [f"tools/docs: Mermaid parser unavailable: {error}"]
    if result.returncode:
        return [result.stderr.strip() or "tools/docs: Mermaid parse failed"]
    return []


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=ROOT)
    args = parser.parse_args()
    errors, diagrams = check_local(args.root, inventory(args.root))
    errors.extend(check_mermaid(diagrams))
    if errors:
        raise SystemExit("\n".join(errors))
    print(f"PASS documentation links, reachability, owners, roadmap control, ABI tables and {len(diagrams)} Mermaid diagrams")


if __name__ == "__main__":
    main()
