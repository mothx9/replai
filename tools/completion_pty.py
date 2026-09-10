#!/usr/bin/env python3
"""Native Linux/macOS oracle: host-delivered candidates and one terminal renderer."""
import argparse
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile
import time
from embedding_pty import Reactor, Session, ROOT, winsize


def selection(s):
    value = re.findall(rb'SELECTION (NONE|\d+ \d+)\n', s.receipts)[-1]
    return None if value == b'NONE' else tuple(map(int, value.split()))


def revision(s):
    return re.findall(rb'REVISION (.+)\n', s.receipts)[-1].decode()


class CompletionReactor(Reactor):
    def event(self, data):
        mark = len(self.receipts)
        self.app.sendall(data)
        token = b'APPLICATION ' + str(data[0]).encode() + b'\n'
        self.until(lambda: token in self.receipts[mark:] and re.search(rb'STATE (true|false) \d+ \d+ [0-9a-f]*\n', self.receipts[mark:].partition(token)[2]) is not None)
        return self.state()


def key(s, data):
    mark = len(s.receipts); before = selection(s); old = revision(s)
    s.send(data)
    def complete():
        if re.search(rb'STATE (true|false) \d+ \d+ [0-9a-f]*\n', s.receipts[mark:]) is None: return False
        if data == b'\t': return selection(s) == ((before[0]+1)%before[1],before[1]) if before else b'COMPLETION\n' in s.receipts[mark:]
        if data == b'\x1b[Z': return selection(s) == ((before[0]-1)%before[1],before[1])
        if data == b'\x1b': return selection(s) is None
        if data == b'\r': return selection(s) is None if before else not s.state()['open']
        if data in (b'\x03', b'\x04'): return not s.state()['open']
        if b'bad\x03' in data: return b'REJECTED ' in s.receipts[mark:]
        return revision(s) != old
    s.until(complete)
    return s.state()


def run(work, prefix=(), plain=False):
    old = os.environ.get('NO_COLOR')
    if plain: os.environ['NO_COLOR'] = '1'
    else: os.environ.pop('NO_COLOR', None)
    try:
        with tempfile.TemporaryDirectory(prefix='replai-completion-') as directory:
            s = CompletionReactor(Path(directory), prefix, binary='completion-driven', prompt='complete')
            evidence = {}; damage = {}
            def measure(name, action):
                s.screen(); mark=len(s.output); started=time.perf_counter_ns()
                action(); screen=s.screen()
                damage[name]=dict(bytes=len(s.output)-mark, observer_elapsed_ns=time.perf_counter_ns()-started)
                return screen
            try:
                s.edit('bu界e\u0301👩‍💻'.encode(), 'bu界e\u0301👩‍💻', len('bu界e\u0301👩‍💻'.encode()))
                key(s,b'\t'); assert selection(s) is None
                old_revision=revision(s)
                # The host waits on the same terminal and independent result socket.
                time.sleep(.03)
                s.edit('\x1b[D!'.encode(), 'bu界e\u0301!👩‍💻', len('bu界e\u0301!'.encode())); assert revision(s)!=old_revision
                before=s.state(); screen=s.screen(); mark=len(s.output)
                s.event(b'A'); assert b'CANDIDATES Stale\n' in s.receipts
                assert s.state()==before and s.screen()==screen and len(s.output)==mark
                assert selection(s) is None
                evidence['delayed_stale_zero_bytes']=True
                screen=measure('show',lambda:s.event(b'F'))
                assert selection(s)==(0,3) and '> build' in screen['text']
                current_menu=selection(s); current_screen=s.screen(); mark=len(s.output)
                s.event(b'A')  # An older result must not replace a newer visible menu.
                assert selection(s)==current_menu and s.screen()==current_screen and len(s.output)==mark
                current=revision(s); draft=(s.state()['text'],s.state()['cursor'])
                screen=measure('next',lambda:key(s,b'\t'))
                assert selection(s)==(1,3) and '> bundle' in screen['text']
                measure('previous',lambda:key(s,b'\x1b[Z'))
                assert selection(s)==(0,3) and revision(s)==current
                for width in [20,40,80,132]:
                    s.operations.append(f'R 24 {width}'); winsize(s.slave,width)
                    screen=measure(f'resize_{width}',lambda:s.event(b'R'))
                    assert selection(s)==(0,3) and revision(s)==current
                    assert '> build' in screen['text']
                    assert ('Build the project' in screen['text']) == (width>=40)
                screen=measure('output',lambda:s.event(b'O'))
                assert 'application event: draft preserved' in screen['text'] and '> build' in screen['text']
                assert selection(s)==(0,3) and revision(s)==current
                assert (s.state()['text'],s.state()['cursor'])==draft
                screen=measure('accept',lambda:key(s,b'\r'))
                assert selection(s) is None and s.state()['text']=='build ' and s.state()['open']
                assert revision(s)!=current and 'bundle' not in screen['text']
                evidence['fresh_accept_after_resize_output']=True
                s.event(b'1'); current=revision(s)
                assert selection(s)==(0,1) and s.state()['text']=='build '
                measure('dismiss',lambda:key(s,b'\x1b'))
                assert selection(s) is None and revision(s)==current
                s.event(b'F'); s.event(b'Z'); assert selection(s) is None
                s.event(b'L'); assert selection(s)==(0,1000)
                key(s,b'\x1b[Z'); assert selection(s)==(999,1000)
                screen=s.screen(); assert '1000 / 1000' in screen['text']
                evidence['bounded_1000_viewport']=screen
                s.event(b'D'); key(s,b'\r'); s.until(lambda:not s.state()['open']); s.restored(); s.reopen()
                s.edit(b'draft','draft',5); s.event(b'F'); key(s,b'\x1b[A')
                assert selection(s) is None and s.state()['text']=='build '
                key(s,b'\x1b[B'); assert s.state()['text']=='draft'
                s.event(b'F'); key(s,b'\x1b[D'); assert selection(s) is None
                s.event(b'F'); key(s,b'\x7f'); assert selection(s) is None
                s.event(b'F'); key(s,'\x1b[200~a\t界\r\n\x1b[201~'.encode()); assert selection(s) is None
                s.event(b'F'); current=revision(s)
                key(s,b'\x1b[200~bad\x03\x1b[201~'); assert selection(s)==(0,3) and revision(s)==current
                # Host output on rejected input must restore the valid menu too.
                assert '> build' in s.screen()['text']
                s.event(b'X'); s.restored(); s.reopen()
                fds=[]
                for _ in range(12):
                    s.edit(b'bu','bu',2);s.event(b'F');s.event(b'Q');s.restored();s.reopen();fds.append(s.state()['observed_fds'])
                assert len(set(fds))==1
                evidence['restored_lifecycles']=len(fds)
                s.event(b'F'); key(s,b'\x03');s.restored();s.reopen();key(s,b'\x04');s.restored();s.reopen()
                s.stop()
                if plain:
                    assert all(value in (b'',b'0') for value in re.findall(rb'\x1b\[([0-9;]*)m',s.output)), 'NO_COLOR emitted styling'
                if prefix and sys.platform=='darwin': assert b'0 leaks for 0 total leaked bytes' in s.receipts+s.output
                evidence.update(plain=plain,damage=damage,exact_restoration=True)
                (work/('plain' if plain else 'styled')).mkdir(exist_ok=True)
                (work/('plain' if plain else 'styled')/'observer.txt').write_bytes(s.receipts)
                (work/('plain' if plain else 'styled')/'terminal.bin').write_bytes(s.output)
                return evidence
            except BaseException:
                print(s.receipts[-5000:].decode(errors='replace'),file=sys.stderr)
                print(s.screen(),file=sys.stderr)
                s.abort(); raise
    finally:
        if old is None: os.environ.pop('NO_COLOR',None)
        else: os.environ['NO_COLOR']=old


