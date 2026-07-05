//! Embedded data assets. JSON registries are `include_str!`-embedded (parsed
//! once, natively testable, immune to asset-path drift); fetchable assets
//! (stylesheet, logomark) join via manganis `asset!()` in the UI layer (C2).

/// The curated phonetic-coverage sentence set (design bundle, 18 entries).
/// Every entry is gated by a native test that synthesizes it.
pub const SENTENCES_JSON: &str = include_str!("../assets/sentences.json");

/// The help-copy registry the `?` popovers resolve by key. Empty string =
/// the console shows `// help pending — <key>`.
pub const HELP_TEXT_JSON: &str = include_str!("../assets/help-text.json");
