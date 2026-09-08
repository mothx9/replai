#!/usr/bin/env python3
"""P0 result envelope, environment capture, summarization and strict validation."""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import statistics
import subprocess
import sys
import time

ROOT=Path(__file__).resolve().parents[2]
SCHEMA=1


def command(*args):
    try:return subprocess.check_output(args,cwd=ROOT,stderr=subprocess.STDOUT,text=True,timeout=20).strip()
    except (OSError,subprocess.SubprocessError) as exc:return 'unavailable: '+str(exc)


def identity():
    paths=sorted([*ROOT.glob('src/*.rs'), ROOT/'Cargo.toml', ROOT/'Cargo.lock', ROOT/'tests/support/posix_pty.rs',
        *[p for p in (ROOT/'tools/perf').rglob('*') if p.is_file() and 'target' not in p.parts and '__pycache__' not in p.parts]])
    digests={str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}
    return dict(head=command('git','rev-parse','HEAD'),tree=command('git','rev-parse','HEAD^{tree}'),
        branch=command('git','branch','--show-current'),dirty=bool(command('git','status','--porcelain')),
        source_sha256=hashlib.sha256(json.dumps(digests,sort_keys=True).encode()).hexdigest(),file_sha256=digests)


def environment():
    def read(path):
        try:return Path(path).read_text(encoding="utf-8").strip()
        except OSError:return None
    cpu=read('/proc/cpuinfo')
    # Preserve unique observed CPU identity fields without thousands of duplicates.
    cpu_identity=sorted(set(line for line in (cpu or '').splitlines() if any(k in line for k in ['model name','CPU implementer','CPU part','Hardware','vendor_id'])))
    governors={str(p):read(p) for p in Path('/sys/devices/system/cpu').glob('cpu*/cpufreq/scaling_governor')}
    return dict(recorded_utc=time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime()),hostname=platform.node(),
        os=platform.platform(),kernel=platform.release(),architecture=platform.machine(),cpu=cpu_identity or command('sysctl','-n','machdep.cpu.brand_string'),
        logical_cores=os.cpu_count(),affinity=sorted(os.sched_getaffinity(0)) if hasattr(os,'sched_getaffinity') else None,
        meminfo=read('/proc/meminfo'),governors=governors or None,loadavg=os.getloadavg() if hasattr(os,'getloadavg') else None,
        rust=command('rustc','-Vv'),cargo=command('cargo','-V'),c_compiler=command('cc','--version'),
        python=sys.version,clock=dict(implementation=time.get_clock_info('perf_counter').implementation,resolution_seconds=time.get_clock_info('perf_counter').resolution),
        strace=command('strace','-V'),valgrind=command('valgrind','--version'),
        build_environment={k:os.environ.get(k) for k in ['RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS','CARGO_BUILD_TARGET','CARGO_PROFILE_RELEASE_OPT_LEVEL']},build_profile='release opt-level=3 debug=1 panic=unwind, no LTO or CPU-native override',
        host_description=os.environ.get('P0_MACHINE','unspecified: supply P0_MACHINE for an authoritative run'),
        memory_cgroup_max=read('/sys/fs/cgroup/memory.max'),cpu_cgroup_max=read('/sys/fs/cgroup/cpu.max'),
        terminal=platform.system()+' openpty 80x24, controlling TTY, xterm-256color, NO_COLOR; no terminal emulator',
        dependencies=command('cargo','tree','--locked','-p','replai','--edges','normal,build'),
        host_hardware={k:command('sysctl','-n',k) for k in ['hw.model','machdep.cpu.brand_string','hw.memsize','hw.physicalcpu','hw.logicalcpu']} if sys.platform=='darwin' else None,
        macos=command('sw_vers') if sys.platform=='darwin' else None,
        os_packages=command('dpkg-query','-W','libc6','strace','valgrind','gcc','python3','nodejs'))


def distribution(values):
    assert values and all(isinstance(x,(int,float)) and math.isfinite(x) and x>=0 for x in values)
    ordered=sorted(values)
    percentile=lambda q:ordered[max(0,math.ceil(q*len(ordered))-1)]
    median=statistics.median(ordered)
    return dict(unit='us',median=median,p95=percentile(.95),p99=percentile(.99),min=ordered[0],max=ordered[-1],
                mad=statistics.median(abs(v-median) for v in ordered),samples=len(values))


