//! Config JSON export/load — the community sharing loop. The schema is the
//! CLI's (`voksa --config`): flat text/flags/knobs always, attitudinals and
//! phonemes DELTA-ONLY (stops nest `closure`/`burst`), plus the
//! `phonetics`/`notes`/`sampleRate`/`voksaVersion` stamps. Load = REPLACE.

use super::descriptor::Descriptors;

/// The compile/prosody flag set (mirrors the CLI flags + wasm bit layout).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Flags {
    /// No prosody at all (`--flat`; disables the voice-shaping controls).
    pub flat: bool,
    /// Terminal question rise (`--xu`).
    pub xu: bool,
    /// Leading pause before every cmevla (`--dotside`).
    pub dotside: bool,
    /// Epenthetic buffer vowels (`--buffer`).
    pub buffer: bool,
}

impl Flags {
    /// The wasm `flags` bit word (flat 0x1, xu 0x2, dotside 0x4, buffer 0x8).
    pub fn bits(self) -> u32 {
        let mut b = 0;
        if self.flat {
            b |= voksa_web::FLAG_FLAT;
        }
        if self.xu {
            b |= voksa_web::FLAG_XU;
        }
        if self.dotside {
            b |= voksa_web::FLAG_DOTSIDE;
        }
        if self.buffer {
            b |= voksa_web::FLAG_BUFFER;
        }
        b
    }
}

/// Everything an export stamps into the config JSON.
#[derive(Debug, Clone, Copy)]
pub struct ExportInputs<'a> {
    /// The full 449-value parameter snapshot.
    pub values: &'a [f32],
    /// The utterance text.
    pub text: &'a str,
    /// The flag set.
    pub flags: Flags,
    /// The tuner's notes (travel inside the JSON).
    pub notes: &'a str,
    /// The phonetic transcription of `text` (what the tuner was looking at).
    pub phonetics: &'a str,
    /// The playback sample rate.
    pub sample_rate: u32,
}

/// Export the config JSON (pretty-printed). Knobs export as flat named
/// fields ALWAYS; attitudinals/phonemes export only f32-dirty values.
pub fn export(desc: &Descriptors, inp: &ExportInputs<'_>) -> String {
    use super::descriptor::{ATT_FIELDS, ATT_KINDS, ItemKind, KNOBS, VOICE_ITEMS, VT_FIELDS};
    use super::logic::is_dirty;
    use super::path::{ATT_FIELD_COUNT, KNOB_COUNT, Path, VOICE_ITEM_COUNT};
    use serde_json::{Map, Value};

    let num = |v: f32| Value::from(v as f64);
    let mut root = Map::new();
    root.insert("text".into(), inp.text.into());
    root.insert("flat".into(), inp.flags.flat.into());
    root.insert("xu".into(), inp.flags.xu.into());
    root.insert("dotside".into(), inp.flags.dotside.into());
    root.insert("buffer".into(), inp.flags.buffer.into());
    for (k, spec) in KNOBS.iter().enumerate().take(KNOB_COUNT) {
        root.insert(spec.key.into(), num(inp.values[desc.knob_index(k)]));
    }

    let mut atts = Map::new();
    for (kind, att) in ATT_KINDS.iter().enumerate() {
        let mut fields = Map::new();
        for (field, spec) in ATT_FIELDS.iter().enumerate().take(ATT_FIELD_COUNT) {
            let idx = Path::Att {
                kind: kind as u8,
                field: field as u8,
            }
            .flat_index();
            if is_dirty(desc.get(idx), inp.values[idx]) {
                fields.insert(spec.key.into(), num(inp.values[idx]));
            }
        }
        if !fields.is_empty() {
            atts.insert(att.key.into(), Value::Object(fields));
        }
    }
    if !atts.is_empty() {
        root.insert("attitudinals".into(), Value::Object(atts));
    }

    let mut phonemes = Map::new();
    for (item, vi) in VOICE_ITEMS.iter().enumerate().take(VOICE_ITEM_COUNT) {
        let base = desc.voice_item_range(item).start;
        let dirty = |slot: usize| {
            let idx = base + slot;
            is_dirty(desc.get(idx), inp.values[idx]).then(|| num(inp.values[idx]))
        };
        let mut obj = Map::new();
        match vi.kind {
            ItemKind::Steady => {
                for (slot, spec) in VT_FIELDS.iter().enumerate() {
                    if let Some(v) = dirty(slot) {
                        obj.insert(spec.key.into(), v);
                    }
                }
                if let Some(v) = dirty(11) {
                    obj.insert("dur_ms".into(), v);
                }
            }
            ItemKind::Dur => {
                if let Some(v) = dirty(0) {
                    obj.insert("dur_ms".into(), v);
                }
            }
            ItemKind::Stop => {
                for (phase, off) in [("closure", 0usize), ("burst", 11)] {
                    let mut block = Map::new();
                    for (i, spec) in VT_FIELDS.iter().enumerate() {
                        if let Some(v) = dirty(off + i) {
                            block.insert(spec.key.into(), v);
                        }
                    }
                    if !block.is_empty() {
                        obj.insert(phase.into(), Value::Object(block));
                    }
                }
                if let Some(v) = dirty(22) {
                    obj.insert("closure_ms".into(), v);
                }
                if let Some(v) = dirty(23) {
                    obj.insert("burst_ms".into(), v);
                }
            }
        }
        if !obj.is_empty() {
            phonemes.insert(vi.key.into(), Value::Object(obj));
        }
    }
    if !phonemes.is_empty() {
        root.insert("phonemes".into(), Value::Object(phonemes));
    }

    root.insert("phonetics".into(), inp.phonetics.into());
    root.insert("notes".into(), inp.notes.into());
    root.insert("sampleRate".into(), inp.sample_rate.into());
    root.insert("voksaVersion".into(), env!("CARGO_PKG_VERSION").into());
    serde_json::to_string_pretty(&Value::Object(root)).expect("maps of primitives serialize")
}

