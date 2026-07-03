//! Realtime-safety: the audio-callback path ([`fill_frames`]) never allocates
//! and zero-fills underruns. `assert_no_alloc` is BSD-1-Clause (disallowed by
//! the MIT/Apache policy), so we hand-roll a counting global allocator scoped
//! to THIS test binary that flags allocations while a thread-local FORBID flag
//! is set.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

use voksa_cli::playback::{fill_frames, fill_from_ring};

thread_local! {
    static FORBID: Cell<bool> = const { Cell::new(false) };
    static VIOLATIONS: Cell<usize> = const { Cell::new(0) };
}

struct CountingAlloc;

// SAFETY: every operation delegates to System; we only add bookkeeping that
// never allocates and never panics.
unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        note();
        unsafe { System.alloc(l) }
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        note();
        unsafe { System.dealloc(p, l) }
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
        note();
        unsafe { System.realloc(p, l, n) }
    }
    unsafe fn alloc_zeroed(&self, l: Layout) -> *mut u8 {
        note();
        unsafe { System.alloc_zeroed(l) }
    }
}

fn note() {
    // try_with: TLS may be mid-teardown on some threads — never panic here.
    let _ = FORBID.try_with(|forbid| {
        if forbid.get() {
            let _ = VIOLATIONS.try_with(|v| v.set(v.get() + 1));
        }
    });
}

#[global_allocator]
static ALLOC: CountingAlloc = CountingAlloc;

/// Run `f` with the FORBID flag set, then assert no allocation happened.
fn assert_no_alloc<R>(f: impl FnOnce() -> R) -> R {
    FORBID.with(|c| c.set(true));
    let r = f();
    FORBID.with(|c| c.set(false));
    let v = VIOLATIONS.with(|v| v.replace(0));
    assert_eq!(v, 0, "allocator entered {v} time(s) on the callback path");
    r
}

#[test]
fn callback_path_never_allocates() {
    let (mut p, mut c) = rtrb::RingBuffer::<f32>::new(4096);
    for i in 0..3000 {
        p.push(i as f32).unwrap();
    }
    let mut out = vec![0.0f32; 512];
    // Pre-warm: touch the TLS cells and run one fill so no first-access
    // allocation is attributed to the forbidden region.
    FORBID.with(|c| {
        let _ = c.get();
    });
    VIOLATIONS.with(|v| {
        let _ = v.get();
    });
    let _ = fill_frames(&mut c, &mut out, 2);
    assert_no_alloc(|| {
        for _ in 0..8 {
            fill_frames(&mut c, &mut out, 2);
        }
    });
}

#[test]
fn underrun_zero_fills() {
    let (mut p, mut c) = rtrb::RingBuffer::<f32>::new(256);
    for _ in 0..100 {
        p.push(0.7).unwrap();
    }
    let mut out = vec![9.0f32; 256];
    let n = fill_from_ring(&mut c, &mut out);
    assert_eq!(n, 100, "took all 100 available");
    assert!(out[..100].iter().all(|&s| s == 0.7), "ring data copied");
    assert!(out[100..].iter().all(|&s| s == 0.0), "underrun zero-filled");
}

#[test]
fn partial_then_empty_semantics() {
    let (mut p, mut c) = rtrb::RingBuffer::<f32>::new(64);
    for _ in 0..10 {
        p.push(0.3).unwrap();
    }
    let mut out = vec![0.0f32; 8];
    assert_eq!(fill_from_ring(&mut c, &mut out), 8, "first call drains 8");
    let n = fill_from_ring(&mut c, &mut out);
    assert_eq!(n, 2, "second call drains the last 2");
    assert!(out[2..].iter().all(|&s| s == 0.0), "tail zero-filled");
    let mut out2 = vec![5.0f32; 8];
    assert_eq!(fill_from_ring(&mut c, &mut out2), 0, "empty ring");
    assert!(out2.iter().all(|&s| s == 0.0), "all zero on empty");
}

#[test]
fn stereo_fan_out_duplicates_mono() {
    let (mut p, mut c) = rtrb::RingBuffer::<f32>::new(64);
    for i in 1..=5 {
        p.push(i as f32).unwrap();
    }
    let mut out = vec![0.0f32; 16]; // 8 stereo frames
    let n = fill_frames(&mut c, &mut out, 2);
    assert_eq!(n, 5);
    for frame in 0..5 {
        assert_eq!(out[frame * 2], out[frame * 2 + 1], "L == R");
        assert_eq!(out[frame * 2], (frame + 1) as f32);
    }
    assert!(out[10..].iter().all(|&s| s == 0.0), "tail frames zeroed");
}
