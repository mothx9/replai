#!/usr/bin/env python3
"""Stage built release artifacts into an empty caller-selected prefix."""
import argparse
from pathlib import Path
import shutil
import sys
import subprocess
import tomllib

ROOT = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--prefix', type=Path, required=True)
parser.add_argument('--artifacts', type=Path, default=ROOT / 'target/release')
args = parser.parse_args()
prefix = args.prefix.resolve()
manifest = tomllib.loads((ROOT / 'Cargo.toml').read_text())
version = manifest['package']['version']
if prefix.exists() and any(prefix.iterdir()):
    raise SystemExit(f'staging prefix must be empty: {prefix}')
for name in ['libreplai_c.a', 'libreplai_c.dylib' if sys.platform == 'darwin' else 'libreplai_c.so']:
    if not (args.artifacts / name).is_file():
        raise SystemExit(f'build release binding first: missing {name}')
(prefix / 'include').mkdir(parents=True, exist_ok=True)
(prefix / 'lib/pkgconfig').mkdir(parents=True)
(prefix / 'lib/cmake/replai').mkdir(parents=True)
shutil.copy2(ROOT / 'include/replai.h', prefix / 'include/replai.h')
for name in ['libreplai_c.a', 'libreplai_c.dylib' if sys.platform == 'darwin' else 'libreplai_c.so']:
    shutil.copy2(args.artifacts / name, prefix / 'lib' / name)
if sys.platform == 'darwin':
    subprocess.run(['install_name_tool', '-id', '@rpath/libreplai_c.dylib', prefix/'lib/libreplai_c.dylib'], check=True)
private = '-liconv' if sys.platform == 'darwin' else '-lgcc_s -lutil -lrt -lpthread -lm -ldl -lc'
(prefix / 'lib/pkgconfig/replai.pc').write_text('''prefix=${pcfiledir}/../..
libdir=${prefix}/lib
includedir=${prefix}/include

Name: replai
Description: REPLAI terminal interaction C binding (pre-release ABI 1)
Version: VERSION
Libs: -L${libdir} -lreplai_c
Libs.private: PRIVATE_LIBS
Cflags: -I${includedir}
'''.replace('VERSION', version).replace('PRIVATE_LIBS', private))
static_system = 'iconv' if sys.platform == 'darwin' else 'gcc_s;util;rt;pthread;m;dl;c'
shared_name = 'libreplai_c.dylib' if sys.platform == 'darwin' else 'libreplai_c.so'
replacements = {
    '@REPLAI_VERSION@': version,
    '@REPLAI_SHARED_LIBRARY@': shared_name,
    '@REPLAI_STATIC_SYSTEM_LIBS@': static_system,
}
for template in sorted((ROOT / 'cmake').glob('*.in')):
    text = template.read_text()
    for key, value in replacements.items():
        text = text.replace(key, value)
    destination = prefix / 'lib/cmake/replai' / template.name.removesuffix('.in')
    destination.write_text(text)
(prefix / 'share/licenses/replai').mkdir(parents=True)
shutil.copy2(ROOT / 'LICENSE', prefix / 'share/licenses/replai/LICENSE')
print(prefix)
