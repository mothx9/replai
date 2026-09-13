//! Seeded semantic sequence campaign for INTERACTION.ERGONOMICS.0.
use serde_json::json;

fn main() {
    let count: u64 = std::env::args()
        .nth(1)
        .as_deref()
        .unwrap_or("100000")
        .parse()
        .expect("sequence count");
    let seed: u64 = std::env::args()
        .nth(2)
        .as_deref()
        .unwrap_or("7632459182231841")
        .parse()
        .expect("seed");
    assert!((1..=1_000_000).contains(&count));
    let mut random = seed;
    let mut data = [0u8; 128 * 4];
    for _ in 0..count {
        for byte in &mut data {
            random = random
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            *byte = (random >> 32) as u8;
        }
        replai_hardening::editor_case(&data);
    }
    println!(
        "{}",
        json!({"passed": true, "sequences": count, "operations_per_sequence": 128, "seed": seed, "final_state": random})
    );
}
