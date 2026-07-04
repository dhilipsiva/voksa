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

/// Count of f32 prosody knobs in the fixed layout the demo + CLI share:
/// `[declination_start_hz, declination_end_hz, stress_duration_factor,
/// stress_f0_excursion_hz, stress_amp_factor, xu_rise_hz, rate]`.
pub const PARAM_COUNT: usize = 7;

/// Build [`ProsodyOptions`] from the flag bits + the f32 param block (fixed
/// order). Missing or non-finite entries fall back to the defaults, so an empty
/// slice reproduces `voksa_render` and the layout is forward-compatible.
fn prosody_from(flags: u32, params: &[f32]) -> ProsodyOptions {
    // STUB (D1 web red): the real param decode lands after the failing test.
    let _ = params;
    ProsodyOptions {
        xu_rise: flags & FLAG_XU != 0,
        ..Default::default()
    }
}

/// Render Lojban `text` to mono f32 PCM at `sample_rate`. `params` is the f32
/// prosody block (empty = all defaults). Shared by the C-ABI exports + tests.
pub fn synth(
    text: &str,
    flags: u32,
    sample_rate: u32,
    params: &[f32],
) -> Result<Vec<f32>, CompileError> {
    let opts = CompileOptions {
        dotside: flags & FLAG_DOTSIDE != 0,
        buffer: flags & FLAG_BUFFER != 0,
    };
    if flags & FLAG_FLAT != 0 {
        render_utterance(text, &opts, sample_rate)
    } else {
        render_utterance_prosodic(text, &opts, &prosody_from(flags, params), sample_rate)
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
    unsafe { render_into(text_ptr, text_len, flags, sample_rate, std::ptr::null(), 0) }
}

/// Like [`voksa_render`], but with a prosody param block: `params_ptr` points to
/// `params_len` f32s in the [`PARAM_COUNT`] layout (a shorter block defaults the
/// rest). JS writes it into wasm memory via [`voksa_alloc`], like the text.
///
/// # Safety
/// `text_ptr`/`text_len` and `params_ptr`/`params_len` must describe readable
/// ranges in wasm memory (`params_ptr` may be null iff `params_len == 0`).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn voksa_render_params(
    text_ptr: *const u8,
    text_len: usize,
    flags: u32,
    sample_rate: u32,
    params_ptr: *const f32,
    params_len: usize,
) -> *mut f32 {
    unsafe {
        render_into(
            text_ptr,
            text_len,
            flags,
            sample_rate,
            params_ptr,
            params_len,
        )
    }
}

/// # Safety
/// The pointer ranges must be readable; `params_ptr` may be null iff len 0.
unsafe fn render_into(
    text_ptr: *const u8,
    text_len: usize,
    flags: u32,
    sample_rate: u32,
    params_ptr: *const f32,
    params_len: usize,
) -> *mut f32 {
    let bytes = unsafe { std::slice::from_raw_parts(text_ptr, text_len) };
    let Ok(text) = std::str::from_utf8(bytes) else {
        OUT_LEN.store(0, Ordering::Relaxed);
        return std::ptr::null_mut();
    };
    let params: &[f32] = if params_ptr.is_null() || params_len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(params_ptr, params_len) }
    };
    match synth(text, flags, sample_rate, params) {
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
        let s = synth("coi munje", 0, SR, &[]).expect("synth ok");
        assert!(!s.is_empty());
        assert!(s.iter().all(|x| x.is_finite()));
    }

    #[test]
    fn flat_and_prosodic_differ() {
        let flat = synth("coi munje", FLAG_FLAT, SR, &[]).unwrap();
        let prosodic = synth("coi munje", 0, SR, &[]).unwrap();
        assert_ne!(flat, prosodic, "prosody must change the samples");
    }

    #[test]
    fn empty_text_errors() {
        assert!(matches!(synth("", 0, SR, &[]), Err(CompileError::Empty)));
    }

    #[test]
    fn dotside_flag_changes_output() {
        // "coi la djan": la-family exempts djan from the pre-cmevla pause;
        // --dotside drops that exemption, so the schedule (and audio) differ.
        let plain = synth("coi la djan", 0, SR, &[]).unwrap();
        let dotside = synth("coi la djan", FLAG_DOTSIDE, SR, &[]).unwrap();
        assert_ne!(plain, dotside);
    }

    #[test]
    fn empty_params_reproduce_defaults() {
        let default_layout = [120.0, 95.0, 1.5, 20.0, 1.2, 25.0, 1.0];
        assert_eq!(
            synth("coi munje", 0, SR, &[]).unwrap(),
            synth("coi munje", 0, SR, &default_layout).unwrap(),
            "an empty block and the default layout must render identically"
        );
    }

    #[test]
    fn params_change_output() {
        // A non-default rate (index 6) must change the rendered length.
        let base = synth("mi tavla do", 0, SR, &[]).unwrap();
        let mut faster = [120.0, 95.0, 1.5, 20.0, 1.2, 25.0, 1.0];
        faster[6] = 2.0; // rate 2x
        let fast = synth("mi tavla do", 0, SR, &faster).unwrap();
        assert!(
            fast.len() < base.len() * 3 / 4,
            "rate 2x should roughly halve the sample count ({} vs {})",
            fast.len(),
            base.len()
        );
    }
}