def summarize(row):
    row=dict(row)
    values=row.pop('samples_us',None)
    if values is not None:
        row['timing']=distribution(values)
        per_group=row.get('samples_per_group')
        if per_group:row['group_medians_us']=[statistics.median(values[i:i+per_group]) for i in range(0,len(values),per_group)]
        if row.get('input_bytes',0)>0 and row['timing']['median']>0:
            row['input_bytes_per_second']=row['input_bytes']/(row['timing']['median']/1e6)
    memories=row.pop('allocation_samples',None)
    if memories is not None:
        row['allocations']={key:dict(median=statistics.median(v[key] for v in memories),min=min(v[key] for v in memories),max=max(v[key] for v in memories)) for key in memories[0]}
    traffic=row.get('counters',{}).get('observed_terminal_bytes')
    if isinstance(traffic,list):row['counters']['observed_terminal_bytes']=dict(min=min(traffic),max=max(traffic),median=statistics.median(traffic))
    return row


def validate(data):
    assert data['schema_version']==SCHEMA
    assert isinstance(data['source']['head'],str) and len(data['source']['head'])==40
    assert isinstance(data['environment'],dict) and data['environment']['rust']
    assert data['methodology']['outlier_policy']=='retain all valid samples; no trimming'
    seen=set()
    for row in data['results']:
        assert row['schema_version']==SCHEMA
        key=(row['id'],row.get('mode'))
        assert key not in seen,f'duplicate result {key}'
        seen.add(key)
        assert row['status'] in ('measured','unsupported','unavailable','not_comparable')
        assert row['component']
        if row['status']!='measured':assert row['reason'];continue
        assert row['iterations']>0
        timing=row.get('timing')
        if timing:
            assert row.get('mode')=='latency'
            assert timing['unit']=='us'
            assert timing['samples']==row['iterations']
            assert all(isinstance(timing[k],(int,float)) and math.isfinite(timing[k]) and timing[k]>=0 for k in ['min','median','p95','p99','max','mad'])
            assert timing['min']<=timing['median']<=timing['p95']<=timing['p99']<=timing['max']
        if row.get('mode')=='allocation':assert not timing and row['allocations']
    return len(seen)


def envelope(raw_files, receipt):
    rows=[]
    for path in raw_files:
        for line in path.read_text(encoding="utf-8").splitlines():
            if line.strip():rows.append(summarize(json.loads(line)))
    data=dict(schema_version=SCHEMA,source=receipt['source'],environment=receipt['environment'],
        methodology=dict(outlier_policy='retain all valid samples; no trimming',percentiles='nearest rank; median averages middle pair',
            warmup='2 per microbenchmark group and 2 PTY/comparison invocations; output/idle report no warmup',
            clocks='monotonic; microseconds are serialization units, not a precision guarantee',
            allocation='separate feature build; System allocation/reallocation requests and logical live bytes; allocator metadata/RSS/internal realloc overlap unobserved',
            micro_timing='setup, validation and returned-value drop excluded; drops inside operation included; control/timer floor recorded',
            pty='includes scheduler/IPC and kernel PTY delivery; does not measure emulator repaint or physical display',
            authority='recorded development VM; descriptive baseline, not a regression-grade isolated bare-metal lab'),results=rows)
    validate(data);return data


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);sub=p.add_subparsers(dest='command',required=True)
    record=sub.add_parser('record');record.add_argument('output',type=Path)
    merge=sub.add_parser('summarize');merge.add_argument('--receipt',type=Path,required=True);merge.add_argument('--output',type=Path,required=True);merge.add_argument('raw',nargs='+',type=Path)
    check=sub.add_parser('validate');check.add_argument('path',type=Path)
    a=p.parse_args()
    if a.command=='record':a.output.write_text(json.dumps(dict(source=identity(),environment=environment()),indent=2)+'\n')
    elif a.command=='summarize':a.output.write_text(json.dumps(envelope(a.raw,json.loads(a.receipt.read_text(encoding="utf-8"))),separators=(',',':'))+'\n')
    else:print(f'valid: {validate(json.loads(a.path.read_text(encoding="utf-8")))} results')
