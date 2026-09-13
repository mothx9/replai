#![no_main]
libfuzzer_sys::fuzz_target!(|data: &[u8]| { replai_hardening::surface_case(data); });
