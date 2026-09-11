#!/usr/bin/env python3
"""Freeze final fuzz inputs with provenance; replay identical bytes on each supported OS."""
import argparse
import base64
import gzip
import hashlib
import json
import platform
from pathlib import Path
import re
import subprocess
import time
import campaign

ROOT = Path(__file__).resolve().parents[2]
MAX_ARCHIVE_BYTES = 128 * 1024 * 1024


def read_archive(path):
    with gzip.open(path, 'rb') as source:
        data = source.read(MAX_ARCHIVE_BYTES + 1)
    assert len(data) <= MAX_ARCHIVE_BYTES, 'archive exceeds bounded replay storage'
    archive = json.loads(data)
    assert archive['version'] == 1 and set(archive['targets']) == set(campaign.TARGETS)
    for name, record in archive['targets'].items():
        summary = record['summary']
        assert summary['target'] == name and summary['passed'] and summary['budget_complete']
        assert summary['cpu_seconds'] >= 3600
        hashes = []
        for value in record['inputs']:
            raw = base64.b64decode(value, validate=True)
            assert len(raw) <= (8192 if name == 'geometry' else 4096)
            hashes.append(hashlib.sha256(raw).hexdigest())
        digest = hashlib.sha256('\n'.join(sorted(hashes)).encode()).hexdigest()
        assert summary['final_corpus'] == {'files': len(hashes), 'sha256': digest}, name
    return archive


def main():
    p = argparse.ArgumentParser(description=__doc__)
    sub = p.add_subparsers(dest='mode', required=True)
    pack = sub.add_parser('pack')
    pack.add_argument('--campaign', type=Path, action='append', required=True,
                      help='ordered sources; last completed target supersedes earlier target')
    pack.add_argument('--output', type=Path, required=True)
    replay = sub.add_parser('replay')
    replay.add_argument('--archive', type=Path, required=True)
    replay.add_argument('--work', type=Path, required=True)
    a = p.parse_args()
    if a.mode == 'pack':
        targets = {}
        for root in a.campaign:
            environment = json.loads((root/'environment.json').read_text())
            assert not environment['dirty']
            for name in campaign.TARGETS:
                path = root/name/'summary.json'
                if not path.exists():
                    continue
                s = json.loads(path.read_text())
                if not s['passed'] or not s['budget_complete']:
                    continue
                inputs = [base64.b64encode(f.read_bytes()).decode('ascii') for f in sorted((root/name/'corpus').iterdir()) if f.is_file()]
                executions = sum(int(n) for log in (root/name).glob('chunk-*.log')
                                 for n in re.findall(r'stat::number_of_executed_units:\s*(\d+)', log.read_text()))
                targets[name] = {'environment': environment, 'summary': s, 'executions_including_chunk_initialization': executions, 'inputs': inputs}
        assert set(targets) == set(campaign.TARGETS), 'every final target needs a completed recorded budget'
        a.output.parent.mkdir(parents=True, exist_ok=True)
        with a.output.open('xb') as output:
            output.write(gzip.compress(json.dumps({'version':1, 'targets':targets}, sort_keys=True).encode(), mtime=0))
        read_archive(a.output)
        print(json.dumps({'archive':str(a.output), 'sha256':hashlib.sha256(a.output.read_bytes()).hexdigest(), 'bytes':a.output.stat().st_size}))
    else:
        archive = read_archive(a.archive)
        a.work.mkdir(parents=True, exist_ok=False)
        subprocess.run(['cargo','build','--locked','--release','--manifest-path','tools/hardening/Cargo.toml','--bin','replay'],cwd=ROOT,check=True)
        binary = ROOT/'tools/hardening/target/release'/('replay.exe' if platform.system()=='Windows' else 'replay')
        receipt = {'head':campaign.command('git','rev-parse','HEAD'), 'tree':campaign.command('git','rev-parse','HEAD^{tree}'),
                   'environment':platform.uname()._asdict(), 'rustc':campaign.command('rustc','-vV'),
                   'archive_sha256':hashlib.sha256(a.archive.read_bytes()).hexdigest(), 'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(), 'results':[]}
        for name, record in archive['targets'].items():
            if name == 'cabi' and platform.system()=='Windows':
                receipt['results'].append({'target':name,'status':'NOT_APPLICABLE','reason':'no Windows terminal backend'})
                continue
            directory = a.work/name
            directory.mkdir()
            for index, value in enumerate(record['inputs']):
                (directory/str(index)).write_bytes(base64.b64decode(value))
            started = time.time()
            with (a.work/(name+'.stdout')).open('w') as out, (a.work/(name+'.stderr')).open('w') as err:
                result = subprocess.run([str(binary),name,str(directory.resolve())],stdout=out,stderr=err,timeout=1800)
            receipt['results'].append({'target':name,'status':'PASS' if result.returncode==0 else 'FAIL',
                                       'corpus':record['summary']['final_corpus'],'start_unix':started,'end_unix':time.time()})
            (a.work/'summary.json').write_text(json.dumps(receipt,indent=2)+'\n')
            result.check_returncode()
            print(name, 'PASS', len(record['inputs']), flush=True)


if __name__=='__main__':
    main()
