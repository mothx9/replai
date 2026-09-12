#!/usr/bin/env python3
"""Adversarial documentation fixtures; never edit the working documentation."""
from pathlib import Path
import re
import tempfile
import unittest

import check_docs as checks


class DocumentationGuard(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="replai-doc-check-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.paths = checks.inventory(checks.ROOT)
        for path in self.paths:
            target = self.root / path
            target.parent.mkdir(parents=True, exist_ok=True)
            if path.endswith((".md", ".json")) or path.startswith((
                "assets/readme/", "tools/readme/", "examples/showcase.rs",
                "examples/support/showcase_host.rs", "tools/docs/capture-requirements.txt",
            )):
                target.write_bytes((checks.ROOT / path).read_bytes())
            else:
                target.touch()

    def append(self, path, text):
        with (self.root / path).open("a") as stream:
            stream.write("\n" + text + "\n")
        self.paths.add(path)

    def reject(self, expected):
        errors, _ = checks.check_local(self.root, self.paths)
        self.assertTrue(any(expected in error for error in errors), errors)

    def test_current_documents_are_admitted(self):
        errors, diagrams = checks.check_local(self.root, self.paths)
        self.assertEqual(errors, [])
        self.assertTrue(diagrams)

    def test_public_asset_manifest_rejects_drift(self):
        target = self.root / "assets/readme/terminal-showcase.png"
        target.write_bytes(target.read_bytes() + b"drift")
        self.reject("asset digest/size drift")

    def test_public_asset_manifest_rejects_evidence_identity_drift(self):
        target = self.root / "assets/readme/manifest.json"
        target.write_text(target.read_text().replace(
            "6975c0979a1fd13f619f2d079b7494945ca18c5e", "0" * 40))
        self.reject("evidence identity drift")

    def test_public_status_markers_require_text(self):
        target = self.root / "README.md"
        target.write_text(target.read_text().replace("🟡 Replacement only", "🟡", 1))
        self.reject("color-only public status marker: 🟡")

    def test_missing_relative_link(self):
        self.append("README.md", "[missing](docs/missing.md)")
        self.reject("README.md: missing link target: docs/missing.md")

    def test_missing_anchor(self):
        self.append("README.md", "[missing](docs/interaction.md#absent)")
        self.reject("README.md: missing anchor: docs/interaction.md#absent")

    def test_orphan_and_unreferenced_asset(self):
        self.append("docs/orphan.md", "# Orphan")
        self.append("assets/orphan.svg", "<svg/>")
        self.reject("docs/orphan.md: unreachable")
        self.reject("assets/orphan.svg: unreachable")

    def test_retired_reference_and_reintroduced_surface(self):
        self.append("README.md", "[old](docs/baseline.md)")
        self.reject("README.md: missing link target: docs/baseline.md")
        self.append("docs/baseline.md", "# Old")
        self.reject("docs/baseline.md: retired/archive surface")

    def test_archive_and_duplicate_status_owner(self):
        (self.root / "docs/archive").mkdir()
        self.append("docs/archive/report.md", "# Project status")
        self.reject("docs/archive/report.md: retired/archive surface")
        self.reject("docs/archive/report.md: project-status owner")

    def test_abi_version_and_tags(self):
        target = self.root / "docs/c-api.md"
        target.write_text(target.read_text().replace("ABI 1", "ABI 9").replace(
            "| CAPACITY | 4 |", "| CAPACITY | 99 |"))
        self.reject("docs/c-api.md: ABI identity differs")
        self.reject("docs/c-api.md: REPLAI_CAPACITY must have one table value 4")

    def test_link_cannot_escape_repository(self):
        self.append("README.md", "[outside](../../outside.md)")
        self.reject("README.md: link escapes repository")

    def test_reference_html_fences_and_duplicate_heading_anchors(self):
        self.append("README.md", '''
## Repeat
## Repeat
[first](#repeat) [second](#repeat-1)
[guide][interaction]
[interaction]: docs/interaction.md
<a href="docs/presentation.md#external-output">Output</a>
```text
[not a link](missing-example.md)
```
`[also not a link](missing.md)`
''')
        errors, _ = checks.check_local(self.root, self.paths)
        self.assertEqual(errors, [])

    def test_unclosed_fence(self):
        self.append("README.md", "```mermaid\nflowchart LR\n A --> B")
        self.reject("README.md: line")
        errors, _ = checks.check_local(self.root, self.paths)
        self.assertTrue(any("unclosed code fence" in error for error in errors))

    def test_roadmap_control_rejects_inconsistent_promotions(self):
        target = self.root / "ROADMAP.md"
        original = target.read_text()
        body = original.split("<!-- maturity:start -->")[1].split("<!-- maturity:end -->")[0]
        rows = [line for line in body.splitlines() if re.match(r"^\| [a-z]+\.", line)]
        first, second = [cell.strip() for cell in rows[0].strip("|").split("|")], rows[1]

        def changed_cell(index, value):
            cells = first.copy()
            cells[index] = value
            return "| " + " | ".join(cells) + " |"

        selected = re.search(r"\*\*(?:[A-Z0-9.]+ — (?:SELECTED_NOT_STARTED|ACTIVE)|NONE)\*\*", original)[0]
        counts = re.search(r"ESTABLISHED=\d+", original)[0]
        mutations = [
            (second, rows[0], "duplicate maturity ID"),
            (rows[0], changed_cell(2, "🟢 COMPLETE"), "invalid maturity state"),
            (rows[0], changed_cell(5, "Z"), "invalid program reference"),
            (counts, "ESTABLISHED=9999", "maturity counts differ"),
            (selected, "**UNDECLARED**", "exactly one selected boundary"),
            (selected, selected + " **EXTRA — SELECTED_NOT_STARTED**", "exactly one selected boundary"),
            ("<!-- maturity:end -->", "<!-- maturity:start -->", "ordered maturity section"),
            (rows[0], changed_cell(4, ""), "seven nonempty fields"),
            (rows[0], changed_cell(6, "No evidence"), "missing evidence/owner link"),
            ("## Current Execution Sequence", "## Unselected work", "selected boundary missing"),
        ]
        for before, after, error in mutations:
            with self.subTest(error=error):
                target.write_text(original.replace(before, after, 1))
                self.reject(error)
        program = original.split("<!-- programs:start -->")[1].split("<!-- programs:end -->")[0]
        target.write_text(original.replace(program, program.replace("| F |", "| Z |", 1)))
        self.reject("invalid or duplicate program")
        target.write_text(original)

    def test_release_scope_covers_remaining_maturity_without_promoting_it(self):
        target = self.root / "ROADMAP.md"
        original = target.read_text()
        scope = original.split("<!-- release-scope:start -->")[1].split("<!-- release-scope:end -->")[0]
        row = next(line for line in scope.splitlines() if line.startswith("| history.storage_search |"))
        count = re.search(r"MUST_V0_1=\d+", original)[0]
        mutations = [
            (row, "", "release scope coverage differs"),
            (row, row + "\n" + row, "duplicate release capability"),
            (row, row.replace("MUST_V0_1", "OPTIONAL", 1), "invalid release class"),
            (row, row.replace("🟡 PARTIAL", "🟢 ESTABLISHED"), "release maturity differs"),
            (row, row.replace("history.storage_search", "unknown.capability"), "release scope coverage differs"),
            (row, row.rsplit("|", 2)[0] + "| Unlinked claim |", "missing release evidence link"),
            (count, "MUST_V0_1=9999", "release counts differ"),
            ("<!-- release-scope:end -->", "", "ordered release-scope section"),
        ]
        for before, after, expected in mutations:
            with self.subTest(expected=expected):
                target.write_text(original.replace(before, after, 1))
                self.reject(expected)
        target.write_text(original)

    def test_scope_boundaries_are_bounded_and_explicit(self):
        target = self.root / "ROADMAP.md"
        original = target.read_text()
        scope = original.split("<!-- scope-boundaries:start -->")[1].split(
            "<!-- scope-boundaries:end -->")[0]
        later = next(line for line in scope.splitlines() if "| LATER |" in line)
        outside = "\n".join(
            line for line in scope.splitlines() if "| OUT_OF_SCOPE |" in line)
        mutations = [
            (later, later + "\n" + later, "duplicate scope boundary"),
            (later, later.replace("LATER", "V0_2"), "invalid scope boundary class"),
            (outside, "", "scope boundaries must cover LATER and OUT_OF_SCOPE"),
            ("<!-- scope-boundaries:end -->", "", "ordered scope-boundaries section"),
        ]
        for before, after, expected in mutations:
            with self.subTest(expected=expected):
                target.write_text(original.replace(before, after, 1))
                self.reject(expected)
        target.write_text(original)

    def test_active_release_scope_authority_and_sequence_are_guarded(self):
        scope = self.root / "docs/release-scope.md"
        original_scope = scope.read_text()
        mutations = [
            ("active REPLAI v0.1 product contract", "candidate notes",
             "active v0.1 authority is ambiguous"),
            ("supersedes the earlier “minimum\nqualified kernel” definition",
             "revisits an earlier definition", "missing explicit scope supersession"),
            ("../ROADMAP.md#first-release-scope", "../ROADMAP.md",
             "missing canonical ROADMAP authority link"),
        ]
        for before, after, expected in mutations:
            with self.subTest(expected=expected):
                scope.write_text(original_scope.replace(before, after, 1))
                self.reject(expected)
        scope.write_text(original_scope + "\nV0_2\n")
        self.reject("stale V0_2 classification")
        scope.write_text(original_scope)

        roadmap = self.root / "ROADMAP.md"
        original_roadmap = roadmap.read_text()
        roadmap.write_text(original_roadmap.replace(
            "| 2 | COMPLETION.KEYMAP.SUGGESTION.0 |",
            "| 2 | WINDOWS.RUNTIME.0 |", 1))
        self.reject("expanded v0.1 dependency sequence differs")
        roadmap.write_text(original_roadmap)

    def test_table_delimiters_are_required_even_when_control_rows_are_valid(self):
        target = self.root / "ROADMAP.md"
        original = target.read_text()
        delimiter = "| --- | --- | --- | --- | --- | --- | --- |\n"
        for replacement in ("", "| --- | --- |\n"):
            with self.subTest(replacement=replacement):
                target.write_text(original.replace(delimiter, replacement, 1))
                self.reject("table header requires a matching Markdown delimiter row")
        target.write_text(original)
        self.append("README.md", "```text\n| example | only |\n```")
        errors, _ = checks.check_local(self.root, self.paths)
        self.assertEqual(errors, [])

    def test_actual_mermaid_parser_rejects_invalid_syntax(self):
        errors = checks.check_mermaid([
            {"file": "docs/bad.md", "line": 12, "code": "flowchart LR\n A --> ["},
        ])
        self.assertTrue(any("docs/bad.md:12: invalid Mermaid" in error for error in errors), errors)


if __name__ == "__main__":
    unittest.main(verbosity=2)