def session_reopened(output):
    return b'demo> ' in output.partition(b'host received: bundle')[2]


def session():
    wrapper='import os,subprocess,sys; r=subprocess.call(sys.argv[1:]); print("EXIT_READY",file=sys.stderr,flush=True); os.read(int(os.environ["P0_EXIT_FD"]),1); sys.exit(r)'
    s=Session([sys.executable,'-c',wrapper,ROOT/'target/debug/examples/completion'])
    try:
        s.until(lambda:b'demo> ' in s.output)
        s.send(b'bu\t');s.until(lambda:b'Build the project' in s.output)
        s.send(b'\t\r\r');s.until(lambda:session_reopened(s.output))
        s.send(b'\x04');s.finish()
        return dict(synchronous_candidates=True,accepted='bundle',exact_restoration=True)
    except BaseException: s.abort();raise


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--work',type=Path,required=True);p.add_argument('--memory',action='store_true');a=p.parse_args();a.work.mkdir(parents=True,exist_ok=True)
    subprocess.run(['cargo','build','--locked','--example','completion','--example','completion-driven','--example','terminal-state'],cwd=ROOT,check=True)
    prefix=[]
    if a.memory:
        if sys.platform=='linux':
            executable=shutil.which('valgrind');assert executable,'Valgrind required'
            prefix=[executable,'--leak-check=full','--show-leak-kinds=all','--errors-for-leak-kinds=definite,indirect','--error-exitcode=99','--log-file='+str(a.work/'valgrind-%p.log')]
        else: prefix=['/usr/bin/leaks','--atExit','--']
    result=dict(platform=sys.platform,memory_command=prefix,styled=run(a.work,prefix),plain=run(a.work,prefix,True),session=session())
    assert result['styled']['bounded_1000_viewport'] == result['plain']['bounded_1000_viewport'], 'color policy changed semantic screen'
    if a.memory and sys.platform=='linux':
        logs=list(a.work.glob('valgrind-*.log'));assert len(logs)==2
        for log in logs: assert 'ERROR SUMMARY: 0 errors' in log.read_text()
    (a.work/'completion.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps(result,indent=2))

if __name__=='__main__':main()
