#!/usr/bin/env python3
"""Exact internal allocation/encoded-byte gates; no hosted latency claims."""
import json
from pathlib import Path
import platform
import subprocess

ROOT = Path(__file__).resolve().parents[2]
gates = json.loads((ROOT/'tools/hardening/evidence/q2-linux-aarch64.json').read_text())['gates']
subprocess.run(['cargo','build','--locked','--release','--manifest-path','tools/hardening/Cargo.toml',
                '--bin','bench','--features','allocations','--target-dir','tools/hardening/target/allocations'],cwd=ROOT,check=True)
binary = ROOT/'tools/hardening/target/allocations/release'/('bench.exe' if platform.system()=='Windows' else 'bench')
output = subprocess.check_output([str(binary),'3','1'],text=True)
rows = [json.loads(line) for line in output.splitlines()]
assert rows.pop(0)['instrumented_allocations']
assert len(rows)==len(gates) and {r['id'] for r in rows}==set(gates)
for row in rows:
    gate=gates[row['id']]
    assert all(v==gate['allocations'] for v in row['allocations']), (row['id'],'allocation regression',row['allocations'],gate['allocations'])
    assert all(v==gate['encoded_bytes'] for v in row['encoded_bytes']), (row['id'],'encoded-byte regression')
print(json.dumps({'deterministic_workloads':len(rows),'repetitions':3,'allocations':'PASS','encoded_bytes':'PASS','latency':'NOT_EVALUATED'}))
