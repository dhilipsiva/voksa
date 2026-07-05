//! voksa browser adapter. The shipping surface is a raw C-ABI with NO
//! wasm-bindgen runtime, so the compiled module declares zero imports and an
//! AudioWorklet can instantiate it synchronously with
//! `new WebAssembly.Instance(module, {})`: a host worklet marshals text into
//! wasm memory, calls [`voksa_render`], copies the samples out, and plays them
//! 128 frames at a time. This C-ABI is the SECONDARY embedding path (non-Rust
//! hosts); the primary path is the `voksa-console` Dioxus component, which
//! links this crate as a plain rlib (ADR 0003). See
//! `docs/handoff-dhilipsiva-dev.md`.

#![warn(missing_docs)]

use std::sync::atomic::{AtomicUsize, Ordering};

use voksa_core::attitudinal::{AttitudinalTable, Deviation};
use voksa_core::compiler::{CompileError, CompileOptions};
use voksa_core::phonemes::VoiceTable;
use voksa_core::prosody::ProsodyOptions;
use voksa_engine_klattsch::{render_utterance, render_utterance_expressive};

/// `flags` bit: disable prosody, mirroring the CLI `--flat` (the `flags`
/// default 0 = prosodic).
pub const FLAG_FLAT: u32 = 0x1;
/// `flags` bit: xu terminal rise (CLI `--xu`).
pub const FLAG_XU: u32 = 0x2;
/// `flags` bit: leading pause before every cmevla (CLI `--dotside`).
pub const FLAG_DOTSIDE: u32 = 0x4;
/// `flags` bit: epenthetic buffer vowels (CLI `--buffer`).
pub const FLAG_BUFFER: u32 = 0x8;

/// Count of f32 prosody knobs in the fixed layout the demo + CLI share:
/// `[declination_start_hz, declination_end_hz, stress_duration_factor,
/// stress_f0_excursion_hz, stress_amp_factor, xu_rise_hz, rate]`.
pub const PARAM_COUNT: usize = 7;

/// Count of f32 attitudinal knobs appended after the prosody block (D2a):
/// 7 kinds ([`voksa_core::attitudinal::AttitudinalKind::ALL`] order) × 8
/// fields ([`voksa_core::attitudinal::Deviation::to_array`] order).
pub const ATTITUDINAL_PARAM_COUNT: usize = 56;

/// Offset of the per-phoneme voice section in the f32 block (D2b).
pub const VOICE_PARAM_OFF: usize = PARAM_COUNT + ATTITUDINAL_PARAM_COUNT;

/// Count of f32 per-phoneme voice knobs appended after the attitudinal block
/// (D2b): the [`voksa_core::phonemes::VoiceTable`] flat layout (its
/// `to_array` doc comment is the normative ordering).
pub const VOICE_PARAM_COUNT: usize = VoiceTable::FIELDS;

/// Offset of the Phase-11 naturalness section (appended AFTER the voice table
/// — the layout is frozen append-only).
pub const NATURALNESS_PARAM_OFF: usize = VOICE_PARAM_OFF + VOICE_PARAM_COUNT;

/// Count of f32 naturalness knobs (P11): `[flutter, breath_aspiration,
/// baseline_oq_delta, baseline_tilt_delta, micro_f0_hz, obstruent_f0_hz,
/// final_lengthen, cluster_shorten, undershoot]` — the ProsodyOptions
/// naturalness block, in declaration order.
pub const NATURALNESS_PARAM_COUNT: usize = 9;

/// Full f32 block: prosody, then attitudinals, then the per-phoneme voice
/// table, then the naturalness knobs. Shorter blocks default the rest, so
/// 7-float (demo-basic), 63-float (demo-attitudinal), and 440-float
/// (demo-advanced) blocks stay valid forever.
pub const FULL_PARAM_COUNT: usize = NATURALNESS_PARAM_OFF + NATURALNESS_PARAM_COUNT;

