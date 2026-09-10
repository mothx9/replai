#!/usr/bin/env python3
"""One Linux/macOS external-reactor oracle for revision-bound multiline validation."""
import argparse
import fcntl
import struct
import termios
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
from completion_pty import key as completion_key, revision, selection, CompletionReactor

def key(s, data):
    if data != b"\r": return completion_key(s,data)
    mark=len(s.receipts); before=selection(s); s.send(data)
    def received():
        tail=s.receipts[mark:]
        if re.search(rb"STATE (true|false) \d+ \d+ [0-9a-f]*\n",tail) is None: return False
        return selection(s) is None if before else b"SUBMISSION_REQUEST\n" in tail
    s.until(received)
    return s.state()



def run(work, prefix=(), plain=False):
    old=os.environ.get('NO_COLOR')
    if plain: os.environ['NO_COLOR']='1'
    else: os.environ.pop('NO_COLOR',None)
    try:
        with tempfile.TemporaryDirectory(prefix='replai-validate-') as directory:
            s=CompletionReactor(Path(directory),prefix,binary='validation-driven',prompt='validate')
            evidence={'plain':plain,'damage':[]}
            def measure(name,op):
                s.screen();before=len(s.output);start=time.perf_counter_ns();op();screen=s.screen()
                evidence['damage'].append(dict(operation=name,bytes=len(s.output)-before,elapsed_us=(time.perf_counter_ns()-start)/1000))
                return screen
            def enter():
                mark=len(s.receipts);key(s,b'\r'); assert b'SUBMISSION_REQUEST' in s.receipts[mark:]
            try:
                s.edit(b'begin {','begin {',7);enter();r=revision(s)
                measure('incomplete',lambda:s.event(b'I'))
                assert s.state()['text']=='begin {\n' and s.state()['open'] and revision(s)!=r
                assert '.. ' in s.screen()['text']
                s.edit(b'task\r','begin {\ntask',12)
                r=revision(s); screen=measure('invalid',lambda:s.event(b'J'))
                assert '! Invalid input' in screen['text'] and 'Expected closing delimiter' in screen['text']
                assert revision(s)==r
                for width in [20,40,80,132]:
                    s.operations.append(f'R 12 {width}');fcntl.ioctl(s.slave,termios.TIOCSWINSZ,struct.pack("HHHH",12,width,0,0))
                    screen=measure(f'resize_{width}',lambda:s.event(b'R'))
                    assert revision(s)==r and '! Invalid input' in screen['text']
                screen=measure('output',lambda:s.event(b'O'))
                assert 'application event: draft preserved' in screen['text'] and '! Invalid input' in screen['text']
                s.edit(b'!', 'begin {\ntask!',13);assert 'Invalid input' not in s.screen()['text']
                # Delayed decisions from the earlier request are silent in all three dispositions.
                for command in [b'C',b'I',b'J']:
                    before=s.screen();s.event(command);assert before==s.screen();assert b'VALIDATION Stale' in s.receipts
                    assert s.state()['open'] and s.state()['text']=='begin {\ntask!'
                enter();s.event(b'I');s.edit(b'}','begin {\ntask!\n}',15);enter()
                screen=measure('complete',lambda:s.event(b'C'));assert not s.state()['open'];s.restored();s.reopen()
                evidence['validated_text']='begin {\ntask!\n}'
                # Recalled multiline entry is vertically editable; history returns the exact draft.
                s.edit(b'draft','draft',5);key(s,b'\x1b[A');assert s.state()['text']==evidence['validated_text']
                key(s,b'\x1b[A');assert s.state()['cursor']<15
                key(s,b'\x1b[B');key(s,b'\x1b[B');assert s.state()['text']=='draft' and s.state()['cursor']==5
                s.event(b'X');s.restored();s.reopen()
                # Completion Enter accepts only; a later Enter requests validation.
                s.edit(b'bu','bu',2);s.event(b'F');mark=len(s.receipts);key(s,b'\r')
                assert s.state()['text']=='build ' and b'SUBMISSION_REQUEST' not in s.receipts[mark:]
                enter();s.event(b'C');s.restored();s.reopen()
                text='e\u0301界👩‍💻{\n}'
                s.edit(b'\x1b[200~'+text.encode()+b'\x1b[201~',text,len(text.encode()))
                enter();s.event(b'J');assert 'Invalid input' in s.screen()['text']
                key(s,b'\x1b[D');assert 'Invalid input' not in s.screen()['text']
                enter();s.event(b'C');s.restored();s.reopen()
                evidence['unicode_validation']=True
                # Atomic multiline paste; substantial drafts at beginning/middle/end, bounded viewport.
                evidence['large']=[]
                for lines in [10,100,1000]:
                    text=('row '+('x'*64)+'\n')*lines
                    s.edit(b'\x1b[200~'+text.encode()+b'\x1b[201~',text,len(text))
                    enter();s.event(b'J');r=revision(s)
                    assert s.state()['text']==text
                    measure(f'large_{lines}_output',lambda:s.event(b'O'));assert revision(s)==r
                    key(s,b'\x1b[A');assert s.state()['cursor']<len(text);assert 'Invalid input' not in s.screen()['text']
                    for movement in [b'\x01',b'\x05',b'\x1b[A']:
                        measure(f'large_{lines}_movement',lambda: key(s,movement))
                        assert s.state()['text']==text
                    key(s,b'\x01')
                    s.edit(b'\x1b[B'*(lines//2),text,69*(lines//2))
                    enter();s.event(b'J');measure(f'large_{lines}_middle_output',lambda:s.event(b'O'))
                    evidence['large'].append(dict(lines=lines,bytes=len(text),screen=s.screen()))
                    s.event(b'X');s.restored();s.reopen()
                # Repeated drop/close while invalid; cleanup checked before process exit.
                fds=[]
                for _ in range(12):
                    s.edit(b'x','x',1);enter();s.event(b'J');s.event(b'Q');s.restored();s.reopen();fds.append(s.state()['observed_fds'])
                assert len(set(fds))==1
                evidence.update(lifecycles=12,exact_restoration=True,stale_silent=True,completion_precedence=True)
                key(s,b'\x04');s.restored();s.reopen();s.stop()
                if plain: assert all(v in (b'',b'0') for v in re.findall(rb'\x1b\[([0-9;]*)m',s.output))
                if prefix and sys.platform=='darwin': assert b'0 leaks for 0 total leaked bytes' in s.receipts+s.output
                path=work/('plain' if plain else 'styled');path.mkdir(exist_ok=True)
                (path/'observer.txt').write_bytes(s.receipts);(path/'terminal.bin').write_bytes(s.output)
                return evidence
            except BaseException:
                print(s.receipts[-4000:].decode(errors='replace'),file=sys.stderr);print(s.screen(),file=sys.stderr);s.abort();raise
    finally:
        if old is None:os.environ.pop('NO_COLOR',None)
        else:os.environ['NO_COLOR']=old


def session():
    wrapper='import os,subprocess,sys; r=subprocess.call(sys.argv[1:]); print("EXIT_READY",file=sys.stderr,flush=True); os.read(int(os.environ["P0_EXIT_FD"]),1); sys.exit(r)'
    s=Session([sys.executable,'-c',wrapper,ROOT/'target/debug/examples/validation'])
    try:
        s.until(lambda:b'validate> ' in s.output)
        s.send(b'{\r');s.until(lambda:b'.. ' in s.output)
        s.send(b'task\r}\r');s.until(lambda:b'validate> ' in s.output.partition(b'host received 8 bytes:')[2])
        s.send(b'\x04');s.finish()
        return dict(synchronous_validation=True,submitted='{\ntask\n}',exact_restoration=True)
    except BaseException:s.abort();raise


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--work',type=Path,required=True);p.add_argument('--memory',action='store_true');a=p.parse_args();a.work.mkdir(parents=True,exist_ok=True)
    subprocess.run(['cargo','build','--locked','--example','validation','--example','validation-driven','--example','terminal-state'],cwd=ROOT,check=True)
    prefix=[]
    if a.memory:
        if sys.platform=='linux':
            executable=shutil.which('valgrind');assert executable,'Valgrind required'
            prefix=[executable,'--leak-check=full','--show-leak-kinds=all','--errors-for-leak-kinds=definite,indirect','--error-exitcode=99','--log-file='+str(a.work/'valgrind-%p.log')]
        else:prefix=['/usr/bin/leaks','--atExit','--']
    result=dict(platform=sys.platform,memory_command=prefix,styled=run(a.work,prefix),plain=run(a.work,prefix,True),session=session())
    if a.memory and sys.platform=='linux':
        logs=list(a.work.glob('valgrind-*.log'));assert len(logs)==2
        for log in logs:assert 'ERROR SUMMARY: 0 errors' in log.read_text()
    assert [v['screen'] for v in result['styled']['large']]==[v['screen'] for v in result['plain']['large']]
    (a.work/'validation.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
if __name__=='__main__':main()
