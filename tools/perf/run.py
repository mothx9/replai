#!/usr/bin/env python3
"""Prepare separately, then run a versioned P0 characterization without compilation."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
from results import ROOT, identity, environment, envelope, command

LINENOISE='a473823d74b93eab2ba83480df16ed37617493f2'
MANIFEST='tools/perf/Cargo.toml'


def run(command, output=None):
    print('+ '+' '.join(map(str,command)),file=sys.stderr,flush=True)
    if output:
        with output.open('w') as handle:subprocess.run(list(map(str,command)),cwd=ROOT,stdout=handle,check=True)
    else:subprocess.run(list(map(str,command)),cwd=ROOT,check=True)


def prepare(work, comparisons, qualification=None):
    run(['cargo','fetch','--locked','--manifest-path',MANIFEST])
    run(['cargo','build','--locked','--release','--manifest-path',MANIFEST,'--bins'])
    run(['cargo','build','--locked','--release','--manifest-path',MANIFEST,'--bin','components','--features','allocations','--target-dir','tools/perf/target/allocations'])
    if comparisons:
        run(['cargo','fetch','--locked','--manifest-path','tools/perf/comparisons/Cargo.toml'])
        run(['cargo','build','--locked','--release','--manifest-path','tools/perf/comparisons/Cargo.toml'])
        upstream=work/'linenoise'
        if not upstream.exists():run(['git','clone','https://github.com/antirez/linenoise.git',upstream])
        run(['git','-C',upstream,'checkout','--detach',LINENOISE])
        run(['cc','-O3','-g','-I'+str(upstream),'tools/perf/comparisons/linenoise.c',upstream/'linenoise.c','-o','tools/perf/comparisons/target/linenoise-host'])
    if qualification:run([sys.executable,'tools/perf/embedding.py','--qualification',qualification])
    binaries=[ROOT/'tools/perf/target/release/components',ROOT/'tools/perf/target/release/pty-host',ROOT/'tools/perf/target/allocations/release/components']
    import hashlib
    (work/'prepared.json').write_text(json.dumps(dict(source=identity(),binaries={str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in binaries}),indent=2)+'\n')


def measure(work, families, smoke, qualification):
    start=identity();receipt=dict(source=start,environment=environment())
    (work/'environment.json').write_text(json.dumps(receipt,indent=2)+'\n')
    # Verify that the prepared binaries still correspond to the measured sources.
    prepared=json.loads((work/'prepared.json').read_text())
    assert prepared['source']['source_sha256']==start['source_sha256'],'sources changed since preparation'
    import hashlib
    for path,digest in prepared['binaries'].items():assert hashlib.sha256((ROOT/path).read_bytes()).hexdigest()==digest,'binary changed since preparation'
    raw=[]
    if 'components' in families:
        for name,binary in [('components','tools/perf/target/release/components'),('allocations','tools/perf/target/allocations/release/components')]:
            path=work/(name+'.jsonl');raw.append(path)
            run([binary,*(['--smoke'] if smoke else [])],path)
    if 'linux' in families:
        path=work/'linux.jsonl';raw.append(path)
        run([sys.executable,'tools/perf/linux.py','--work',work/'traces',*(['--smoke'] if smoke else [])],path)
    if 'comparisons' in families:
        path=work/'comparisons.jsonl';raw.append(path)
        run([sys.executable,'tools/perf/compare.py','--work',work/'comparison-traces',*(['--smoke'] if smoke else [])],path)
    if 'sizes' in families:
        assert qualification,'--qualification required for sizes'
        path=work/'sizes.jsonl';raw.append(path)
        run([sys.executable,'tools/perf/sizes.py','--qualification',qualification,'--work',work/'stripped'],path)
    assert identity()['source_sha256']==start['source_sha256'],'sources changed during measurement'
    if sys.platform=='darwin':
        receipt['environment']['power_source_final']=command('pmset','-g','batt')
        (work/'environment.json').write_text(json.dumps(receipt,indent=2)+'\n')
    (work/'baseline.json').write_text(json.dumps(envelope(raw,receipt),separators=(',',':'))+'\n')


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--work',type=Path,required=True);p.add_argument('--prepare',action='store_true');p.add_argument('--families',default='components,linux,comparisons,sizes');p.add_argument('--smoke',action='store_true');p.add_argument('--qualification',type=Path);a=p.parse_args()
    a.work=a.work.resolve();a.work.mkdir(parents=True,exist_ok=True)
    families=a.families.split(',');assert set(families)<=set(['components','linux','comparisons','sizes'])
    if a.prepare:
        if 'sizes' in families:assert a.qualification,'--qualification required to prepare release C sizes'
        prepare(a.work,'comparisons' in families,a.qualification if 'sizes' in families else None)
    else:measure(a.work,families,a.smoke,a.qualification)
