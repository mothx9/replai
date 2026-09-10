#![no_main]
libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    replai_hardening::protocol_case(data);
});
