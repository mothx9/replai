#!/usr/bin/env python3
"""Real Linux/macOS I2 oracle over the shared external-reactor host."""
import argparse, json, os, re, shutil, subprocess, sys, tempfile, time
from pathlib import Path
from embedding_pty import ROOT, Session, winsize
from completion_pty import CompletionReactor, key, revision
from validation_pty import key as validation_key

def run(work,prefix=(),plain=False):
    old=os.environ.get('NO_COLOR')
    if plain:os.environ['NO_COLOR']='1'
    else:os.environ.pop('NO_COLOR',None)
    try:
        with tempfile.TemporaryDirectory(prefix='replai-i2-') as directory:
            s=CompletionReactor(Path(directory),prefix,binary='validation-driven',prompt='validate',styled=not plain)
            evidence={'plain':plain,'damage':[],'large':[]}
            def measure(name,action):
                s.screen();mark=len(s.output);start=time.perf_counter_ns();action();screen=s.screen()
                evidence['damage'].append(dict(operation=name,bytes=len(s.output)-mark,observer_us=(time.perf_counter_ns()-start)/1000))
                return screen
            def present():s.event(b'H');return measure('install',lambda:s.event(b'P'))
            try:
                s.edit(b'bu','bu',2);r=revision(s);screen=present();assert '[~ild' in screen['text'];assert revision(s)==r
                # Same-revision replacement; styling and noncanonical hint don't edit.
                measure('replace',lambda:s.event(b'P'));assert s.state()['text']=='bu' and s.state()['cursor']==2
                # Inspect terminal color through the same VT observer, not byte equality alone.
                cells=subprocess.run([ROOT/'target/debug/examples/terminal-state'],input='\n'.join(s.operations)+'\n',text=True,capture_output=True,check=True).stdout
                expected='Default' if plain else 'Idx(81)'
                assert f'cell 0 10 62 {expected} Default false' in cells
                evidence['styled_cell_verified']=not plain
                for width in [20,40,80,132]:
                    s.operations.append(f'R 24 {width}');winsize(s.slave,width)
                    screen=measure(f'resize_{width}',lambda:s.event(b'R'));assert '[~' in screen['text'];assert revision(s)==r
                screen=measure('output',lambda:s.event(b'O'));assert '[~ild' in screen['text'] and 'application event' in screen['text'];assert revision(s)==r
                # One stored parse feeds I1 and I2, no library callback/reparse.
                screen=measure('completion',lambda:s.event(b'K'));assert '[~ild' not in screen['text'] and '> build' in screen['text']
                screen=measure('dismiss',lambda:s.event(b'D'));assert '[~ild' in screen['text'];assert revision(s)==r
                validation_key(s,b'\r');s.event(b'J');assert '[~ild' in s.screen()['text'] and 'Invalid input' in s.screen()['text']
                # Edit after request; delayed old presentation must not disturb current display.
                s.edit(b'!','bu!',3);assert '[~' not in s.screen()['text'];before=s.screen();mark=len(s.output)
                s.event(b'P');assert b'PRESENTATION Stale' in s.receipts and s.screen()==before and len(s.output)==mark
                screen=present();assert '[~' in screen['text'];fresh=s.state()
                key(s,b'\x1b[D');assert '[~' not in s.screen()['text'];before=s.screen();mark=len(s.output);s.event(b'P');assert s.screen()==before and len(s.output)==mark
                key(s,b'\x1b[C');assert s.state()['text']==fresh['text'];before=s.screen();mark=len(s.output);s.event(b'P');assert len(s.output)==mark
                # New current result clears explicitly without touching draft/revision.
                present();r=revision(s);measure('clear',lambda:s.event(b'G'));assert '[~' not in s.screen()['text'] and revision(s)==r
                s.event(b'X');s.restored();s.reopen()
                s.edit(b'bu','bu',2);present();validation_key(s,b'\r');s.event(b'B');assert not s.state()['open'];assert b'host' not in s.state()['text'].encode();assert s.state()['text']=='bu';s.restored();s.reopen()
                # Hint cannot be submitted. History/paste preserve canonical bytes only.
                s.edit(b'x','x',1);present();s.edit(b'\x1b[A','bu',2);assert re.findall(rb'DISPLAY (true|false)\n',s.receipts)[-1]==b'false';s.event(b'X');s.reopen()
                s.edit(b'bu','bu',2);present();r=revision(s);s.event(b'K');key(s,b'\t');key(s,b'\r')
                assert s.state()['text']=='bundle' and revision(s)!=r and re.findall(rb'DISPLAY (true|false)\n',s.receipts)[-1]==b'false'
                present();pasted='界e\u0301👩‍💻';s.edit(b'\x1b[200~'+pasted.encode()+b'\x1b[201~','bundle'+pasted,len(('bundle'+pasted).encode()))
                assert re.findall(rb'DISPLAY (true|false)\n',s.receipts)[-1]==b'false';s.event(b'X');s.restored();s.reopen()
                for lines in [10,100,1000]:
                    text=('row 界 e\u0301 👩‍💻 '+'x'*40+'\n')*lines
                    s.edit(b'\x1b[200~'+text.encode()+b'\x1b[201~',text,len(text.encode()));r=revision(s);present()
                    for width in [20,40,80,132]:
                        s.operations.append(f'R 24 {width}');winsize(s.slave,width);measure(f'large_{lines}_resize_{width}',lambda:s.event(b'R'));assert revision(s)==r
                    measure(f'large_{lines}_output',lambda:s.event(b'O'));assert s.state()['text']==text
                    key(s,b'\x01');present();assert '[~' not in s.screen()['text']
                    validation_key(s,b'\r');s.event(b'I');assert re.findall(rb'DISPLAY (true|false)\n',s.receipts)[-1]==b'false'
                    evidence['large'].append(dict(lines=lines,bytes=len(text.encode()),screen=s.screen()))
                    s.event(b'X');s.restored();s.reopen()
                fds=[]
                for _ in range(12):
                    s.edit(b'bu','bu',2);present();s.event(b'Q');s.restored();s.reopen();fds.append(s.state()['observed_fds'])
                assert len(set(fds))==1
                key(s,b'\x04');s.restored();s.reopen();s.stop()
                if plain:assert all(v in (b'',b'0') for v in re.findall(rb'\x1b\[([0-9;]*)m',s.output))
                if prefix and sys.platform=='darwin':assert b'0 leaks for 0 total leaked bytes' in s.receipts+s.output
                path=work/('plain' if plain else 'styled');path.mkdir(exist_ok=True);(path/'terminal.bin').write_bytes(s.output);(path/'observer.txt').write_bytes(s.receipts)
                evidence.update(delayed_stale_zero_bytes=True,canonical_submission='bu',lifecycle_restoration=12,shared_snapshot_products=True)
                return evidence
            except BaseException:
                print(s.receipts[-3000:].decode(errors='replace'),file=sys.stderr);print(s.screen(),file=sys.stderr);s.abort();raise
    finally:
        if old is None:os.environ.pop('NO_COLOR',None)
        else:os.environ['NO_COLOR']=old

