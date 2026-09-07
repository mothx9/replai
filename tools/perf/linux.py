#!/usr/bin/env python3
"""Real Linux PTYs; readiness and exact-byte completion, never quiet-time sleeps."""
import argparse
import errno
import fcntl
import json
import os
from pathlib import Path
import re
import resource
import selectors
import shutil
import signal
import struct
import subprocess
import termios
import time

ROOT = Path(__file__).resolve().parents[2]
HOST = ROOT / 'tools/perf/target/release/pty-host'
COMPONENTS = ROOT / 'tools/perf/target/release/components'


def winsize(fd, columns):
    fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack('HHHH', 24, columns, 0, 0))


class Session:
    def __init__(self, command, trace=None, terminal_stderr=False):
        self.master, self.slave = os.openpty()
        winsize(self.slave, 80)
        self.saved = termios.tcgetattr(self.slave)
        os.set_blocking(self.master, False)
        self.output = bytearray()
        self.receipts = bytearray()
        self.rusage_before = resource.getrusage(resource.RUSAGE_CHILDREN)
        if trace:
            command = ['strace', '-qq', '-ttt', '-yy', '-s', '40', '-e',
                       'trace=write,writev,read,ioctl,ppoll,poll', '-o', str(trace), *map(str, command)]
        def controlling_terminal():
            os.setsid()
            fcntl.ioctl(0, termios.TIOCSCTTY, 0)
        control_read, control_write = os.pipe() if terminal_stderr else (None, None)
        self.process = subprocess.Popen(list(map(str, command)), stdin=self.slave,
            stdout=self.slave, stderr=self.slave if terminal_stderr else subprocess.PIPE, preexec_fn=controlling_terminal,
            pass_fds=(control_write,) if terminal_stderr else (),
            env={**os.environ, 'TERM': 'xterm-256color', 'NO_COLOR': '1', 'LC_ALL': 'C.UTF-8', **({'P0_RECEIPT_FD':str(control_write)} if terminal_stderr else {})})
        if terminal_stderr:
            os.close(control_write)
            self.control=os.fdopen(control_read,'rb')
        else:self.control=self.process.stderr
        self.selector = selectors.DefaultSelector()
        self.selector.register(self.master, selectors.EVENT_READ)
        self.selector.register(self.control, selectors.EVENT_READ)

    def pump(self, deadline):
        remaining = deadline-time.monotonic()
        if remaining <= 0:
            raise TimeoutError(f'PTY timeout: exit={self.process.poll()}, stderr={self.receipts[-500:]!r}, output={self.output[-200:]!r}')
        for key, events in self.selector.select(remaining):
            if key.fd==self.master and not events & selectors.EVENT_READ:
                continue
            try:
                data = os.read(key.fd, 65536)
            except OSError as exc:
                if exc.errno == errno.EIO:
                    data = b''
                else:
                    raise
            if not data:
                self.selector.unregister(key.fileobj)
            elif key.fd == self.master:
                self.output.extend(data)
                # Terminal query response, used ONLY by comparison fixtures.
                if b'\x1b[6n' in data:
                    os.write(self.master, b'\x1b[1;1R')
            else:
                self.receipts.extend(data)

    def until(self, predicate, timeout=30):
        deadline=time.monotonic()+timeout
        while not predicate():
            if self.process.poll() is not None and self.control not in self.selector.get_map():
                while True:
                    try:
                        data=os.read(self.master,65536)
                        if not data:break
                        self.output.extend(data)
                    except BlockingIOError:break
                if predicate():return
                raise RuntimeError(f'PTY exited prematurely: {self.receipts[-500:]!r}')
            self.pump(deadline)

    def send(self, data, timeout=120):
        end=time.monotonic()+timeout
        data=memoryview(data)
        while data:
            try:
                n=os.write(self.master, data)
                data=data[n:]
            except BlockingIOError:
                # Paste emits nothing until the closing delimiter. Waiting only
                # for readable output here deadlocks and expires a valid paste.
                self.selector.modify(self.master,selectors.EVENT_READ|selectors.EVENT_WRITE)
                try:self.pump(end)
                finally:self.selector.modify(self.master,selectors.EVENT_READ)

    def ready(self, expected_initial=None, gate=False):
        self.until(lambda: b'READY\n' in self.receipts)
        if expected_initial is not None:
            expected=b'\x1b[?2004h'+expected_initial
            self.until(lambda:len(self.output)>=len(expected))
            assert self.output == expected, (self.output[-100:],expected[-100:])
        else:
            # READY is emitted only after production open writes. Drain currently
            # queued output without a sleep before beginning the workload.
            while True:
                try:
                    data=os.read(self.master,65536)
                    if not data:break
                    self.output.extend(data)
                except BlockingIOError:break
        self.output.clear()
        if gate:self.send(b"!")

    def finish(self):
        deadline=time.monotonic()+30
        while self.process.poll() is None:
            if self.control not in self.selector.get_map():
                self.process.wait(timeout=30)
                break
            self.pump(deadline)
        # Read remaining pipe output after exit.
        self.receipts.extend(self.control.read())
        code=self.process.wait()
        # Process exit/receipt EOF does not drain the PTY queue. Count all bytes
        # before closing the retained slave/master pair (no quiet-time sleep).
        while True:
            try:
                data=os.read(self.master,65536)
                if not data:break
                self.output.extend(data)
            except BlockingIOError:break
        assert code==0,(code,self.receipts[-1000:])
        assert termios.tcgetattr(self.slave)==self.saved, 'terminal state not restored'
        now=resource.getrusage(resource.RUSAGE_CHILDREN)
        cpu=(now.ru_utime+now.ru_stime)-(self.rusage_before.ru_utime+self.rusage_before.ru_stime)
        self.selector.close();self.control.close()
        os.close(self.master);os.close(self.slave)
        return cpu

    def abort(self):
        if self.process.poll() is None:
            os.killpg(self.process.pid,signal.SIGKILL)
            self.process.wait()
        self.selector.close();self.control.close()
        os.close(self.master);os.close(self.slave)


