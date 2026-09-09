#!/usr/bin/env python3
"""Real external-reactor and blocking examples; reusable Linux/macOS terminal oracle."""
import argparse
import json
import os
from pathlib import Path
import re
import socket
import shutil
import subprocess
import sys
import tempfile
import time

sys.path.insert(0, str(Path(__file__).resolve().parent / 'perf'))
from linux import Session, attributes, winsize, ROOT


class Reactor(Session):
    def __init__(self, directory, prefix=()):
        path = directory / 'events.sock'
        self.listener = socket.socket(socket.AF_UNIX)
        self.listener.bind(str(path)); self.listener.listen(1); self.listener.settimeout(20)
        self.operations = ['R 24 80']
        super().__init__([*prefix, ROOT/'target/debug/examples/driven', path])
        self.app, _ = self.listener.accept()
        self.until(lambda: b'READY\n' in self.receipts and b'driven> ' in self.output)

    def pump(self, deadline):
        mark = len(self.output)
        super().pump(deadline)
        if len(self.output) > mark:
            self.operations.append('D ' + self.output[mark:].hex())

    def state(self):
        matches = re.findall(rb'STATE (true|false) (\d+) (\d+) ([0-9a-f]*)\n', self.receipts)
        if not matches: return None
        opened, cursor, advances, text = matches[-1]
        return dict(open=opened == b'true', cursor=int(cursor), advances=int(advances), text=bytes.fromhex(text.decode()).decode())

    def event(self, data):
        mark = len(self.receipts)
        self.app.sendall(data)
        self.until(lambda: b'STATE ' in self.receipts[mark:] and self.receipts.endswith(b'\n'))
        return self.state()

    def edit(self, data, text, cursor):
        mark = len(self.receipts)
        self.send(data)
        self.until(lambda: b'STATE ' in self.receipts[mark:] and self.state()['text'] == text and self.state()['cursor'] == cursor)
        return self.state()

    def screen(self):
        # Barrier: the host STATE receipt follows synchronous terminal writes.
        while True:
            try: data = os.read(self.master, 65536)
            except BlockingIOError: break
            if not data: break
            self.output.extend(data); self.operations.append('D ' + data.hex())
        result = subprocess.run([ROOT/'target/debug/examples/terminal-state'], input='\n'.join(self.operations)+'\n', text=True, capture_output=True, check=True)
        rows = dict(line.split(' ', 1) for line in result.stdout.splitlines() if not line.startswith('cell '))
        rows['text'] = bytes.fromhex(rows['text']).decode()
        assert rows['alternate'] == 'false' and rows['background_default'] == 'true', rows
        return rows

    def restored(self):
        assert attributes(self.slave) == self.saved, 'exact termios before process exit'
        assert self.screen()['paste'] == 'false', 'paste mode not disabled'

    def reopen(self):
        self.restored(); result = self.event(b'N')
        assert result['open'] and result['text'] == ''

    def stop(self):
        self.event(b'Q'); self.restored()
        self.app.sendall(b'E'); self.until(lambda: self.process.poll() is not None, timeout=60)
        assert self.process.returncode == 0
        self.receipts.extend(self.control.read())
        while True:
            try: data = os.read(self.master, 65536)
            except BlockingIOError: break
            if not data: break
            self.output.extend(data)
        self.app.close(); self.listener.close(); self.abort()


