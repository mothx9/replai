#!/usr/bin/env python3
"""Qualify installed C artifacts; each phase is also an explicit CI gate."""
import argparse
import ctypes
import json
import os
from pathlib import Path
import shlex
import shutil
import subprocess
import tempfile
import tomllib
import sys

from c_pty import run_suite

ROOT = Path(__file__).resolve().parents[1]


def restrict_reads(root, extra=()):
    """Linux Landlock: consumers cannot read the repository or Cargo sources."""
    if sys.platform == 'darwin':
        # SBPL is generated at runtime; no machine/SDK paths enter the repository.
        denied = [str(ROOT), str(Path.home()/'.cargo')]
        profile = '(version 1)(allow default)(deny file-read* ' + ' '.join('(subpath '+json.dumps(p)+')' for p in denied) + ')'
        def restrict_darwin():
            sandbox = ctypes.CDLL('/usr/lib/libsandbox.dylib', use_errno=True)
            sandbox.sandbox_init.argtypes = [ctypes.c_char_p, ctypes.c_uint64, ctypes.POINTER(ctypes.c_char_p)]
            sandbox.sandbox_init.restype = ctypes.c_int
            error = ctypes.c_char_p()
            if sandbox.sandbox_init(profile.encode(), 0, ctypes.byref(error)):
                raise RuntimeError('Darwin consumer isolation failed: '+str(error.value))
            try:
                with open(ROOT/'Cargo.toml','rb'): pass
            except PermissionError: return
            raise AssertionError('consumer unexpectedly has access to repository')
        return restrict_darwin
    def restrict():
        libc = ctypes.CDLL(None, use_errno=True)
        class Ruleset(ctypes.Structure):
            _fields_ = [('access', ctypes.c_uint64)]
        class PathRule(ctypes.Structure):
            _pack_ = 1
            _fields_ = [('access', ctypes.c_uint64), ('parent', ctypes.c_int)]
        access = 1 | 4 | 8  # EXECUTE, READ_FILE, READ_DIR; writes are unrestricted.
        rule = Ruleset(access)
        fd = libc.syscall(444, ctypes.byref(rule), ctypes.sizeof(rule), 0)
        if fd < 0:
            raise OSError(ctypes.get_errno(), 'Landlock ruleset required for isolation')
        try:
            for path in ['/usr', '/etc', '/proc', '/dev', str(root), *extra]:
                parent = os.open(path, os.O_PATH | os.O_CLOEXEC)
                try:
                    item = PathRule(access, parent)
                    if libc.syscall(445, fd, 1, ctypes.byref(item), 0) != 0:
                        raise OSError(ctypes.get_errno(), 'Landlock path rule')
                finally:
                    os.close(parent)
            if libc.prctl(38, 1, 0, 0, 0) != 0 or libc.syscall(446, fd, 0) != 0:
                raise OSError(ctypes.get_errno(), 'Landlock enforcement')
        finally:
            os.close(fd)
        try:
            with open(ROOT / 'Cargo.toml', 'rb'):
                pass
        except PermissionError:
            return
        raise AssertionError('consumer unexpectedly has access to repository')
    return restrict


