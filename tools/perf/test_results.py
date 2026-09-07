#!/usr/bin/env python3
"""Reject invalid timing, missing provenance, mixed instrumentation and duplicates."""
import copy
import unittest
from unittest import mock
from types import SimpleNamespace
from results import distribution, summarize, validate

class Integrity(unittest.TestCase):
    def valid(self):
        return dict(schema_version=1,source={'head':'a'*40},environment={'rust':'recorded'},methodology={'outlier_policy':'retain all valid samples; no trimming'},
            results=[dict(schema_version=1,id='test',component='test',status='measured',mode='latency',iterations=3,timing=distribution([1,2,3]))])
    def test_environment_without_posix_load_average(self):
        from results import environment
        with mock.patch('results.os',SimpleNamespace(cpu_count=lambda:2,environ={})), mock.patch('results.command',return_value='unavailable'):
            self.assertIsNone(environment()['loadavg'])

    def test_statistics(self):
        self.assertEqual(distribution([9,1,3,2])['median'],2.5)
        self.assertEqual(distribution(list(range(1,101)))['p95'],95)
    def test_valid_and_rejections(self):
        validate(self.valid())
        for mutate in [lambda d:d['results'].append(copy.deepcopy(d['results'][0])),
                       lambda d:d['results'][0]['timing'].update(p99=0),
                       lambda d:d['results'][0]['timing'].update(median=float('nan')),
                       lambda d:d['source'].update(head='unknown'),
                       lambda d:d['results'][0].update(mode='allocation'),
                       lambda d:d['results'][0].update(iterations=0),
                       lambda d:d['results'][0].update(status='unavailable')]:
            d=self.valid();mutate(d)
            with self.assertRaises((AssertionError,KeyError)):validate(d)
    def test_allocation_summary_has_no_timing(self):
        row=summarize(dict(samples_us=None,allocation_samples=[dict(retained_delta_bytes=-8)]))
        self.assertNotIn('timing',row)
        self.assertEqual(row['allocations']['retained_delta_bytes']['median'],-8)

if __name__=='__main__':unittest.main()
