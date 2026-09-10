#!/usr/bin/env python3
"""Interleaved exact P0 long_ascii workload against two explicitly built PTY hosts.

Build --before from a git archive of the declared baseline; --after is the current
release pty-host. This adds no timing threshold or cross-library speed ranking.
"""
import argparse
import hashlib
import json
from pathlib import Path
import statistics
import subprocess
import sys
import time
import tempfile
sys.path.insert(0, str(Path(__file__).resolve().parent / 'perf'))
from linux import Session, cases, COMPONENTS, winsize


def run(binary):
    s = Session([binary, '--mode', 'comparison'])
    try:
        s.ready()
        start = time.perf_counter_ns()
        s.send(b'a'*1000+b'\x1b[DX\r')
        s.until(lambda:b'SUBMITTED:' in s.receipts and b'\n' in s.receipts.split(b'SUBMITTED:',1)[1])
        elapsed = (time.perf_counter_ns()-start)/1000
        actual = bytes.fromhex(s.receipts.split(b'SUBMITTED:',1)[1].splitlines()[0].decode())
        assert actual == b'a'*999+b'Xa'
        s.finish() # Verifies exact terminal restoration and successful process exit.
        return elapsed, len(s.output)
    except BaseException:
        s.abort(); raise


def visible(binary, path, case, prediction):
    s = Session([binary, '--case-file', path])
    try:
        s.ready(prediction['initial_output'].encode())
        expected = prediction['output'].encode()
        start = time.perf_counter_ns()
        if 'resize' in case: winsize(s.slave, case['resize'])
        s.send(case['input'].encode())
        s.until(lambda:len(s.output)>=len(expected))
        elapsed = (time.perf_counter_ns()-start)/1000
        assert s.output == expected
        s.send(b'\r'); s.until(lambda:b'"submitted"' in s.receipts)
        s.finish()
        received=[json.loads(line) for line in s.receipts.splitlines() if line.startswith(b'{')]
        assert next(v['submitted'] for v in received if 'submitted' in v) == case['expected']
        return elapsed
    except BaseException:
        s.abort(); raise


if __name__ == '__main__':
    p=argparse.ArgumentParser(description=__doc__)
    for name in ['before','after']:
        p.add_argument('--'+name, type=Path, required=True)
        p.add_argument('--'+name+'-revision', required=True)
    p.add_argument('--out', type=Path, required=True)
    a=p.parse_args()
    records={name:dict(binary_sha256=hashlib.sha256(getattr(a,name).read_bytes()).hexdigest(),
                      revision=getattr(a,name+'_revision'), samples_us=[], terminal_bytes=[])
             for name in ['before','after']}
    for i in range(65):
        # Alternate order; two warm-up pairs then 63 observations per source.
        for name in (['before','after'] if i%2==0 else ['after','before']):
            elapsed, traffic=run(getattr(a,name))
            if i>=2:
                records[name]['samples_us'].append(elapsed)
                records[name]['terminal_bytes'].append(traffic)
    for r in records.values():
        r['median_us']=statistics.median(r['samples_us'])
        r['p95_us']=sorted(r['samples_us'])[int(len(r['samples_us'])*.95)]
    visible_cases = {}
    with tempfile.TemporaryDirectory(prefix='replai-i0-paired-') as directory:
        path = Path(directory)/'case.json'
        for case in cases(True):
            path.write_text(json.dumps(case))
            prediction=json.loads(subprocess.check_output([COMPONENTS,'--predict',path]))
            samples = {name:[] for name in records}
            for i in range(33):
                for name in (['before','after'] if i%2==0 else ['after','before']):
                    elapsed = visible(getattr(a,name),path,case,prediction)
                    if i>=2:samples[name].append(elapsed)
            visible_cases[case['name']] = {name:dict(samples_us=values,median_us=statistics.median(values)) for name,values in samples.items()}
    result=dict(workload='P0 compare.py long_ascii: 1000 ASCII bytes + Left + X + Enter',
                endpoint='delivered burst through exact submission receipt, including cleanup and IPC; not key-to-visible',
                order='alternating before/after; two warm-up pairs; 63 samples per source',
                machine=subprocess.check_output(['uname','-a'],text=True).strip(),
                harness_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
                records=records, visible_cases=visible_cases,
                visible_endpoint='existing linux.py exact terminal-byte oracle; 31 samples per source, alternating order; separate from submission workload')
    a.out.write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps({name:{k:v for k,v in r.items() if k not in ['samples_us','terminal_bytes']} for name,r in records.items()},indent=2))

    for name, cases in visible_cases.items():
        print(name, cases['before']['median_us'], cases['after']['median_us'])
