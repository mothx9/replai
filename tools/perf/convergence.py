#!/usr/bin/env python3
"""Derive same-run parity and before/after evidence; never a hosted timing gate."""
import argparse
import json
from pathlib import Path

from results import validate


def primary(data):
    rows = {r.get('library'): r for r in data['results']
            if r['component'] == 'comparison' and r.get('operation') == 'long_ascii'}
    for library in ('replai', 'rustyline', 'reedline'):
        r = rows[library]
        assert r['status'] == 'measured'
        assert r['counters']['submission_exact'] and r['counters']['terminal_restored']
        assert (r['input_bytes'], r['columns'], r['rows']) == (1004, 80, 24)
    fastest = min(('rustyline', 'reedline'), key=lambda name: rows[name]['timing']['median'])
    limit = 1.25 * rows[fastest]['timing']['median']
    return dict(fastest_reference=fastest, parity_limit_us=limit,
                replai_median_us=rows['replai']['timing']['median'],
                target_met=rows['replai']['timing']['median'] <= limit,
                rows=[rows[name] for name in ('replai', 'linenoise-blocking', 'linenoise-feed', 'rustyline', 'reedline')])


def report(before, after):
    validate(before)
    validate(after)
    b = {(r['id'], r.get('mode')): r for r in before['results']}
    pairs = []
    for row in after['results']:
        previous = b.get((row['id'], row.get('mode')))
        if previous and row['status'] == previous['status'] == 'measured':
            pairs.append(dict(id=row['id'], mode=row.get('mode'),
                              before={k: previous[k] for k in ('timing', 'allocations', 'counters', 'observations') if k in previous},
                              after={k: row[k] for k in ('timing', 'allocations', 'counters', 'observations') if k in row}))
    keys = ('cpu', 'architecture', 'host_hardware', 'macos', 'rust', 'power_source', 'power_policy')
    return dict(report_version=1, before_source=before['source']['head'], after_source=after['source']['head'],
                same_hardware=all(before['environment'].get(k) == after['environment'].get(k) for k in ('cpu', 'architecture', 'host_hardware')),
                environment_differences={k:dict(before=before['environment'].get(k),after=after['environment'].get(k)) for k in keys if before['environment'].get(k) != after['environment'].get(k)},
                primary=primary(after), paired_results=pairs,
                interpretation='Descriptive same-host runs; inspect environment differences and variance. No global score or CI timing threshold.')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('before', type=Path)
    parser.add_argument('after', type=Path)
    args = parser.parse_args()
    print(json.dumps(report(json.loads(args.before.read_text()), json.loads(args.after.read_text())), separators=(',', ':')))
