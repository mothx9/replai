#!/usr/bin/env python3
"""Production poll-call counts over real PTYs; these are NOT syscall counts."""
import argparse
import json
from pathlib import Path

from linux import HOST, Session


def measure(work, repeats=3):
    workloads = [
        ('short_ascii', b'command', 'command'),
        ('unicode', 'café 界🌍'.encode(), 'café 界🌍'),
        ('long_ascii', b'a' * 1000 + b'\x1b[DX', 'a' * 999 + 'Xa'),
    ]
    for size in (65536, 1048576):
        unit = 'source line\n'
        body = unit * (size // len(unit)) + 'x' * (size % len(unit))
        workloads.append((f'paste_multiline_{size}', b'\x1b[200~' + body.encode() + b'\x1b[201~', body))
    for name, wire, expected in workloads:
        calls, output = [], []
        for _ in range(repeats):
            case = work / 'driver-case.json'
            case.write_text(json.dumps(dict(initial='')))
            session = Session([HOST, '--case-file', case])
            try:
                session.ready()
                session.send(wire + b'\r')
                session.until(lambda: b'"submitted"' in session.receipts, timeout=180)
                session.finish()
                receipt = next(json.loads(line) for line in session.receipts.splitlines()
                               if line.startswith(b'{') and b'"submitted"' in line)
                assert receipt['submitted'] == expected
                calls.append(receipt['poll_calls'])
                output.append(len(session.output))
            except BaseException:
                session.abort()
                raise
        yield dict(schema_version=1, id='driver/' + name, component='driver',
                   operation=name, status='measured', iterations=repeats,
                   input_bytes=len(wire) + 1, columns=80, rows=24,
                   counters=dict(poll_calls=calls, observed_terminal_bytes=output,
                                 submission_exact=True, terminal_restored=True),
                   scope='READY through exact submission and full PTY drain; poll calls include submission, not OS reads or writes')


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--work', required=True, type=Path)
    parser.add_argument('--repeats', type=int, default=3)
    args = parser.parse_args()
    args.work.mkdir(parents=True, exist_ok=True)
    for result in measure(args.work, args.repeats):
        print(json.dumps(result), flush=True)