/// A parsed config, resolved to REPLACE semantics: a full 449-value block
/// (engine defaults overlaid with the config's keys) plus text/flags/notes.
#[derive(Debug, Clone, PartialEq)]
pub struct LoadPlan {
    /// The full parameter block to apply (always 449 values).
    pub values: Vec<f32>,
    /// The utterance text (`None` = keep the current text).
    pub text: Option<String>,
    /// The flag set (absent keys = false, like the CLI).
    pub flags: Flags,
    /// The notes field ("" when absent).
    pub notes: String,
}

/// Parse a config JSON into a [`LoadPlan`]. Unknown fields are ignored
/// (CLI semantics); malformed JSON or wrongly-typed known values error
/// without touching any state.
pub fn load(desc: &Descriptors, json: &str) -> Result<LoadPlan, String> {
    use super::descriptor::{ATT_FIELDS, ATT_KINDS, ItemKind, KNOBS, VOICE_ITEMS, VT_FIELDS};
    use super::path::Path;
    use serde_json::Value;

    let doc: Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
    let obj = doc.as_object().ok_or("config must be a JSON object")?;

    let num = |v: &Value, ctx: &str| -> Result<f32, String> {
        v.as_f64()
            .map(|f| f as f32)
            .ok_or_else(|| format!("{ctx} must be a number"))
    };
    let flag = |key: &str| -> Result<bool, String> {
        match obj.get(key) {
            None => Ok(false),
            Some(v) => v
                .as_bool()
                .ok_or_else(|| format!("{key} must be a boolean")),
        }
    };
    // A string field errors on present-but-not-a-string (like the knob/flag
    // helpers), so a config the console accepts also parses through voksa-cli's
    // `Config` (whose `text`/`notes` are `String`). Absent = `None`.
    let str_field = |key: &str| -> Result<Option<String>, String> {
        match obj.get(key) {
            None => Ok(None),
            Some(v) => Ok(Some(
                v.as_str()
                    .ok_or_else(|| format!("{key} must be a string"))?
                    .to_string(),
            )),
        }
    };

    let mut values: Vec<f32> = (0..desc.len()).map(|i| desc.get(i).default).collect();

    for (k, spec) in KNOBS.iter().enumerate() {
        if let Some(v) = obj.get(spec.key) {
            values[desc.knob_index(k)] = num(v, spec.key)?;
        }
    }

    if let Some(atts) = obj.get("attitudinals") {
        let atts = atts.as_object().ok_or("attitudinals must be an object")?;
        for (cmavo, fields) in atts {
            let Some(kind) = ATT_KINDS.iter().position(|a| a.key == cmavo) else {
                continue; // unknown cmavo keys are ignored, like the CLI
            };
            let fields = fields
                .as_object()
                .ok_or_else(|| format!("attitudinals.{cmavo} must be an object"))?;
            for (fk, fv) in fields {
                let Some(field) = ATT_FIELDS.iter().position(|f| f.key == fk) else {
                    continue;
                };
                let idx = Path::Att {
                    kind: kind as u8,
                    field: field as u8,
                }
                .flat_index();
                values[idx] = num(fv, fk)?;
            }
        }
    }

    if let Some(phonemes) = obj.get("phonemes") {
        let phonemes = phonemes.as_object().ok_or("phonemes must be an object")?;
        for (pkey, pval) in phonemes {
            let Some(item) = VOICE_ITEMS.iter().position(|i| i.key == pkey) else {
                continue;
            };
            let pobj = pval
                .as_object()
                .ok_or_else(|| format!("phonemes.{pkey} must be an object"))?;
            let base = desc.voice_item_range(item).start;
            let mut set = |slot: usize, v: &Value, ctx: &str| -> Result<(), String> {
                values[base + slot] = num(v, ctx)?;
                Ok(())
            };
            match VOICE_ITEMS[item].kind {
                ItemKind::Steady => {
                    for (fk, fv) in pobj {
                        if let Some(slot) = VT_FIELDS.iter().position(|f| f.key == fk) {
                            set(slot, fv, fk)?;
                        } else if fk == "dur_ms" {
                            set(11, fv, fk)?;
                        }
                    }
                }
                ItemKind::Dur => {
                    if let Some(v) = pobj.get("dur_ms") {
                        set(0, v, "dur_ms")?;
                    }
                }
                ItemKind::Stop => {
                    for (phase, off) in [("closure", 0usize), ("burst", 11)] {
                        if let Some(block) = pobj.get(phase) {
                            let block = block.as_object().ok_or_else(|| {
                                format!("phonemes.{pkey}.{phase} must be an object")
                            })?;
                            for (fk, fv) in block {
                                if let Some(i) = VT_FIELDS.iter().position(|f| f.key == fk) {
                                    set(off + i, fv, fk)?;
                                }
                            }
                        }
                    }
                    if let Some(v) = pobj.get("closure_ms") {
                        set(22, v, "closure_ms")?;
                    }
                    if let Some(v) = pobj.get("burst_ms") {
                        set(23, v, "burst_ms")?;
                    }
                }
            }
        }
    }

    Ok(LoadPlan {
        values,
        text: str_field("text")?,
        flags: Flags {
            flat: flag("flat")?,
            xu: flag("xu")?,
            dotside: flag("dotside")?,
            buffer: flag("buffer")?,
        },
        notes: str_field("notes")?.unwrap_or_default(),
    })
}