def driven(work, prefix=()):
    with tempfile.TemporaryDirectory(prefix='replai-reactor-') as directory:
        s = Reactor(Path(directory), prefix)
        evidence = {}
        try:
            evidence['sizes_bytes'] = dict(zip(['Interaction','Deadline','WaitInterest','Wake'], map(int, re.search(rb'SIZES (\d+) (\d+) (\d+) (\d+)', s.receipts).groups())))
            initial = s.event(b'S')
            cpu0 = process_cpu(s.process.pid)
            time.sleep(0.4)  # Observer interval: the child is blocked in the host reactor.
            idle = s.event(b'S'); cpu1 = process_cpu(s.process.pid)
            assert initial['advances'] == idle['advances'] == 0
            evidence['idle'] = dict(seconds=0.4, replai_advancements=0, required_deadline=None, child_cpu_seconds=None if cpu0 is None else cpu1-cpu0,
                scope='host blocks on terminal/socket; no REPLAI call between observer events')
            text = 'e\u0301!界'
            evidence['unicode'] = s.edit('e\u0301界\x1b[D!'.encode(), text, 4)
            evidence['unicode_ready_advance_ns'] = int(re.findall(rb'READY_ADVANCE_NS (\d+)', s.receipts)[-1])
            before = s.screen(); assert before['cursor'] == '0 10', before
            evidence['output'] = s.event(b'O'); after = s.screen()
            assert s.state()['text'] == text and s.state()['cursor'] == 4
            assert 'application event: draft preserved' in after['text'] and 'driven> '+text in after['text']
            assert after['cursor'].split()[1] == '10', after
            evidence['output_screen'] = after
            s.edit(b'\r', text, 4); s.until(lambda: not s.state()['open']); s.restored(); s.reopen()
            s.edit(b'draft\x1b[D', 'draft', 4)
            s.edit(b'\x1b[A', text, len(text.encode()))
            evidence['history_return'] = s.edit(b'\x1b[B', 'draft', 4)
            s.edit(b'\x03', 'draft', 4); s.until(lambda: not s.state()['open']); s.reopen()
            evidence['completion'] = s.edit(b'he\t', 'hello', 5)
            s.event(b'X'); s.reopen()
            pasted = 'first\n界 second\nthird'
            evidence['paste'] = s.edit('\x1b[200~first\r\n界 second\rthird\x1b[201~'.encode(), pasted, len(pasted.encode()))
            s.screen(); s.operations.append('R 24 12'); winsize(s.slave, 12)
            start = time.perf_counter_ns(); evidence['resize'] = s.event(b'R')
            evidence['resize_event_to_receipt_us'] = (time.perf_counter_ns()-start)/1000
            screen = s.screen(); assert 'third' in screen['text'], screen
            evidence['resized_screen'] = screen
            evidence['ctrl_l'] = s.edit(b'\x0c', pasted, len(pasted.encode()))
            s.event(b'X'); s.reopen()
            mark = len(s.receipts); s.send(b'\x1b[')
            s.until(lambda: b'STATE ' in s.receipts[mark:])
            s.until(lambda: b'DEADLINE\n' in s.receipts[mark:] and b'REJECTED ' in s.receipts[mark:])
            assert s.receipts[mark:].count(b'DEADLINE\n') == 1
            evidence['deadline_expiry'] = s.receipts[mark:].decode()
            s.edit(b'ab', 'ab', 2)
            mark = len(s.receipts); s.send(b'\x1b['); s.until(lambda:b'STATE ' in s.receipts[mark:])
            time.sleep(0.15)
            s.edit(b'D!', 'a!b', 2)
            time.sleep(0.15); s.event(b'S')
            assert b'DEADLINE\n' not in s.receipts[mark:]
            evidence['deadline_superseded'] = s.state()
            s.event(b'X'); s.reopen()
            fd_counts = []
            for _ in range(30):
                if sys.platform == 'linux': fd_counts.append(len(list(Path(f'/proc/{s.process.pid}/fd').iterdir())))
                s.event(b'Q'); s.restored(); s.reopen()
            assert not fd_counts or len(set(fd_counts)) == 1, fd_counts
            evidence['lifecycle'] = dict(repetitions=30, active_fd_counts=fd_counts, exact_termios=True, paste_disabled_on_every_close=True)
            s.edit(b'\x04', '', 0); s.until(lambda:not s.state()['open']); s.restored()
            evidence['eof'] = s.state()
            s.stop()
            (work/'reactor-stderr.log').write_bytes(s.receipts)
            (work/'reactor-output.bin').write_bytes(s.output)
            if prefix and sys.platform == 'darwin':
                assert b'0 leaks for 0 total leaked bytes' in s.receipts+s.output
        except BaseException:
            s.app.close(); s.listener.close(); s.abort(); raise
        return evidence