def session(delayed=False):
    wrapper='import os,subprocess,sys; r=subprocess.call(sys.argv[1:]); print("EXIT_READY",file=sys.stderr,flush=True); os.read(int(os.environ["P0_EXIT_FD"]),1); sys.exit(r)'
    s=Session([sys.executable,'-c',wrapper,ROOT/'target/debug/examples/analysis-presentation',*(['--delayed-analysis'] if delayed else [])])
    try:
        s.until(lambda:b'analyze>' in s.output);s.send(b'bu')
        if delayed:
            s.until(lambda:b'ANALYSIS_PENDING' in s.receipts);s.send(b'!');s.until(lambda:b'DELAYED_PRESENTATION Stale' in s.receipts)
        else:s.until(lambda:b'[~ild' in s.output)
        s.send(b'\r');submitted=b'host received: '+(b'bu!' if delayed else b'bu')+b'\r\n'
        s.until(lambda:b'analyze>' in s.output.partition(submitted)[2]);s.send(b'\x04');s.finish()
        return dict(synchronous=True,delayed_stale=delayed,canonical_submission='bu!' if delayed else 'bu',exact_restoration=True)
    except BaseException:s.abort();raise

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--work',type=Path,required=True);p.add_argument('--memory',action='store_true');a=p.parse_args();a.work.mkdir(parents=True,exist_ok=True)
    subprocess.run(['cargo','build','--locked','--example','analysis-presentation','--example','validation-driven','--example','terminal-state'],cwd=ROOT,check=True)
    prefix=[]
    if a.memory:
        if sys.platform=='linux':
            exe=shutil.which('valgrind');assert exe,'Valgrind required';prefix=[exe,'--leak-check=full','--show-leak-kinds=all','--errors-for-leak-kinds=definite,indirect','--error-exitcode=99','--log-file='+str(a.work/'valgrind-%p.log')]
        else:prefix=['/usr/bin/leaks','--atExit','--']
    result=dict(platform=sys.platform,memory_command=prefix,styled=run(a.work,prefix),plain=run(a.work,prefix,True),session=session(),delayed_session=session(True))
    if a.memory and sys.platform=='linux':
        logs=list(a.work.glob('valgrind-*.log'));assert len(logs)==2
        for log in logs:assert 'ERROR SUMMARY: 0 errors' in log.read_text()
    assert [x['screen'] for x in result['styled']['large']]==[x['screen'] for x in result['plain']['large']]
    (a.work/'analysis-presentation.json').write_text(json.dumps(result,indent=2)+'\n');print('PASS analysis presentation: session, driven, styled/plain, stale, Unicode, 10/100/1000 lines, restoration')
if __name__=='__main__':main()
