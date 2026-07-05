//! Release-version parity (P11 W4): the demo stamps `voksaVersion` into every
//! exported config JSON; if it drifts from the crate version, community bug
//! reports carry the wrong provenance. Native-only (`std::fs`).
#![cfg(not(target_arch = "wasm32"))]

#[test]
fn demo_version_stamp_matches_crate_version() {
    let html = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/www/index.html"))
        .expect("www/index.html");
    let needle = format!("c.voksaVersion = '{}';", env!("CARGO_PKG_VERSION"));
    assert!(
        html.contains(&needle),
        "www/index.html must stamp the crate version: expected `{needle}`"
    );
}
