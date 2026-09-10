fn main() {
    let args: Vec<_> = std::env::args().collect();
    replai_hardening::benchmark(
        args.get(1).map_or(31, |s| s.parse().unwrap()),
        args.get(2).map_or(5, |s| s.parse().unwrap()),
    );
}
