//! C calls use live, disjoint, correctly sized allocations. Never fuzz addresses.
#[cfg(test)]
use crate::terminal::pty_support as pty;
use replai_c::*;
use std::{mem::size_of, os::fd::AsRawFd, ptr};
#[cfg(not(test))]
#[path = "../../tests/support/posix_pty.rs"]
mod pty;

pub fn cabi_case(data: &[u8]) {
    let (master, slave) = pty::pair();
    rustix::fs::fcntl_setfl(&master, rustix::fs::OFlags::NONBLOCK).unwrap();
    rustix::termios::tcsetwinsize(
        &slave,
        rustix::termios::Winsize {
            ws_row: 24,
            ws_col: 80,
            ws_xpixel: 0,
            ws_ypixel: 0,
        },
    )
    .unwrap();
    let termios = rustix::termios::tcgetattr(&slave).unwrap();
    let mut h = ptr::null_mut();
    let mut config = ReplaiConfig {
        struct_size: size_of::<ReplaiConfig>() as u32,
        abi_version: 1,
        max_input_bytes: 4096,
        history_entries: 8,
        reserved: [0; 2],
    };
    let mut output = [0xa5_u8; 4098];
    let mut drain = [0_u8; 32768];
    // SAFETY: each record lives for the call, spans are within their allocations,
    // outputs never alias inputs/handle, and destruction nulls the sole handle.
    unsafe {
        config.struct_size = 4;
        assert_eq!(replai_create(&config, &mut h), REPLAI_ABI_MISMATCH);
        assert!(h.is_null());
        config.struct_size = size_of::<ReplaiConfig>() as u32;
        config.abi_version = 2;
        assert_eq!(replai_create(&config, &mut h), REPLAI_ABI_MISMATCH);
        assert!(h.is_null());
        config.abi_version = 1;
        assert_eq!(replai_create(&config, &mut h), REPLAI_OK);
        // Exercise the exact admitted boundary using live buffers, then start
        // arbitrary call sequences from the ordinary empty draft.
        let capacity_input = [b'x'; 4097];
        assert_eq!(
            replai_set_draft(h, capacity_input.as_ptr(), 4096),
            REPLAI_OK
        );
        assert_eq!(
            replai_set_draft(h, capacity_input.as_ptr(), 4097),
            REPLAI_CAPACITY
        );
        let (mut required, mut cursor) = (0, 0);
        assert_eq!(
            replai_draft_copy(
                h,
                output.as_mut_ptr().add(1),
                4095,
                &mut required,
                &mut cursor
            ),
            REPLAI_BUFFER_TOO_SMALL
        );
        assert_eq!(required, 4096);
        assert!(
            output.iter().all(|b| *b == 0xa5),
            "short copy must not write partially"
        );
        assert_eq!(
            replai_draft_copy(
                h,
                output.as_mut_ptr().add(1),
                4096,
                &mut required,
                &mut cursor
            ),
            REPLAI_OK
        );
        assert_eq!((required, cursor), (4096, 4096));
        assert_eq!(&output[1..4097], &capacity_input[..4096]);
        assert_eq!(output[0], 0xa5);
        assert_eq!(output[4097], 0xa5);
        assert_eq!(replai_clear(h), REPLAI_OK);
        for (i, c) in data.chunks(4).take(64).enumerate() {
            let byte = c[0];
            let payload = &data[i.min(data.len())..data.len().min(i + 256)];
            let p = payload.as_ptr();
            let n = payload.len();
            let mut event = ReplaiEvent {
                struct_size: size_of::<ReplaiEvent>() as u32,
                abi_version: 1,
                kind: 0,
                status: 0,
                text_bytes: 0,
                cursor_bytes: 0,
                reserved: [0; 2],
            };
            let mut required = 0;
            let mut cursor = 0;
            let status = match byte % 15 {
                0 => replai_open(h, slave.as_raw_fd(), slave.as_raw_fd()),
                1 => replai_close(h),
                2 => replai_set_draft(h, p, n),
                3 => replai_history_add(h, p, n),
                4 => replai_clear(h),
                5 => replai_complete(
                    h,
                    c.get(1).copied().unwrap_or(0) as usize,
                    c.get(2).copied().unwrap_or(0) as usize,
                    p,
                    n,
                ),
                6 => replai_external_output(h, c.get(1).copied().unwrap_or(0) as u32, p, n),
                7 => replai_prompt(h, p, n, b"".as_ptr(), 0, b"... ".as_ptr(), 4),
                8 => {
                    let _ = rustix::io::write(&master, b"x\r");
                    replai_poll(h, 0, &mut event)
                }
                9 => {
                    event.struct_size = 4;
                    let s = replai_poll(h, 0, &mut event);
                    assert_eq!(s, REPLAI_ABI_MISMATCH);
                    s
                }
                10 => {
                    event.abi_version = 2;
                    let s = replai_interrupt(h, &mut event);
                    assert_eq!(s, REPLAI_ABI_MISMATCH);
                    s
                }
                11 => replai_interrupt(h, &mut event),
                12 => {
                    let capacity = c.get(1).copied().unwrap_or(0) as usize;
                    let mut small = vec![0xa5; capacity + 2];
                    let result = replai_submitted_copy(
                        h,
                        small.as_mut_ptr().add(1),
                        capacity,
                        &mut required,
                    );
                    assert_eq!(small[0], 0xa5);
                    assert_eq!(small[capacity + 1], 0xa5);
                    if result == REPLAI_BUFFER_TOO_SMALL {
                        assert!(small.iter().all(|b| *b == 0xa5));
                    }
                    result
                }
                13 => replai_draft_copy(h, ptr::null_mut(), 0, &mut required, &mut cursor),
                _ => {
                    let capacity = c.get(1).copied().unwrap_or(0) as usize;
                    let mut small = vec![0xa5; capacity + 2];
                    let result = replai_status_text(
                        byte as i32,
                        small.as_mut_ptr().add(1),
                        capacity,
                        &mut required,
                    );
                    assert_eq!(small[0], 0xa5);
                    assert_eq!(small[capacity + 1], 0xa5);
                    if result == REPLAI_BUFFER_TOO_SMALL {
                        assert!(small.iter().all(|b| *b == 0xa5));
                    }
                    result
                }
            };
            assert_ne!(
                status, REPLAI_INTERNAL,
                "contained panic is still a finding"
            );
            assert!((0..=14).contains(&status));
            assert_eq!(output[0], 0xa5);
            assert_eq!(output[4097], 0xa5);
            assert_eq!(
                replai_draft_copy(
                    h,
                    output.as_mut_ptr().add(1),
                    4096,
                    &mut required,
                    &mut cursor
                ),
                REPLAI_OK
            );
            let s = std::str::from_utf8(&output[1..1 + required]).unwrap();
            assert!(s.is_char_boundary(cursor));
            assert!(required <= 4096);
            while rustix::io::read(&master, &mut drain).is_ok_and(|n| n > 0) {}
        }
        assert_eq!(replai_close(h), REPLAI_OK);
        assert_eq!(replai_close(h), REPLAI_OK);
        assert_eq!(replai_destroy(&mut h), REPLAI_OK);
        assert!(h.is_null());
        assert_eq!(replai_destroy(&mut h), REPLAI_INVALID_ARGUMENT);
    }
    assert_eq!(
        format!("{:?}", rustix::termios::tcgetattr(&slave).unwrap()),
        format!("{termios:?}")
    );
}