/// Build [`ProsodyOptions`] from the flag bits + the f32 param block (fixed
/// order). Missing or non-finite entries fall back to the defaults, so an empty
/// slice reproduces `voksa_render` and the layout is forward-compatible.
fn prosody_from(flags: u32, params: &[f32]) -> ProsodyOptions {
    let d = ProsodyOptions::default();
    let g = |i: usize, fallback: f32| {
        params
            .get(i)
            .copied()
            .filter(|v| v.is_finite())
            .unwrap_or(fallback)
    };
    ProsodyOptions {
        xu_rise: flags & FLAG_XU != 0,
        declination_start_hz: g(0, d.declination_start_hz),
        declination_end_hz: g(1, d.declination_end_hz),
        stress_duration_factor: g(2, d.stress_duration_factor),
        stress_f0_excursion_hz: g(3, d.stress_f0_excursion_hz),
        stress_amp_factor: g(4, d.stress_amp_factor),
        xu_rise_hz: g(5, d.xu_rise_hz),
        rate: g(6, d.rate),
        // Phase-11 naturalness knobs cross at [NATURALNESS_PARAM_OFF..)
        // (appended after the voice table — the layout is frozen append-only).
        flutter: g(NATURALNESS_PARAM_OFF, d.flutter),
        breath_aspiration: g(NATURALNESS_PARAM_OFF + 1, d.breath_aspiration),
        baseline_oq_delta: g(NATURALNESS_PARAM_OFF + 2, d.baseline_oq_delta),
        baseline_tilt_delta: g(NATURALNESS_PARAM_OFF + 3, d.baseline_tilt_delta),
        micro_f0_hz: g(NATURALNESS_PARAM_OFF + 4, d.micro_f0_hz),
        obstruent_f0_hz: g(NATURALNESS_PARAM_OFF + 5, d.obstruent_f0_hz),
        final_lengthen: g(NATURALNESS_PARAM_OFF + 6, d.final_lengthen),
        cluster_shorten: g(NATURALNESS_PARAM_OFF + 7, d.cluster_shorten),
        undershoot: g(NATURALNESS_PARAM_OFF + 8, d.undershoot),
    }
}

/// Build the runtime [`AttitudinalTable`] from the f32 block's attitudinal
/// section (`params[PARAM_COUNT..FULL_PARAM_COUNT]`). Missing or non-finite
/// entries fall back to the pinned defaults, so a 7-float (demo-basic) or
/// empty block reproduces `AttitudinalTable::default()` exactly.
fn attitudinal_from(params: &[f32]) -> AttitudinalTable {
    let mut table = AttitudinalTable::default();
    for (k, dev) in table.deviations.iter_mut().enumerate() {
        let base = PARAM_COUNT + k * Deviation::FIELDS;
        let mut fields = dev.to_array();
        for (f, slot) in fields.iter_mut().enumerate() {
            if let Some(v) = params.get(base + f).copied().filter(|v| v.is_finite()) {
                *slot = v;
            }
        }
        *dev = Deviation::from_array(fields);
    }
    table
}

/// Build the runtime [`VoiceTable`] from the f32 block's per-phoneme section
/// (`params[VOICE_PARAM_OFF..FULL_PARAM_COUNT]`). Missing or non-finite
/// entries fall back to the pinned defaults, so 7-/63-float and empty blocks
/// reproduce `VoiceTable::default()` exactly.
fn voice_table_from(params: &[f32]) -> VoiceTable {
    let mut a = VoiceTable::default().to_array();
    for (i, slot) in a.iter_mut().enumerate() {
        if let Some(v) = params
            .get(VOICE_PARAM_OFF + i)
            .copied()
            .filter(|v| v.is_finite())
        {
            *slot = v;
        }
    }
    VoiceTable::from_array(a)
}

/// Render Lojban `text` to mono f32 PCM at `sample_rate`. `params` is the f32
/// block: prosody ([`PARAM_COUNT`]) then attitudinals
/// ([`ATTITUDINAL_PARAM_COUNT`]); shorter blocks (or empty) default the rest.
/// Shared by the C-ABI exports + tests.
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
        render_utterance_expressive(
            text,
            &opts,
            &prosody_from(flags, params),
            &attitudinal_from(params),
            &voice_table_from(params),
            sample_rate,
        )
    }
}

/// Phonetic transcription of `text` under the flag bits (dotside/buffer change
/// it; flat/xu don't) — the syllable/stress/pause line the demo shows so the
/// community can spot wrong phonetics. Shared by [`voksa_transcribe`] + tests.
pub fn transcription(text: &str, flags: u32) -> Result<String, CompileError> {
    let opts = CompileOptions {
        dotside: flags & FLAG_DOTSIDE != 0,
        buffer: flags & FLAG_BUFFER != 0,
    };
    voksa_core::transcribe::transcribe(text, &opts)
}

