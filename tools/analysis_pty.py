#!/usr/bin/env python3
"""One Linux/macOS external-reactor oracle for retained host analysis."""
import argparse
import json
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import time
from embedding_pty import Reactor, ROOT, winsize


def run(work):
    with tempfile.TemporaryDirectory(prefix='replai-analysis-') as directory:
        s = Reactor(Path(directory), binary='analysis', prompt='analysis')
        def revision():
            return re.findall(rb'REVISION (.+)\n', s.receipts)[-1].decode()
        def stale():
            before = s.state(); screen = s.screen(); rev = revision(); mark = len(s.receipts); output_bytes = len(s.output)
            s.event(b'A')
            assert b'ANALYSIS Stale\n' in s.receipts[mark:]
            assert s.state() == before and revision() == rev
            assert s.screen() == screen, 'stale analysis altered semantic screen'
            assert len(s.output) == output_bytes, 'stale application emitted terminal bytes'
        evidence = {}
        try:
            s.edit('run 界e\u0301👩‍💻'.encode(), 'run 界e\u0301👩‍💻', len('run 界e\u0301👩‍💻'.encode()))
            s.event(b'T'); old = revision()
            # The host reactor continues serving terminal readiness while a
            # synthetic host analysis remains outside REPLAI. The socket later
            # delivers its completion, independently of terminal input.
            time.sleep(0.03)
            s.edit(b'\x1b[D!', 'run 界e\u0301!👩‍💻', len('run 界e\u0301!'.encode()))
            assert revision() != old
            stale(); evidence['delayed_stale'] = s.state()
            s.event(b'F'); assert s.state()['text'] == 'ready'
            assert b'ANALYSIS Applied\n' in s.receipts
            evidence['fresh_applied'] = s.state()
            s.event(b'T'); old = revision()
            s.event(b'O'); assert revision() == old
            s.screen(); s.operations.append('R 24 20'); winsize(s.slave, 20)
            s.event(b'R'); assert revision() == old
            s.event(b'V'); assert revision() == old
            s.event(b'A'); assert s.state()['text'] == 'obsolete'
            evidence['output_resize_reopen_current'] = s.state()
            # Submission invalidates even though the retained bytes are unchanged.
            s.event(b'T'); old = revision()
            s.edit(b'\r', 'obsolete', 8); s.until(lambda:not s.state()['open'])
            s.restored(); assert revision() != old
            s.reopen(); s.edit(b'obsolete', 'obsolete', 8); stale()
            s.event(b'T'); old = revision()
            s.edit(b'\x1b[D', 'obsolete', 7); assert revision() != old; stale()
            s.event(b'X'); s.reopen()
            s.edit(b'draft\x1b[D', 'draft', 4); s.event(b'T'); old = revision()
            s.edit(b'\x1b[A', 'obsolete', 8)
            s.edit(b'\x1b[B', 'draft', 4)
            assert revision() != old; stale(); evidence['history_return_stale'] = s.state()
            s.event(b'X'); s.reopen()
            old = revision()
            s.edit('\x1b[200~a\r\n界\x1b[201~'.encode(), 'a\n界', 5)
            assert revision() != old
            s.event(b'T'); old = revision()
            mark = len(s.receipts); s.send(b'\x1b[200~bad\x03\x1b[201~')
            s.until(lambda:b'REJECTED ' in s.receipts[mark:] and b'STATE ' in s.receipts[mark:])
            assert revision() == old and s.state()['text'] == 'a\n界'
            s.event(b'X'); s.reopen()
            s.edit(b'he\t', 'hello', 5); evidence['completion'] = s.state()
            s.event(b'X'); s.reopen()
            s.edit(b'\x04', '', 0); s.until(lambda:not s.state()['open']); s.restored()
            evidence['eof'] = s.state(); s.reopen()
            fds = []
            for _ in range(20):
                s.edit(b'x', 'x', 1); s.event(b'T'); s.edit(b'y', 'xy', 2); stale()
                s.event(b'X'); s.restored(); s.reopen(); fds.append(s.state()['observed_fds'])
            assert len(set(fds)) == 1
            evidence['repeated_lifecycles'] = len(fds)
            s.stop(); evidence['restoration'] = True
        except BaseException:
            print(s.receipts[-4000:].decode(errors='replace'), file=sys.stderr)
            s.abort(); raise
        (work/'observer.txt').write_bytes(s.receipts)
        (work/'terminal.bin').write_bytes(s.output)
        return evidence


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__); p.add_argument('--work', type=Path, required=True)
    a = p.parse_args(); a.work.mkdir(parents=True, exist_ok=True)
    subprocess.run(['cargo','build','--locked','--example','analysis','--example','terminal-state'], cwd=ROOT, check=True)
    result = dict(platform=sys.platform, oracle=run(a.work))
    (a.work/'analysis.json').write_text(json.dumps(result, indent=2)+'\n')
    print(json.dumps(result, indent=2))