class Qualification:
    def __init__(self, root):
        self.root = root.resolve()
        self.prefix_a = self.root / 'prefix-a'
        self.prefix = self.root / 'prefix-b'
        self.consumer = self.root / 'consumer'
        self.env = os.environ.copy()
        for key in list(self.env):
            if key.startswith('CARGO') or key in ('LD_LIBRARY_PATH', 'DYLD_LIBRARY_PATH', 'DYLD_FALLBACK_LIBRARY_PATH', 'LIBRARY_PATH', 'CPATH', 'C_INCLUDE_PATH', 'CPLUS_INCLUDE_PATH', 'SDKROOT'):
                self.env.pop(key)
        self.env.update(PATH='/usr/bin:/bin:/opt/homebrew/bin' if sys.platform=='darwin' else '/usr/bin:/bin', TMPDIR=str(self.root / 'tmp'), PKG_CONFIG_PATH=str(self.prefix / 'lib/pkgconfig'), PKG_CONFIG_LIBDIR=str(self.prefix / 'lib/pkgconfig'))
        self.isolate = restrict_reads(self.root)

    def run(self, command, name, *, isolated=True, env=None):
        print('+ ' + shlex.join(map(str, command)), flush=True)
        result = subprocess.run(list(map(str, command)), cwd=self.consumer if isolated else ROOT,
            env=env or (self.env if isolated else None), text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
            preexec_fn=self.isolate if isolated else None)
        (self.root / (name + '.log')).write_text(result.stdout)
        with (self.root / 'commands.log').open('a') as log:
            log.write(shlex.join(map(str, command)) + '\n')
        if result.returncode:
            print(result.stdout, flush=True)
            raise subprocess.CalledProcessError(result.returncode, command)
        return result.stdout

    def prepare(self):
        assert not self.prefix.exists() and not self.prefix_a.exists(), 'use a fresh qualification directory'
        self.consumer.mkdir(parents=True)
        (self.root / 'tmp').mkdir()
        self.run(['python3', 'tools/generate_abi.py', '--check'], 'header-drift', isolated=False)
        # Metadata audits the complete locked graph, including target-specific crates.
        self.run(['cargo', 'fetch', '--locked'], 'fetch-locked-graph', isolated=False)
        self.run(['cargo', 'build', '--locked', '--release', '-p', 'replai-c'], 'release-binding', isolated=False)
        self.run(['cargo', 'build', '--locked', '--release', '-p', 'replai', '--example', 'terminal-state'], 'vt-oracle', isolated=False)
        self.run(['cargo', 'build', '--locked', '--release', '-p', 'replai-c', '--example', 'layout'], 'rust-layout-build', isolated=False)
        self.run(['python3', 'tools/stage_c.py', '--prefix', self.prefix_a], 'stage', isolated=False)
        self.prefix_a.rename(self.prefix)
        assert not self.prefix_a.exists(), 'original staging prefix still exists'
        for source in ['examples/c/demo.c', 'tests/c/contracts.c', 'tests/c/layout.c', 'tests/fixtures/presentation.tsv']:
            shutil.copy2(ROOT / source, self.consumer / Path(source).name)
        for name in ['layout', 'terminal-state']:
            shutil.copy2(ROOT / 'target/release/examples' / name, self.consumer / ('rust-' + name))
        for suffix in ['c', 'cpp']:
            (self.consumer / ('header.' + suffix)).write_text('#include <replai.h>\n')
        smoke = '#include <replai.h>\nint main() { uint32_t v = 0; return replai_abi_version(&v) != REPLAI_OK || v != REPLAI_C_ABI_VERSION; }\n'
        (self.consumer / 'smoke.cpp').write_text(smoke)
        (self.consumer / 'smoke.c').write_text(smoke)
        self.run(['cc', '--version'], 'compiler')
        self.run(['c++', '--version'], 'cpp-compiler')
        cflags = shlex.split(self.run(['pkg-config', '--cflags', 'replai'], 'pkg-cflags'))
        flags = ['-Wall', '-Wextra', '-Wpedantic', '-Werror']
        for compiler, standard, source in [('cc', 'c11', 'header.c'), ('c++', 'c++17', 'header.cpp')]:
            output = self.run([compiler, '-std=' + standard, *flags, *cflags, '-c', source, '-o', source + '.o'], 'header-' + standard)
            assert output == '', 'header compiler emitted diagnostics'
        self.run(['cc', '-std=c11', *flags, *cflags, 'layout.c', '-o', 'c-layout'], 'c-layout-build')
        c = self.run(['./c-layout'], 'c-layout')
        rust = self.run(['./rust-layout'], 'rust-layout')
        assert c == rust, 'C/Rust record or constant layout mismatch'
        print(c + 'ABI layout C == Rust', flush=True)
        self.run(['cc', '-std=c11', *cflags, '-M', 'demo.c'], 'resolved-header')
        assert str(self.prefix / 'lib/pkgconfig/../../include/replai.h') in (self.root / 'resolved-header.log').read_text()
        for mode in ['shared', 'static']:
            pkg = ['pkg-config', '--cflags', '--libs', *(['--static'] if mode == 'static' else []), 'replai']
            link = shlex.split(self.run(pkg, 'pkg-' + mode))
            if mode == 'static':
                link = [str(self.prefix / 'lib/libreplai_c.a') if arg == '-lreplai_c' else arg for arg in link]
            else:
                link.append('-Wl,-rpath,' + str(self.prefix / 'lib'))
            for source in ['demo.c', 'contracts.c']:
                output = self.run(['cc', '-std=c11', *flags, source, *link, '-o', Path(source).stem + '-' + mode], 'build-' + Path(source).stem + '-' + mode)
                assert output == '', 'consumer compiler emitted diagnostics'
        for mode in ['shared', 'static']:
            link = shlex.split(self.run(['pkg-config', '--cflags', *(['--static'] if mode == 'static' else []), '--libs', 'replai'], 'pkg-cpp-' + mode))
            if mode == 'static':
                link = [str(self.prefix / 'lib/libreplai_c.a') if arg == '-lreplai_c' else arg for arg in link]
            else:
                link.append('-Wl,-rpath,' + str(self.prefix / 'lib'))
            self.run(['c++', '-std=c++17', *flags, 'smoke.cpp', *link, '-o', 'cpp-' + mode], 'cpp-link-' + mode)
            self.run(['./cpp-' + mode], 'cpp-run-' + mode)
        self.cmake_consumers()
        print(f'Installed consumer compiled with repository reads denied; PREFIX={self.prefix}', flush=True)

    def cmake_consumers(self):
        for language, source, standard in [('C', 'smoke.c', '11'), ('CXX', 'smoke.cpp', '17')]:
            for mode in ['shared', 'static']:
                name = f'cmake-{language.lower()}-{mode}'
                source_dir = self.consumer / name
                source_dir.mkdir()
                shutil.copy2(self.consumer / source, source_dir / source)
                prop = 'C_STANDARD' if language == 'C' else 'CXX_STANDARD'
                (source_dir / 'CMakeLists.txt').write_text(
                    'cmake_minimum_required(VERSION 3.20)\n'
                    f'project(replai_external LANGUAGES {language})\n'
                    'find_package(replai 0.1 CONFIG REQUIRED)\n'
                    f'add_executable(consumer {source})\n'
                    f'set_property(TARGET consumer PROPERTY {prop} {standard})\n'
                    f'target_link_libraries(consumer PRIVATE replai::{mode})\n'
                    + (f'set_property(TARGET consumer PROPERTY BUILD_RPATH "{self.prefix / "lib"}")\n' if mode == 'shared' else '')
                )
                build = self.consumer / (name + '-build')
                self.run(['cmake', '-S', source_dir, '-B', build, '-DCMAKE_PREFIX_PATH=' + str(self.prefix), '-DCMAKE_BUILD_TYPE=Release'], name + '-configure')
                self.run(['cmake', '--build', build, '--verbose'], name + '-build')
                self.run([build / 'consumer'], name + '-run')

    def exercise(self, mode):
        output = self.run(['./contracts-' + mode], 'contracts-' + mode)
        print(output, flush=True)
        run_suite([str(self.consumer / ('demo-' + mode))], self.consumer / 'rust-terminal-state', self.consumer / 'presentation.tsv', self.root / ('pty-' + mode + '.json'), self.isolate, self.env)

    def memory(self):
        if sys.platform == 'darwin':
            for mode in ['static', 'shared']:
                report = self.run(['/usr/bin/leaks', '--atExit', '--', './contracts-'+mode], 'leaks-'+mode)
                assert '0 leaks for 0 total leaked bytes' in report, report
                print(report, flush=True)
            return
        executable = shutil.which(os.environ.get('VALGRIND', 'valgrind'))
        if executable is None:
            raise SystemExit('Valgrind is required; memory qualification cannot be skipped')
        executable = str(Path(executable).resolve())
        extra = [str(Path(executable).parents[2])] if not executable.startswith('/usr/') else []
        previous = self.isolate
        self.isolate = restrict_reads(self.root, extra)
        try:
            for mode in ['static', 'shared']:
                logfile = self.root / ('valgrind-' + mode + '.log')
                command = [executable, '--tool=memcheck', '--leak-check=full', '--show-leak-kinds=all', '--errors-for-leak-kinds=definite,indirect', '--error-exitcode=99', '--log-file=' + str(logfile), './contracts-' + mode]
                self.run(command, 'memory-contracts-' + mode)
                report = logfile.read_text()
                assert 'in use at exit: 0 bytes in 0 blocks' in report and 'ERROR SUMMARY: 0 errors' in report, report
                print(report, flush=True)
            # The actual event-loop demo also runs all PTY scenarios under Memcheck.
            command = [executable, '--tool=memcheck', '--leak-check=full', '--show-leak-kinds=all', '--errors-for-leak-kinds=definite,indirect', '--error-exitcode=99', '--log-file=' + str(self.root / 'valgrind-demo-%p.log'), str(self.consumer / 'demo-shared')]
            print('+ ' + shlex.join(command) + ' [PTY scenarios]', flush=True)
            run_suite(command, self.consumer / 'rust-terminal-state', self.consumer / 'presentation.tsv', self.root / 'pty-memory.json', self.isolate, self.env)
            for logfile in self.root.glob('valgrind-demo-*.log'):
                report = logfile.read_text()
                assert 'in use at exit: 0 bytes in 0 blocks' in report and 'ERROR SUMMARY: 0 errors' in report, report
        finally:
            self.isolate = previous

    def audit(self):
        manifest = tomllib.loads((ROOT / 'Cargo.toml').read_text())
        assert manifest['lints']['rust']['unsafe_code'] == 'forbid'
        metadata = json.loads(self.run(['cargo', 'metadata', '--locked', '--offline', '--format-version', '1'], 'dependency-audit', isolated=False))
        local = {p['name'] for p in metadata['packages'] if p['source'] is None}
        assert local == {'replai', 'replai-c'}, local
        assert all(p['source'] is None or p['source'].startswith('registry+') for p in metadata['packages'])
        binding = next(p for p in metadata['packages'] if p['name'] == 'replai-c')
        assert {d['name'] for d in binding['dependencies']} == {'replai', 'libc'}
        schema = json.loads((ROOT / 'api/c-abi.json').read_text())
        expected = {f['name'] for f in schema['functions']}
        if sys.platform == 'darwin':
            observed = self.run(['nm', '-gU', self.prefix / 'lib/libreplai_c.dylib'], 'symbols')
            assert {line.split()[-1].removeprefix('_') for line in observed.splitlines() if line.strip()} == expected, observed
            loader = self.run(['otool', '-L', './demo-shared'], 'loader-shared')
            assert '@rpath/libreplai_c.dylib' in loader, loader
            # The actual loaded image must come from this staged prefix.
            trace_env = {**self.env, 'DYLD_PRINT_LIBRARIES':'1'}
            loaded = self.run(['./cpp-shared'], 'loader-resolved', env=trace_env)
            assert str(self.prefix/'lib/libreplai_c.dylib') in loaded, loaded
            static = self.run(['otool', '-L', './demo-static'], 'loader-static')
        else:
            observed = self.run(['nm', '-D', '--defined-only', self.prefix / 'lib/libreplai_c.so'], 'symbols')
            assert {line.split()[-1] for line in observed.splitlines()} == expected, observed
            loader = self.run(['ldd', './demo-shared'], 'loader-shared')
            assert str(self.prefix / 'lib/libreplai_c.so') in loader, loader
            static = self.run(['ldd', './demo-static'], 'loader-static')
        assert 'libreplai_c' not in static, static
        pc = (self.prefix / 'lib/pkgconfig/replai.pc').read_text()
        assert '/home/' not in pc and str(ROOT) not in pc and str(self.root) not in pc
        for metadata in (self.prefix / 'lib/cmake/replai').glob('*.cmake'):
            value = metadata.read_text()
            assert str(ROOT) not in value and str(self.root) not in value and str(self.prefix_a) not in value
        assert not self.prefix_a.exists()
        print(loader + 'Namespace, staged loader resolution, metadata isolation: PASS', flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--work', type=Path, help='fresh persistent evidence directory')
    parser.add_argument('--phase', choices=['all', 'prepare', 'static', 'shared', 'memory', 'audit'], default='all')
    args = parser.parse_args()
    root = args.work or Path(tempfile.mkdtemp(prefix='replai-c-qualification-'))
    root.mkdir(parents=True, exist_ok=True)
    if args.phase == 'all':
        # Keep native tooling phases process-isolated. In particular, macOS
        # sandbox/leaks setup must not run in a Python process that previously
        # drove PTY children.
        for phase in ['prepare', 'static', 'shared', 'memory', 'audit']:
            result = subprocess.run(
                [sys.executable, __file__, '--work', str(root), '--phase', phase],
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
            )
            if result.returncode:
                print(result.stdout, flush=True)
                raise subprocess.CalledProcessError(result.returncode, result.args)
            print('GATE ' + phase + ' PASS', flush=True)
        print('Evidence: ' + str(root.resolve()), flush=True)
        return
    q = Qualification(root)
    for phase in [args.phase]:
        print('GATE ' + phase, flush=True)
        if phase in ('static', 'shared'):
            q.exercise(phase)
        else:
            getattr(q, phase)()
    print('Evidence: ' + str(root.resolve()), flush=True)


if __name__ == '__main__':
    main()
