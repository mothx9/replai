#!/usr/bin/env python3
"""Artifact bytes and stripped copies; no size optimizations or production edits."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
from results import ROOT, command

p=argparse.ArgumentParser(description=__doc__);p.add_argument('--qualification',type=Path,required=True);p.add_argument('--work',type=Path,required=True);a=p.parse_args()
a.work.mkdir(parents=True,exist_ok=True)
artifacts={
    'rust_example': ROOT/'target/release/examples/demo',
    'c_static_consumer':a.qualification/'consumer/demo-static-release',
    'c_shared_consumer':a.qualification/'consumer/demo-shared-release',
    'dynamic_library':a.qualification/'prefix/lib/libreplai_c.so',
    'static_archive':a.qualification/'prefix/lib/libreplai_c.a',
}
for name,path in artifacts.items():
    if not path.exists():
        print(json.dumps(dict(schema_version=1,id='size/'+name,component='embedding',status='unavailable',reason='artifact missing: '+str(path))));continue
    output=a.work/(name+path.suffix);shutil.copy2(path,output)
    stripped=None
    if shutil.which('strip'):
        subprocess.run(['strip','--strip-debug' if name=='static_archive' else '--strip-all',output],check=True)
        stripped=output.stat().st_size
    print(json.dumps(dict(schema_version=1,id='size/'+name,component='embedding',operation=name,status='measured',mode='size',iterations=1,
        counters=dict(unstripped_bytes=path.stat().st_size,stripped_bytes=stripped),sha256=hashlib.sha256(path.read_bytes()).hexdigest(),
        sections=command('size',str(path)),scope='absolute release artifact; C host cc -O2 with isolated C01-C15 qualification, library release; static archive not final consumer size')))