/// The full default f32 param block — prosody, then the pinned attitudinal
/// vectors, then the pinned voice table, in the canonical layout. The demo
/// reads this FROM the wasm (via [`voksa_default_params`]) to seed its
/// sliders, so the UI's defaults can never drift from the engine's tables.
pub fn default_params() -> Vec<f32> {
    let mut out = Vec::with_capacity(FULL_PARAM_COUNT);
    let p = ProsodyOptions::default();
    out.extend_from_slice(&[
        p.declination_start_hz,
        p.declination_end_hz,
        p.stress_duration_factor,
        p.stress_f0_excursion_hz,
        p.stress_amp_factor,
        p.xu_rise_hz,
        p.rate,
    ]);
    let atts = AttitudinalTable::default();
    for k in voksa_core::attitudinal::AttitudinalKind::ALL {
        out.extend_from_slice(&atts.get(k).to_array());
    }
    out.extend_from_slice(&VoiceTable::default().to_array());
    out.extend_from_slice(&[
        p.flutter,
        p.breath_aspiration,
        p.baseline_oq_delta,
        p.baseline_tilt_delta,
        p.micro_f0_hz,
        p.obstruent_f0_hz,
        p.final_lengthen,
        p.cluster_shorten,
        p.undershoot,
    ]);
    debug_assert_eq!(out.len(), FULL_PARAM_COUNT);
    out
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

/// Phonetic transcription of `text` (UTF-8 at `text_ptr`/`text_len`) under the
/// flag bits: returns a pointer to UTF-8 bytes (length via [`voksa_out_len`];
/// free with [`voksa_dealloc`] passing that length). Null (length 0) on a
/// compile error or invalid UTF-8.
///
/// # Safety
/// `text_ptr`/`text_len` must describe a readable byte range in wasm memory.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn voksa_transcribe(
    text_ptr: *const u8,
    text_len: usize,
    flags: u32,
) -> *mut u8 {
    let bytes = unsafe { std::slice::from_raw_parts(text_ptr, text_len) };
    let Ok(text) = std::str::from_utf8(bytes) else {
        OUT_LEN.store(0, Ordering::Relaxed);
        return std::ptr::null_mut();
    };
    match transcription(text, flags) {
        Ok(s) => {
            let mut b = s.into_bytes();
            b.shrink_to_fit(); // guarantee capacity == len for the free
            let ptr = b.as_mut_ptr();
            OUT_LEN.store(b.len(), Ordering::Relaxed);
            std::mem::forget(b);
            ptr
        }
        Err(_) => {
            OUT_LEN.store(0, Ordering::Relaxed);
            std::ptr::null_mut()
        }
    }
}

