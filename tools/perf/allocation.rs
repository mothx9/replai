//! Benchmark-process-only System allocator observation; never linked into REPLAI.
#[cfg(feature = "allocations")]
mod observed {
    use std::alloc::{GlobalAlloc, Layout, System};
    use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
    pub struct Counting;
    static LIVE: AtomicU64 = AtomicU64::new(0);
    static PEAK: AtomicU64 = AtomicU64::new(0);
    static COUNT: AtomicU64 = AtomicU64::new(0);
    static BYTES: AtomicU64 = AtomicU64::new(0);
    fn add(n: usize) {
        COUNT.fetch_add(1, Relaxed);
        BYTES.fetch_add(n as u64, Relaxed);
        let live = LIVE.fetch_add(n as u64, Relaxed) + n as u64;
        PEAK.fetch_max(live, Relaxed);
    }
    // Safety: forward each unchanged pointer/layout pair to System. Counters use
    // non-allocating atomics. This single-threaded fixture never exposes pointers.
    unsafe impl GlobalAlloc for Counting {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            let p = unsafe { System.alloc(layout) };
            if !p.is_null() {
                add(layout.size());
            }
            p
        }
        unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
            let p = unsafe { System.alloc_zeroed(layout) };
            if !p.is_null() {
                add(layout.size());
            }
            p
        }
        unsafe fn dealloc(&self, p: *mut u8, layout: Layout) {
            LIVE.fetch_sub(layout.size() as u64, Relaxed);
            unsafe { System.dealloc(p, layout) };
        }
        unsafe fn realloc(&self, p: *mut u8, layout: Layout, size: usize) -> *mut u8 {
            let q = unsafe { System.realloc(p, layout, size) };
            if !q.is_null() {
                LIVE.fetch_sub(layout.size() as u64, Relaxed);
                add(size);
            }
            q
        }
    }
    pub fn start() -> (u64, u64, u64) {
        let live = LIVE.load(Relaxed);
        PEAK.store(live, Relaxed);
        (COUNT.load(Relaxed), BYTES.load(Relaxed), live)
    }
    pub fn end(s: (u64, u64, u64)) -> serde_json::Value {
        // Snapshot before JSON construction, which is not part of the operation.
        let count = COUNT.load(Relaxed) - s.0;
        let bytes = BYTES.load(Relaxed) - s.1;
        let retained = LIVE.load(Relaxed) as i64 - s.2 as i64;
        let peak = PEAK.load(Relaxed).saturating_sub(s.2);
        serde_json::json!({"allocation_calls":count,"requested_bytes":bytes,
            "retained_delta_bytes":retained,"peak_live_delta_bytes":peak})
    }
}
#[cfg(feature = "allocations")]
#[global_allocator]
static ALLOCATOR: observed::Counting = observed::Counting;
#[cfg(feature = "allocations")]
pub use observed::{end, start};
#[cfg(not(feature = "allocations"))]
pub fn start() -> (u64, u64, u64) {
    (0, 0, 0)
}
#[cfg(not(feature = "allocations"))]
pub fn end(_: (u64, u64, u64)) -> serde_json::Value {
    serde_json::Value::Null
}