def trace_idle(work):
    if sys.platform != 'linux' or not shutil.which('strace'):
        return dict(status='unavailable', reason='Linux strace unavailable; virtual transport independently asserts zero calls')
    trace = work/'idle.strace'
    with tempfile.TemporaryDirectory(prefix='replai-idle-') as directory:
        s = Reactor(Path(directory), ['strace','-qq','-yy','-s','200','-e','trace=write,read,ioctl,poll,ppoll','-o',str(trace)])
        try:
            s.event(b'S'); time.sleep(0.4); s.event(b'S'); s.stop()
        except BaseException: s.abort(); raise
    log = trace.read_text()
    segments = log.split('"OBSERVER\\n"')
    assert len(segments) == 3, log
    interval = segments[1]
    assert 'TIOCGWINSZ' not in interval
    assert not re.search(r'read\([^\n]*</dev/pts/', interval)
    waits = [line for line in interval.splitlines() if line.startswith(('ppoll(', 'poll('))]
    assert len(waits) == 1 and 'NULL' in waits[0], waits
    return dict(status='measured', dimension_queries=0, terminal_reads=0, host_blocking_waits=1,
        replai_periodic_waits=0, scope='between observer notifications; one host reactor ppoll, no terminal/library work')


def process_cpu(pid):
    if sys.platform != 'linux': return None
    fields = Path(f'/proc/{pid}/stat').read_text().split()
    return (int(fields[13])+int(fields[14])) / os.sysconf('SC_CLK_TCK')


def simple():
    # Keep the PTY session leader alive after the example returns: restoration is
    # observed before hangup, on Darwin as well as Linux.
    wrapper = 'import os,subprocess,sys; r=subprocess.call(sys.argv[1:]); print("EXIT_READY",file=sys.stderr,flush=True); os.read(int(os.environ["P0_EXIT_FD"]),1); sys.exit(r)'
    s = Session([sys.executable, '-c', wrapper, ROOT/'target/debug/examples/simple'])
    try:
        s.until(lambda:b'simple> ' in s.output)
        s.send('e\u0301界\x1b[D!\r'.encode())
        s.until(lambda:'echo: e\u0301!界'.encode() in s.output and s.output.count(b'simple> ') >= 2)
        s.send(b'draft\x1b[A\x1b[B\r')
        s.until(lambda:b'echo: draft' in s.output and s.output.count(b'simple> ') >= 3)
        s.send(b'\x03')
        s.until(lambda:s.output.count(b'simple> ') >= 4)
        s.send(b'\x04')
        s.finish()
        return dict(submitted='e\u0301!界', history_draft='draft', interrupt_then_eof=True, exact_termios=True)
    except BaseException: s.abort(); raise


def admission():
    binary = ROOT/'target/debug/examples/simple'
    refused = {}
    for term in ['dumb', '']:
        master, slave = os.openpty()
        try:
            before = attributes(slave)
            result = subprocess.run([binary], stdin=slave, stdout=slave, stderr=subprocess.PIPE, env={**os.environ, 'TERM':term}, timeout=10)
            assert result.returncode != 0 and b'CapabilityMismatch' in result.stderr, result
            assert attributes(slave) == before
            refused[repr(term)] = result.stderr.decode().strip()
        finally: os.close(master); os.close(slave)
    result = subprocess.run([binary], input=b'', capture_output=True, env={**os.environ, 'TERM':'xterm'}, timeout=10)
    assert result.returncode != 0 and b'UnsuitableTerminal' in result.stderr
    return dict(environment_refusal=refused, non_tty='UnsuitableTerminal', before_raw=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--work', type=Path, required=True)
    parser.add_argument('--memory', action='store_true')
    parser.add_argument('--prefix', nargs=argparse.REMAINDER, default=[])
    a = parser.parse_args(); a.work.mkdir(parents=True, exist_ok=True)
    subprocess.run(['cargo','build','--locked','--examples'], cwd=ROOT, check=True)
    prefix = a.prefix
    if a.memory:
        if sys.platform == 'linux':
            executable = shutil.which('valgrind')
            assert executable, 'Valgrind is required for this gate'
            prefix = [executable, '--leak-check=full', '--show-leak-kinds=all', '--errors-for-leak-kinds=definite,indirect', '--error-exitcode=99', '--log-file='+str(a.work/'valgrind.log')]
        else:
            prefix = ['/usr/bin/leaks', '--atExit', '--']
    result = dict(platform=sys.platform, memory_command=prefix, driven=driven(a.work, prefix), simple=simple(), admission=admission(), idle_trace=trace_idle(a.work) if not a.memory else dict(status="separate normal run"))
    if a.memory and sys.platform == 'linux':
        assert 'ERROR SUMMARY: 0 errors' in (a.work/'valgrind.log').read_text()
    (a.work/'embedding.json').write_text(json.dumps(result, indent=2)+'\n')
    print(json.dumps(result, indent=2))


if __name__ == '__main__': main()
