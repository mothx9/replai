use std::{env, fs, path::Path};
fn dispatch(target: &str, bytes: &[u8]) {
    match target {
        "protocol" => replai_hardening::protocol_case(bytes),
        "editor" => replai_hardening::editor_case(bytes),
        "results" => replai_hardening::results_case(bytes),
        "geometry" => replai_hardening::geometry_case(bytes),
        #[cfg(unix)]
        "cabi" => replai_hardening::cabi_case(bytes),
        _ => panic!("unsupported target {target}"),
    }
}
fn main() {
    let args: Vec<_> = env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("faults") => {
            replai_hardening::fault_campaign(args[2].parse().unwrap());
        }
        Some("generated") => {
            let count: u64 = args[2].parse().unwrap();
            let first: u64 = args.get(3).map(|s| s.parse().unwrap()).unwrap_or(1);
            assert!(count <= 1_000_000);
            for seed in first..first + count {
                let mut state = seed;
                let mut data = [0u8; 512];
                for b in &mut data {
                    state = state
                        .wrapping_mul(6364136223846793005)
                        .wrapping_add(1442695040888963407);
                    *b = (state >> 32) as u8;
                }
                // Printed before execution so a panic identifies its replay seed.
                if seed == first || seed % 1000 == 0 {
                    eprintln!("seed {seed}");
                }
                let result = std::panic::catch_unwind(|| {
                    dispatch("editor", &data);
                    dispatch("results", &data);
                });
                if let Err(p) = result {
                    eprintln!("FAILED seed {seed}");
                    std::panic::resume_unwind(p);
                }
            }
            println!(
                "{{\"sequences\":{count},\"first_seed\":{first},\"max_operations\":128,\"oracles\":[\"editor-history\",\"host-results-lifecycle\"]}}"
            );
        }
        Some(target) => {
            let path = Path::new(&args[2]);
            let mut paths = if path.is_dir() {
                fs::read_dir(path)
                    .unwrap()
                    .map(|e| e.unwrap().path())
                    .filter(|p| p.is_file())
                    .collect::<Vec<_>>()
            } else {
                vec![path.into()]
            };
            paths.sort();
            for p in &paths {
                eprintln!("{}", p.display());
                dispatch(target, &fs::read(p).unwrap());
            }
            println!("{{\"target\":\"{target}\",\"inputs\":{}}}", paths.len());
        }
        _ => panic!("replay TARGET CORPUS_DIR | replay generated COUNT [FIRST_SEED]"),
    }
}
