#!/usr/bin/env python3
"""Fast negative tests for the release evidence harness itself."""
import tempfile
import gzip
import hashlib
import json
import corpus
from pathlib import Path
import unittest
import campaign
import regression


class HardeningGuard(unittest.TestCase):
    def test_five_distinct_targets(self):
        self.assertEqual(set(campaign.TARGETS), {"protocol", "editor", "results", "geometry", "cabi"})

    def test_seed_identity_is_reproducible(self):
        with tempfile.TemporaryDirectory() as root:
            a, b = Path(root)/"a", Path(root)/"b"
            a.mkdir(); b.mkdir()
            campaign.seeds(a); campaign.seeds(b)
            self.assertEqual(campaign.digest(a), campaign.digest(b))
            (b/"extra").write_bytes(b"another input")
            self.assertNotEqual(campaign.digest(a), campaign.digest(b))

    def test_registered_formula_uses_distinct_median_and_p95_mad(self):
        samples = [n for n in (90, 100, 110, 120, 130) for _ in range(31)]
        t = regression.thresholds(samples, 1)
        self.assertEqual(t["median_ns"], 110)
        self.assertEqual(t["median_allowance_ns"], 50)
        self.assertEqual(t["p95_allowance_ns"], 32.5)

    def test_corpus_receipts_and_tampering(self):
        record = {"environment": {}, "inputs": [""], "summary": {
            "passed": True, "budget_complete": True, "cpu_seconds": 3600,
            "final_corpus": {"files": 1, "sha256": hashlib.sha256(hashlib.sha256(b"").hexdigest().encode()).hexdigest()}}}
        archive = {"version": 1, "targets": {}}
        for name in campaign.TARGETS:
            entry = json.loads(json.dumps(record))
            entry["summary"]["target"] = name
            archive["targets"][name] = entry
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)/"corpus.json.gz"
            path.write_bytes(gzip.compress(json.dumps(archive).encode()))
            self.assertEqual(corpus.read_archive(path), archive)
            archive["targets"]["editor"]["inputs"] = ["eA=="]
            path.write_bytes(gzip.compress(json.dumps(archive).encode()))
            with self.assertRaises(AssertionError):
                corpus.read_archive(path)

    def test_timer_floor(self):
        t = regression.thresholds([1]*155, 10)
        self.assertEqual(t["median_allowance_ns"], 20)
        self.assertEqual(t["p95_allowance_ns"], 20)

    def test_missing_batches_and_nonfinite_samples_refuse(self):
        for values in ([1]*31, [float("nan")]*155, [float("inf")]*155, [-1]*155):
            with self.assertRaises(AssertionError):
                regression.thresholds(values, 1)


if __name__ == "__main__":
    unittest.main()
