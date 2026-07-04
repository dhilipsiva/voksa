//! voksa browser adapter. The shipping surface is a raw C-ABI with NO
//! wasm-bindgen runtime, so the compiled module declares zero imports and an
//! AudioWorklet instantiates it synchronously with
//! `new WebAssembly.Instance(module, {})` (see www/voksa-processor.js). The
//! worklet marshals text into wasm memory, calls [`voksa_render`], copies the
//! samples out, and plays them 128 frames at a time.

use std::sync::atomic::{AtomicUsize, Ordering};

use voksa_core::compiler::{CompileError, CompileOptions};
use voksa_core::prosody::ProsodyOptions;
use voksa_engine_klattsch::{render_utterance, render_utterance_prosodic};

/// `flags` bit layout, mirroring the native CLI (default 0 = prosodic).
pub const FLAG_FLAT: u32 = 0x1;
pub const FLAG_XU: u32 = 0x2;
pub const FLAG_DOTSIDE: u32 = 0x4;
pub const FLAG_BUFFER: u32 = 0x8;

/// Render Lojban `text` to mono f32 PCM at `sample_rate`. Shared by the C-ABI
/// exports and the tests. `xu` is ignored on the flat branch.
pub fn synth(text: &str, flags: u32, sample_rate: u32) -> Result<Vec<f32>, CompileError> {
    let opts = CompileOptions {
        dotside: flags & FLAG_DOTSIDE != 0,
        buffer: flags & FLAG_BUFFER != 0,
    };
    if flags & FLAG_FLAT != 0 {
        render_utterance(text, &opts, sample_rate)
    } else {
        let prosody = ProsodyOptions {
            xu_rise: flags & FLAG_XU != 0,
        };
        render_utterance_prosodic(text, &opts, &prosody, sample_rate)
    }
}

/// Sample count of the buffer from the most recent [`voksa_render`]. A single
/// register is sound because the AudioWorklet is single-threaded and calls
/// render exactly once before reading it.
static OUT_LEN: AtomicUsize = AtomicUsize::new(0);

/// Allocate `len` bytes of wasm memory for the caller (JS) to write UTF-8 text
/// into; free it with [`voksa_dealloc`] passing the same `len`.
///
/// # Safety
/// The returned pointer is valid for `len` bytes until [`voksa_dealloc`].
#[unsafe(no_mangle)]
pub extern "C" fn voksa_alloc(len: usize) -> *mut u8 {
    let mut buf = Vec::<u8>::with_capacity(len);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

/// Free a buffer from [`voksa_alloc`].
///
/// # Safety
/// `ptr`/`len` must come from a prior [`voksa_alloc`] call with the same `len`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn voksa_dealloc(ptr: *mut u8, len: usize) {
    if !ptr.is_null() {
        // len == capacity: `voksa_alloc` used `with_capacity(len)`.
        drop(unsafe { Vec::from_raw_parts(ptr, 0, len) });
    }
}

/// Render `text` (UTF-8 at `text_ptr`/`text_len`) and return the base pointer
/// of the f32 sample buffer; read its length with [`voksa_out_len`], then free
/// it with [`voksa_free_f32`]. Returns null (length 0) on a compile error or
/// invalid UTF-8.
///
/// # Safety
/// `text_ptr`/`text_len` must describe a readable byte range in wasm memory.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn voksa_render(
    text_ptr: *const u8,
    text_len: usize,
    flags: u32,
    sample_rate: u32,
) -> *mut f32 {
    let bytes = unsafe { std::slice::from_raw_parts(text_ptr, text_len) };
    let Ok(text) = std::str::from_utf8(bytes) else {
        OUT_LEN.store(0, Ordering::Relaxed);
        return std::ptr::null_mut();
    };
    match synth(text, flags, sample_rate) {
        Ok(mut samples) => {
            samples.shrink_to_fit(); // guarantee capacity == len for the free
            let ptr = samples.as_mut_ptr();
            OUT_LEN.store(samples.len(), Ordering::Relaxed);
            std::mem::forget(samples);
            ptr
        }
        Err(_) => {
            OUT_LEN.store(0, Ordering::Relaxed);
            std::ptr::null_mut()
        }
    }
}

/// Sample count of the buffer returned by the most recent [`voksa_render`].
#[unsafe(no_mangle)]
pub extern "C" fn voksa_out_len() -> usize {
    OUT_LEN.load(Ordering::Relaxed)
}

/// Free a sample buffer from [`voksa_render`].
///
/// # Safety
/// `ptr`/`len` must be a prior [`voksa_render`] return with its [`voksa_out_len`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn voksa_free_f32(ptr: *mut f32, len: usize) {
    if !ptr.is_null() {
        drop(unsafe { Vec::from_raw_parts(ptr, len, len) });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const SR: u32 = 48_000;

    #[test]
    fn coi_munje_finite_nonempty() {
        let s = synth("coi munje", 0, SR).expect("synth ok");
        assert!(!s.is_empty());
        assert!(s.iter().all(|x| x.is_finite()));
    }

    #[test]
    fn flat_and_prosodic_differ() {
        let flat = synth("coi munje", FLAG_FLAT, SR).unwrap();
        let prosodic = synth("coi munje", 0, SR).unwrap();
        assert_ne!(flat, prosodic, "prosody must change the samples");
    }

    #[test]
    fn empty_text_errors() {
        assert!(matches!(synth("", 0, SR), Err(CompileError::Empty)));
    }

    #[test]
    fn dotside_flag_changes_output() {
        // "coi la djan": la-family exempts djan from the pre-cmevla pause;
        // --dotside drops that exemption, so the schedule (and audio) differ.
        let plain = synth("coi la djan", 0, SR).unwrap();
        let dotside = synth("coi la djan", FLAG_DOTSIDE, SR).unwrap();
        assert_ne!(plain, dotside);
    }
}