def cases(smoke=False):
    result=[
        dict(name='ascii_append',initial='x'*64,input='X',expected='x'*64+'X'),
        dict(name='unicode_append',initial='café 界',input='🌍',expected='café 界🌍'),
        dict(name='middle_insert',initial='a'*64,cursor=32,input='X',expected='a'*32+'X'+'a'*32),
        dict(name='middle_edit',initial='a'*64,input='\x01'+'\x1b[C'*32+'X',expected='a'*32+'X'+'a'*32),
        dict(name='cursor_movement',initial='café 界',input='\x1b[D',expected='café 界'),
        dict(name='history',initial='unsent',input='\x1b[A',expected='history second'),
        dict(name='completion',initial='rep',input='\t',replacement='replacement',expected='replacement'),
        dict(name='ctrl_l',initial='draft',input='\x0c',expected='draft'),
        dict(name='resize',initial='draft '*20,input='',resize=20,expected='draft '*20),
        dict(name='multiline_paste',initial='',input='\x1b[200~first\n界 second\nthird\x1b[201~',expected='first\n界 second\nthird'),
    ]
    if smoke:
        body='a'*65536
        result.append(dict(name='paste_backpressure_65536',initial='',input='\x1b[200~'+body+'\x1b[201~',expected=body,text_class='ascii',input_bytes=65536))
    if not smoke:
        # A single edit with cursor setup excluded, to separate it from the
        # compound navigation+edit scenario above.
        result.append(dict(name='long_ascii_append',initial='a'*4096,input='X',expected='a'*4096+'X'))
        for size in [1024,4096,16384,65536,262144,1048576]:
            for name,unit in [('ascii','abcd '),('prose','café 界 prose '),('source','fn x() {\n    f();\n}\n'),('json','{"key":42}\n'),('mixed','café\n界\t🌍\n')]:
                unit_bytes=len(unit.encode());body=unit*(size//unit_bytes)+'x'*(size%unit_bytes)
                result.append(dict(name=f'paste_{name}_{size}',initial='',input='\x1b[200~'+body+'\x1b[201~',expected=body,text_class=name,input_bytes=size))
    return result


def trace_counts(path, start=None, end=None):
    counts={'write_syscalls':0,'terminal_bytes_written':0,'read_syscalls':0,'dimension_queries':0,'poll_syscalls':0}
    active=False
    for line in path.read_text().splitlines():
        if '"READY\\n"' in line:
            active=True;continue
        if not active:continue
        if start is not None:
            stamp=float(line.split()[0])
            if not start<=stamp<=end:continue
        tty=('/dev/pts/' in line or '/dev/tty>' in line)
        if tty and re.search(r'\bwrite(?:v)?\(',line):
            counts['write_syscalls']+=1
            match=re.search(r'= (\d+)\s*$',line)
            if match:counts['terminal_bytes_written']+=int(match[1])
        if tty and 'read(' in line:counts['read_syscalls']+=1
        if 'TIOCGWINSZ' in line:counts['dimension_queries']+=1
        if 'ppoll(' in line or 'poll(' in line:counts['poll_syscalls']+=1
    return counts


def interactive(work, smoke, samples):
    for case in cases(smoke):
        path=work/'case.json';path.write_text(json.dumps(case))
        prediction=json.loads(subprocess.check_output([COMPONENTS,'--predict',path]))
        assert prediction['text']==case['expected'],case['name']
        expected=prediction['output'].encode()
        assert expected,case['name']
        timings=[];bytes_out=[]
        for n in range(samples+2):
            session=Session([HOST,'--case-file',path])
            try:
                session.ready(prediction['initial_output'].encode())
                start=time.perf_counter_ns()
                if 'resize' in case:winsize(session.slave,case['resize'])
                session.send(case['input'].encode())
                session.until(lambda:len(session.output)>=len(expected),timeout=180)
                elapsed=(time.perf_counter_ns()-start)/1000
                assert session.output==expected,case['name']
                if n>=2:timings.append(elapsed);bytes_out.append(len(session.output))
                session.send(b'\r')
                session.until(lambda:b'"submitted"' in session.receipts,timeout=180)
                session.finish()
                received=[json.loads(s) for s in session.receipts.splitlines() if s.startswith(b'{')]
                submitted=next(v['submitted'] for v in received if 'submitted' in v)
                assert submitted==case['expected'],case['name']
            except BaseException:
                session.abort();raise
        yield dict(schema_version=1,id='pty/'+case['name'],component='pty',operation=case['name'],
            mode='latency',status='measured',input_bytes=case.get('input_bytes',len(case['input'].encode())),
            initial_bytes=len(case['initial'].encode()),text_class=case.get('text_class','mixed'),
            columns=case.get('resize',80),rows=24,iterations=samples,warmup=2,samples_us=timings,
            counters=dict(vt_bytes=bytes_out[0],logical_mutations=prediction['logical_mutations'],terminal_restored=True,submission_exact=True),
            endpoint='master input write start to exact final VT byte read; includes two-process scheduling and PTY delivery')
        # Tracing is a separate invocation. Bound substantial traces to 4 KiB.
        if shutil.which('strace') and len(case['input'].encode())<=4110:
            trace=work/(case['name']+'.strace');session=Session([HOST,'--case-file',path],trace)
            try:
                session.ready(prediction['initial_output'].encode());start=time.time()
                if 'resize' in case:winsize(session.slave,case['resize'])
                session.send(case['input'].encode());session.until(lambda:len(session.output)>=len(expected))
                end=time.time();assert session.output==expected
                session.send(b'\r');session.until(lambda:b'"submitted"' in session.receipts);session.finish()
            except BaseException:session.abort();raise
            yield dict(schema_version=1,id='syscall/'+case['name'],component='transport',operation=case['name'],mode='syscall',status='measured',iterations=1,
                input_bytes=len(case['input'].encode()),columns=case.get('resize',80),rows=24,
                counters=trace_counts(trace,start,end),scope='strace entry timestamps inside delivery interval; already-pending poll may be excluded; no traced latency')


def idle_and_output(work, smoke):
    for timeout in [0,1,10,100,1000]:
        seconds=0.15 if smoke else (1 if timeout==0 else 3)
        observations=[]
        for repeat in range(1 if smoke else 3):
            session=Session([HOST,'--mode','idle','--timeout-ms',timeout,'--seconds',seconds])
            try:
                session.ready(gate=True);start=time.monotonic()
                context=Path(f'/proc/{session.process.pid}/status').read_text()
                rss=re.search(r'^VmRSS:\s+(\d+) kB',context,re.M)
                resident_kib=int(rss[1]) if rss else None
                session.until(lambda:b'"poll_calls"' in session.receipts,timeout=seconds+10)
                # Child-reported calls provide exact waits; /proc snapshot is a
                # coarse optional view. wait4 rusage includes startup+cleanup.
                cpu=session.finish()
                value=next(json.loads(x) for x in session.receipts.splitlines() if x.startswith(b'{'))
                value.update(cpu_seconds_lifecycle=cpu,window_seconds=time.monotonic()-start,resident_kib_at_ready=resident_kib)
                observations.append(value)
            except BaseException:session.abort();raise
        yield dict(schema_version=1,id=f'idle/{timeout}',component='idle',operation='poll',status='measured',mode='idle',iterations=len(observations),observations=observations,
                   scope='production poll calls and lifecycle-inclusive child rusage; zero timeout is deliberate busy polling')
        if shutil.which('strace') and timeout>=10:
            trace=work/f'idle-{timeout}.strace';session=Session([HOST,'--mode','idle','--timeout-ms',timeout,'--seconds',seconds],trace)
            try:session.ready(gate=True);session.until(lambda:b'"poll_calls"' in session.receipts,timeout=seconds+10);session.finish()
            except BaseException:session.abort();raise
            value=next(json.loads(x) for x in session.receipts.splitlines() if x.startswith(b'{'))
            yield dict(schema_version=1,id=f'idle_syscall/{timeout}',component='idle',operation='poll',status='measured',mode='syscall',iterations=1,
                       counters=trace_counts(trace),elapsed_seconds=value['elapsed_seconds'],poll_calls=value['poll_calls'],scope='after READY through cleanup; CPU from untraced run only')
    for size in ([80,16384] if smoke else [16,80,1024,4096,16384]):
        for rate in ([20] if smoke else [5,20,50,100,200]):
            seconds=0.2 if smoke else 2
            session=Session([HOST,'--mode','output','--initial','retained draft 界','--size',size,'--rate',rate,'--seconds',seconds])
            try:
                session.ready(gate=True);session.until(lambda:b'"draft_restored"' in session.receipts,timeout=seconds+30)
                cpu=session.finish();output=len(session.output)
            except BaseException:session.abort();raise
            value=next(json.loads(x) for x in session.receipts.splitlines() if x.startswith(b'{'))
            samples_us=value.pop('samples_us')
            yield dict(schema_version=1,id=f'output/{size}/{rate}',component='output',operation='synchronous_chunks',status='measured',mode='latency',iterations=len(samples_us),
                samples_us=samples_us,input_bytes=size,rate=rate,observations=value,counters=dict(observed_terminal_bytes_including_close=output,cpu_seconds_lifecycle=cpu,terminal_restored=True),
                scope='host-call timer excludes rate scheduling sleeps; serialized single host, no concurrent editing')
            if shutil.which('strace'):
                trace=work/f'output-{size}-{rate}.strace'
                session=Session([HOST,'--mode','output','--initial','retained draft 界','--size',size,'--rate',rate,'--seconds',seconds],trace)
                try:session.ready(gate=True);session.until(lambda:b'"draft_restored"' in session.receipts,timeout=seconds+30);session.finish()
                except BaseException:session.abort();raise
                yield dict(schema_version=1,id=f'output_syscall/{size}/{rate}',component='output',operation='synchronous_chunks',status='measured',mode='syscall',iterations=1,
                    input_bytes=size,rate=rate,counters=trace_counts(trace),scope='after READY through close; subtract separately reported close bytes/write for per-call cost')


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--work',type=Path,required=True);p.add_argument('--smoke',action='store_true');p.add_argument('--samples',type=int,default=31);p.add_argument('--family',choices=['all','pty','idle-output'],default='all');a=p.parse_args()
    if not sys_platform_linux():
        print(json.dumps(dict(schema_version=1,id='linux',component='platform',status='unsupported',reason='qualified real backend requires Linux')));return
    a.work.mkdir(parents=True,exist_ok=True)
    generators=[]
    if a.family in ('all','pty'):generators.append(interactive(a.work,a.smoke,3 if a.smoke else a.samples))
    if a.family in ('all','idle-output'):generators.append(idle_and_output(a.work,a.smoke))
    for generator in generators:
        for row in generator:print(json.dumps(row),flush=True)
    if not shutil.which('strace'):
        print(json.dumps(dict(schema_version=1,id='strace',component='tooling',status='unavailable',reason='optional strace missing; syscalls not inferred from logical writes')))


def sys_platform_linux():
    import sys
    return sys.platform=='linux'


if __name__=='__main__':main()
