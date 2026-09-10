#!/usr/bin/env python3
"""Run one unchanged public-API measurement fixture against two exact Git archives.

Temporary source/build fixtures are not Git worktrees. Production files are not
patched. Only this extra benchmark binary is added to each archived perf package.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import statistics
import subprocess
import tarfile
import tempfile
ROOT=Path(__file__).resolve().parents[1]

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--before',required=True);p.add_argument('--after',required=True);p.add_argument('--out',type=Path,required=True)
    a=p.parse_args();fixture=(ROOT/'tools/perf/direct_completion.rs').read_bytes()
    result={'fixture_sha256':hashlib.sha256(fixture).hexdigest(),'machine':os.uname().nodename,'scope':'1 KiB ASCII draft, revision-bound replacement with build; open/drain/close excluded, terminal write included; separate timing/allocation binaries','sources':{}}
    with tempfile.TemporaryDirectory(prefix='replai-direct-control-') as directory:
        for name,revision in [('before',a.before),('after',a.after)]:
            revision=subprocess.check_output(['git','rev-parse',revision],cwd=ROOT,text=True).strip()
            tree=subprocess.check_output(['git','rev-parse',revision+'^{tree}'],cwd=ROOT,text=True).strip()
            root=Path(directory)/name;root.mkdir();archive=Path(directory)/(name+'.tar')
            subprocess.run(['git','archive','--format=tar','--output',str(archive),revision],cwd=ROOT,check=True)
            with tarfile.open(archive) as tar:tar.extractall(root,filter='data')
            (root/'tools/perf/direct_completion.rs').write_bytes(fixture)
            manifest=root/'tools/perf/Cargo.toml'
            with manifest.open('a') as f:f.write('\n[[bin]]\nname="direct-completion"\npath="direct_completion.rs"\n')
            source={'revision':revision,'tree':tree,'results':{}}
            for mode in ['latency','allocation']:
                target=root/'target'/mode
                command=['cargo','build','--locked','--release','--manifest-path',str(manifest),'--target-dir',str(target),'--bin','direct-completion']
                if mode=='allocation':command+=['--features','allocations']
                subprocess.run(command,cwd=root,check=True)
                binary=target/'release/direct-completion'
                data=[json.loads(line) for line in subprocess.check_output([binary],cwd=root,text=True,env={**os.environ,'TERM':'xterm-256color','NO_COLOR':'1'}).splitlines()]
                for row in data:
                    if row['samples_us']:row['median_us']=statistics.median(row['samples_us'])
                source['results'][mode]={'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'measurements':data}
            result['sources'][name]=source
    a.out.write_text(json.dumps(result,indent=2)+'\n')
    for name,source in result['sources'].items():
        for row in source['results']['latency']['measurements']:print(name,row['operation'],row['median_us'])

if __name__=='__main__':main()
