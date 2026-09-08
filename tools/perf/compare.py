#!/usr/bin/env python3
"""Overlapping POSIX PTY burst-to-submission workloads, no performance ranking."""
import argparse
import json
import hashlib
import re
import subprocess
from pathlib import Path
import time
from linux import ROOT, HOST, Session
from run import LINENOISE


def run(work, samples, smoke):
    base=ROOT/'tools/perf/comparisons/target'
    manifest=(ROOT/'tools/perf/comparisons/Cargo.toml').read_text()
    revisions={name:re.search(r'^'+name+r' = .*rev = "([0-9a-f]{40})"',manifest,re.M).group(1) for name in ('rustyline','reedline')}
    revisions.update({'replai':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
                      'linenoise-blocking':LINENOISE,'linenoise-feed':LINENOISE})
    libraries=[('replai',[HOST,'--mode','comparison']),
               ('linenoise-blocking',[base/'linenoise-host']),
               ('linenoise-feed',[base/'linenoise-host','feed']),
               ('rustyline',[base/'release/rustyline-host']),
               ('reedline',[base/'release/reedline-host'])]
    cases=[('short_ascii','abcdefghX','abcdefghX'),
           ('long_ascii','a'*1000+'\x1b[DX','a'*999+'Xa'),
           ('unicode','café 界🌍\x1b[DX','café 界X🌍'),
           ('combining','cafe\u0301\x1b[DX','cafXe\u0301'),
           ('history','\x1b[A','history second'),
           ('completion','rep\t','replacement'),
           ('multiline','\x1b[200~first\nsecond\x1b[201~','first\nsecond')]
    if not smoke:cases.append(('long_ascii_4096','a'*4096+'\x1b[DX','a'*4095+'Xa'))
    for name,command in libraries:
        if not command[0].exists():
            yield dict(schema_version=1,id=f'comparison/{name}',component='comparison',status='unavailable',reason='fixture executable missing',library=name);continue
        binary_sha256=hashlib.sha256(command[0].read_bytes()).hexdigest()
        for case,wire,expected in cases:
            timings=[];traffic=[];restored=True;failure=None
            for n in range(samples+2):
                s=Session(command,terminal_stderr=name=='reedline')
                try:
                    if name=='replai':s.ready()
                    else:
                        s.until(lambda:b'> ' in s.output,timeout=15)
                        # Prompt write is the readiness barrier. Remaining optional
                        # initialization belongs to the observed library behavior.
                        s.output.clear()
                    start=time.perf_counter_ns();s.send(wire.encode()+b'\r')
                    s.until(lambda:b'SUBMITTED:' in s.receipts and b'\n' in s.receipts.split(b'SUBMITTED:',1)[1],timeout=30)
                    elapsed=(time.perf_counter_ns()-start)/1000
                    receipt=s.receipts.split(b'SUBMITTED:',1)[1].splitlines()[0]
                    actual=bytes.fromhex(receipt.decode()).decode()
                    if actual!=expected:
                        failure=f'nonoverlapping semantics: expected {len(expected.encode())} bytes, received {len(actual.encode())}; exact-text check failed'
                    s.finish()
                    byte_count=len(s.output)
                    if failure:break
                    if n>=2:timings.append(elapsed);traffic.append(byte_count)
                except (TimeoutError,RuntimeError,AssertionError) as exc:
                    failure=f'{type(exc).__name__}: {str(exc)[:240]}';s.abort();break
            if failure:
                yield dict(schema_version=1,id=f'comparison/{name}/{case}',component='comparison',library=name,library_revision=revisions[name],binary_sha256=binary_sha256,operation=case,status='not_comparable',reason=failure)
            else:
                yield dict(schema_version=1,id=f'comparison/{name}/{case}',component='comparison',library=name,library_revision=revisions[name],binary_sha256=binary_sha256,operation=case,status='measured',mode='latency',
                    iterations=samples,warmup=2,input_bytes=len(wire.encode()),columns=80,rows=24,samples_us=timings,
                    counters=dict(observed_terminal_bytes=traffic,submission_exact=True,terminal_restored=restored),
                    endpoint='single delivered input burst through submission hex receipt; includes editing, terminal output, submit cleanup and receipt IPC; not key-to-visible')


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--work',type=Path,required=True);p.add_argument('--samples',type=int,default=31);p.add_argument('--smoke',action='store_true');a=p.parse_args()
    a.work.mkdir(parents=True,exist_ok=True)
    for row in run(a.work,3 if a.smoke else a.samples,a.smoke):print(json.dumps(row),flush=True)
