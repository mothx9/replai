#!/usr/bin/env python3
"""Prepare ordinary -O2 C size consumers using qualified, isolated staged artifacts."""
import argparse
from pathlib import Path
import shlex
import sys
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from qualify_c import Qualification
from c_pty import run_suite

p=argparse.ArgumentParser(description=__doc__);p.add_argument('--qualification',type=Path,required=True);a=p.parse_args()
q=Qualification(a.qualification)
for mode in ['static','shared']:
    pkg=['pkg-config','--cflags','--libs',*(['--static'] if mode=='static' else []),'replai']
    flags=shlex.split(q.run(pkg,'size-pkg-'+mode))
    if mode=='static':flags=[str(q.prefix/'lib/libreplai_c.a') if x=='-lreplai_c' else x for x in flags]
    else:flags.append('-Wl,-rpath,'+str(q.prefix/'lib'))
    name='demo-'+mode+'-release'
    q.run(['cc','-O2','-std=c11','-Wall','-Wextra','-Werror','demo.c',*flags,'-o',name],'size-build-'+mode)
    run_suite([str(q.consumer/name)],q.consumer/'rust-terminal-state',q.consumer/'presentation.tsv',q.root/('pty-size-'+mode+'.json'),q.isolate,q.env)
print('Release C size consumers: isolated C01-C15 PASS')
