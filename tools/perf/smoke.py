#!/usr/bin/env python3
"""Portable CI integrity only: no thresholds and no hosted performance claims."""
import json
from pathlib import Path
import subprocess
import tempfile
from results import ROOT, environment, identity, envelope, validate

def run(*args):subprocess.run(args,cwd=ROOT,check=True)

with tempfile.TemporaryDirectory(prefix='replai-performance-smoke-') as temp:
    work=Path(temp)
    run('cargo','fmt','--manifest-path','tools/perf/Cargo.toml','--check')
    run('cargo','build','--locked','--release','--manifest-path','tools/perf/Cargo.toml','--bins')
    suffix='.exe' if __import__('os').name=='nt' else ''
    binary=ROOT/'tools/perf/target/release'/('components'+suffix)
    with (work/'latency.jsonl').open('w') as out:subprocess.run([binary,'--smoke'],stdout=out,check=True)
    run('cargo','build','--locked','--release','--manifest-path','tools/perf/Cargo.toml','--bin','components','--features','allocations','--target-dir',str(work/'alloc-build'))
    with (work/'allocations.jsonl').open('w') as out:subprocess.run([work/'alloc-build/release'/('components'+suffix),'--smoke'],stdout=out,check=True)
    data=envelope([work/'latency.jsonl',work/'allocations.jsonl'],dict(source=identity(),environment=environment()))
    print(f'Portable smoke: {validate(data)} valid results (not a performance baseline)')
    run('python3' if __import__('os').name!='nt' else 'python','tools/perf/test_results.py')
