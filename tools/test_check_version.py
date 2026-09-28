#!/usr/bin/env python3
"""Discriminating product version projection checks."""
from pathlib import Path
import shutil
import tempfile
import unittest

import check_version


class VersionTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        for path in ("Cargo.toml", "Cargo.lock", "crates/replai-c/Cargo.toml",
                     "tools/distribution_policy.json"):
            target = self.root / path
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(check_version.ROOT / path, target)

    def replace(self, path, old, new):
        target = self.root / path
        target.write_text(target.read_text().replace(old, new))

    def test_live_projections(self):
        self.assertEqual(check_version.check(self.root), [])

    def test_binding_drift(self):
        self.replace("crates/replai-c/Cargo.toml", 'version = "0.1.0"', 'version = "0.2.0"')
        self.assertTrue(check_version.check(self.root))

    def test_dependency_drift(self):
        self.replace("crates/replai-c/Cargo.toml", '"=0.1.0"', '"^0.1.0"')
        self.assertTrue(check_version.check(self.root))

    def test_lock_drift(self):
        self.replace("Cargo.lock", 'version = "0.1.0"', 'version = "0.2.0"')
        self.assertTrue(check_version.check(self.root))

    def test_sdk_policy_drift(self):
        self.replace("tools/distribution_policy.json", "replai-c-sdk-0.1.0", "replai-c-sdk-0.2.0")
        self.assertTrue(check_version.check(self.root))

    def test_semver_grammar(self):
        for value in ("0.1.0", "1.2.3-rc.1", "1.2.3+build.7"):
            self.assertIsNotNone(check_version.SEMVER.fullmatch(value))
        for value in ("01.1.0", "1.2", "1.2.3-01", "latest", "1.2.3-"):
            self.assertIsNone(check_version.SEMVER.fullmatch(value))


if __name__ == "__main__":
    unittest.main()
