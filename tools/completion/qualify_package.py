#!/usr/bin/env python3
"""Build a new-surface consumer through an isolated local Cargo registry."""
import argparse
import hashlib
import http.server
import json
from pathlib import Path
import shutil
import subprocess
import threading


def run(command, cwd):
    subprocess.run(command, cwd=cwd, check=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--crate", type=Path, required=True)
    parser.add_argument("--work", type=Path, required=True)
    parser.add_argument("--toolchain", default="1.98.1")
    args = parser.parse_args()
    args.work.mkdir(parents=True, exist_ok=False)
    web = args.work / "web"
    index = web / "index"
    download = web / "crates/replai/0.1.0"
    (index / "re/pl").mkdir(parents=True)
    download.mkdir(parents=True)
    crate = args.crate.resolve()
    shutil.copy2(crate, download / "download")

    server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), http.server.SimpleHTTPRequestHandler)
    port = server.server_address[1]
    (index / "config.json").write_text(json.dumps({
        "dl": f"http://127.0.0.1:{port}/crates/{{crate}}/{{version}}/download",
        "api": None,
    }))
    crates_io = "https://github.com/rust-lang/crates.io-index"
    dependencies = [
        {"name":"unicode-segmentation","req":"^1.13.3","features":[],"optional":False,"default_features":True,"target":None,"kind":"normal","registry":crates_io},
        {"name":"unicode-width","req":"^0.2.2","features":[],"optional":False,"default_features":False,"target":None,"kind":"normal","registry":crates_io},
        {"name":"rustix","req":"^1.1.4","features":["termios","event","fs"],"optional":False,"default_features":True,"target":"cfg(any(target_os = \"linux\", target_os = \"macos\"))","kind":"normal","registry":crates_io},
        {"name":"nix","req":"=0.31.3","features":["event","poll"],"optional":False,"default_features":False,"target":"cfg(target_os = \"macos\")","kind":"normal","registry":crates_io},
    ]
    record = {"name":"replai","vers":"0.1.0","deps":dependencies,"cksum":hashlib.sha256(crate.read_bytes()).hexdigest(),"features":{},"yanked":False,"links":None,"rust_version":"1.98.1"}
    (index / "re/pl/replai").write_text(json.dumps(record, separators=(",", ":")) + "\n")

    consumer = args.work / "consumer"
    (consumer / ".cargo").mkdir(parents=True)
    (consumer / "src").mkdir()
    (consumer / ".cargo/config.toml").write_text(
        f'[registries.replai-local]\nindex = "sparse+http://127.0.0.1:{port}/index/"\n'
    )
    (consumer / "Cargo.toml").write_text('''[package]
name = "replai-completion-package-consumer"
version = "0.0.0"
edition = "2024"
publish = false

[dependencies]
replai = { version = "=0.1.0", registry = "replai-local" }
''')
    (consumer / "src/main.rs").write_text('''use replai::{Action, CompletionItem, EditAction, Editor, Key, KeyMap, MatchCase, complete_prefix, suggest_from_static};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut map = KeyMap::new();
    map.bind(Key::meta(b'r')?, Action::Edit(EditAction::Redo))?;
    let mut editor = Editor::new(128, 0);
    editor.insert("dep")?;
    let snapshot = editor.analysis_snapshot();
    let items = [CompletionItem::new("deploy")?, CompletionItem::new("debug")?];
    assert_eq!(complete_prefix(&snapshot, 0..3, "dep", &items, MatchCase::Sensitive)?.candidates().len(), 1);
    assert_eq!(suggest_from_static(&snapshot, &["deploy"])?.unwrap().text(), "loy");
    println!("packaged-keymap-completion-suggestion-ok");
    Ok(())
}
''')
    old = Path.cwd()
    try:
        import os
        os.chdir(web)
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        run(["cargo", f"+{args.toolchain}", "generate-lockfile"], consumer)
        run(["cargo", f"+{args.toolchain}", "check", "--locked"], consumer)
        run(["cargo", f"+{args.toolchain}", "test", "--locked"], consumer)
        run(["cargo", f"+{args.toolchain}", "run", "--locked"], consumer)
    finally:
        server.shutdown()
        server.server_close()
        os.chdir(old)
    summary = {
        "status": "PASS", "dependency": "isolated sparse registry, no path/workspace dependency",
        "crate_sha256": record["cksum"], "toolchain": args.toolchain,
        "consumer_source_sha256": hashlib.sha256((consumer / "src/main.rs").read_bytes()).hexdigest(),
        "lock_sha256": hashlib.sha256((consumer / "Cargo.lock").read_bytes()).hexdigest(),
    }
    (args.work / "summary.json").write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n")
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