/// Pointer to the full default f32 param block (length via [`voksa_out_len`],
/// == `FULL_PARAM_COUNT`); free with [`voksa_free_f32`]. The demo instantiates
/// the module on the MAIN thread too (zero imports make that a one-liner) and
/// seeds its sliders from this, so UI defaults are the engine's, always.
#[unsafe(no_mangle)]
pub extern "C" fn voksa_default_params() -> *mut f32 {
    let mut v = default_params();
    v.shrink_to_fit();
    let ptr = v.as_mut_ptr();
    OUT_LEN.store(v.len(), Ordering::Relaxed);
    std::mem::forget(v);
    ptr
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

    /// The full 449-float block at its default values (prosody + pinned
    /// attitudinal vectors + pinned voice table + naturalness knobs, in the
    /// canonical layout).
    fn default_full_block() -> Vec<f32> {
        default_params()
    }

    #[test]
    fn full_default_block_reproduces_defaults() {
        // A full block carrying exactly the default values must render
        // byte-identically to the empty block (layout correctness guard).
        assert_eq!(
            synth("coi munje .ui", 0, SR, &[]).unwrap(),
            synth("coi munje .ui", 0, SR, &default_full_block()).unwrap(),
        );
    }

    #[test]
    fn attitudinal_params_change_output() {
        // Bumping Joy's f0_mean_hz (first attitudinal slot) must change a .ui
        // render — the advanced tab's knobs reach the engine.
        let mut tuned = default_full_block();
        tuned[PARAM_COUNT] = 60.0;
        assert_ne!(
            synth("coi munje .ui", 0, SR, &[]).unwrap(),
            synth("coi munje .ui", 0, SR, &tuned).unwrap(),
            "a tuned Joy vector must change the .ui render"
        );
    }

    #[test]
    fn attitudinal_params_inert_without_marker() {
        // Modal text carries no attitudinal scope, so table changes are inert.
        let mut tuned = default_full_block();
        tuned[PARAM_COUNT] = 60.0;
        assert_eq!(
            synth("coi munje", 0, SR, &[]).unwrap(),
            synth("coi munje", 0, SR, &tuned).unwrap(),
        );
    }

    #[test]
    fn basic_seven_float_block_stays_valid() {
        // demo-basic clients send exactly 7 floats; that must keep working
        // (attitudinals default) even for attitudinal-bearing text.
        let basic = [120.0, 95.0, 1.5, 20.0, 1.2, 25.0, 1.0];
        assert_eq!(
            synth("coi munje .ui", 0, SR, &[]).unwrap(),
            synth("coi munje .ui", 0, SR, &basic).unwrap(),
        );
    }

    #[test]
    fn attitudinal_63_block_stays_valid() {
        // demo-attitudinal clients send exactly 63 floats; the voice section
        // must default for them.
        let mut block = vec![120.0, 95.0, 1.5, 20.0, 1.2, 25.0, 1.0];
        for k in voksa_core::attitudinal::AttitudinalKind::ALL {
            block.extend(k.deviation().to_array());
        }
        assert_eq!(block.len(), VOICE_PARAM_OFF);
        assert_eq!(
            synth("coi munje .ui", 0, SR, &[]).unwrap(),
            synth("coi munje .ui", 0, SR, &block).unwrap(),
        );
    }

    #[test]
    fn voice_params_change_output() {
        // Bumping vowel-a F1 (the first voice slot, index VOICE_PARAM_OFF) must
        // change an a-bearing render — the phoneme knobs reach the engine.
        let mut tuned = default_full_block();
        tuned[VOICE_PARAM_OFF] = 900.0;
        assert_ne!(
            synth("mi klama", 0, SR, &[]).unwrap(),
            synth("mi klama", 0, SR, &tuned).unwrap(),
            "a tuned /a/ F1 must change the render"
        );
    }

    #[test]
    fn transcription_renders_stress_and_flags() {
        assert_eq!(transcription("coi munje", 0).unwrap(), "coi MUN.je");
        // dotside + buffer flags must reach the transcription.
        assert_eq!(
            transcription("la djan. cu klama", FLAG_DOTSIDE).unwrap(),
            "la ‖ DJAN ‖ cu KLA.ma"
        );
        assert_eq!(
            transcription("le zdani", FLAG_BUFFER).unwrap(),
            "le Z(ɪ)DA.ni"
        );
    }

    #[test]
    fn naturalness_params_at_440_change_output() {
        // The Phase-11 knobs cross at [NATURALNESS_PARAM_OFF..): bumping
        // final_lengthen (index 440+6) must lengthen the render.
        let base = synth("coi munje", 0, SR, &[]).unwrap();
        let mut tuned = default_full_block();
        tuned[NATURALNESS_PARAM_OFF + 6] = 2.0;
        let long = synth("coi munje", 0, SR, &tuned).unwrap();
        assert!(
            long.len() > base.len() + SR as usize / 100,
            "final_lengthen 2.0 must lengthen the render: {} vs {}",
            long.len(),
            base.len()
        );
    }

    #[test]
    fn voice_440_block_stays_valid() {
        // demo-advanced clients send exactly 440 floats; the naturalness
        // section must default for them.
        let block: Vec<f32> = default_full_block()[..NATURALNESS_PARAM_OFF].to_vec();
        assert_eq!(block.len(), 440);
        assert_eq!(
            synth("coi munje .ui", 0, SR, &[]).unwrap(),
            synth("coi munje .ui", 0, SR, &block).unwrap(),
        );
    }

    #[test]
    fn default_params_block_is_full_and_pinned() {
        // The wasm-exported defaults must BE the canonical default block: the
        // demo seeds its sliders from this, so any drift here is UI drift.
        let d = default_params();
        assert_eq!(d.len(), FULL_PARAM_COUNT);
        assert_eq!(
            d,
            default_full_block(),
            "defaults must equal the composed pinned tables"
        );
        assert_eq!(
            synth("coi munje .ui", 0, SR, &[]).unwrap(),
            synth("coi munje .ui", 0, SR, &d).unwrap(),
            "rendering with the exported defaults must equal the empty block"
        );
    }
}
