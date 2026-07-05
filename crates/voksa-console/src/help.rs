//! The help-copy registry: every `?` affordance in the console resolves its
//! text here by key. The registry is `include_str!`-embedded
//! (`assets/help-text.json`, parsed once) so it is natively testable and can
//! never drift from an asset path. A missing or empty entry yields the
//! `// help pending — <key>` fallback, so a gap is *visible* rather than
//! silent — and a native test (`tests/help.rs`) forbids gaps in the shipped
//! build. Field keys are SHARED across phonemes/emotions (`vt.fields.f1_hz`
//! serves every phoneme's F1, `att.fields.f0_mean_hz` every emotion's pitch
//! shift), so ~70 entries cover all 449 rows.

use std::collections::HashMap;
use std::sync::OnceLock;

/// The parsed registry (key → copy), built once from the embedded JSON.
fn registry() -> &'static HashMap<String, String> {
    static REG: OnceLock<HashMap<String, String>> = OnceLock::new();
    REG.get_or_init(|| serde_json::from_str(crate::assets::HELP_TEXT_JSON).unwrap_or_default())
}

/// The help copy for `key`, or the `// help pending — <key>` fallback when the
/// key is absent or its entry is empty/whitespace.
pub fn help_for(key: &str) -> String {
    match registry().get(key) {
        Some(s) if !s.trim().is_empty() => s.clone(),
        _ => format!("// help pending — {key}"),
    }
}
