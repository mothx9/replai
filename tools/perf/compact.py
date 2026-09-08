#!/usr/bin/env python3
"""Select a reproducible schema-v1 dossier subset; keep full raw files outside Git."""
import argparse
import json
from pathlib import Path
import re
from results import ROOT, validate

SIZES = {0, 64, 4096, 65536, 1048576}
SCALED = {'editor', 'layout', 'render', 'vt_encode', 'movement', 'paste_decoder',
          'paste_normalization', 'paste_keymap', 'paste_editor', 'paste_layout_render'}


def compact(data):
    validate(data)
    result = json.loads(json.dumps(data))
    result['results'] = [r for r in result['results']
                         if r['component'] not in SCALED or r.get('input_bytes') in SIZES]
    for row in result['results']:
        # Native size's verbose archive/member and virtual-segment listing is a
        # raw trace, not the disk-size measurement; it can contain staging paths.
        row.pop('sections', None)
    env = result['environment']
    for key, value in list(env.items()):
        if isinstance(value, str):
            value = value.replace(str(ROOT), '<repository>')
            value = re.sub(r'^InstalledDir:.*$', 'InstalledDir: <system toolchain>', value, flags=re.M)
            value = re.sub(r' \(id=\d+\)', '', value)
            env[key] = value
    result['selection'] = dict(sizes_bytes=sorted(SIZES), scaled_components=sorted(SCALED),
                              full_result_count=len(data['results']),
                              omitted='unselected scales and verbose native section/archive listings; full run.py output retains them',
                              environment_redaction='repository path, compiler installation directory, transient battery device id')
    validate(result)
    return result


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('input', type=Path)
    p.add_argument('output', type=Path)
    a = p.parse_args()
    result = compact(json.loads(a.input.read_text()))
    a.output.parent.mkdir(parents=True, exist_ok=True)
    a.output.write_text(json.dumps(result, separators=(',', ':')) + '\n')
    print(f"{len(result['results'])} representative results from {result['selection']['full_result_count']}")
