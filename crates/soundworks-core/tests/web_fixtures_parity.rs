//! Drift guard for F-012: the committed web-preview fallback JSON must stay in
//! sync with the Rust reference fixtures. If this fails, regenerate it with
//! `npm --prefix apps/web run gen:fixtures` (or
//! `cargo run -p soundworks-core --bin gen_web_fixtures -- apps/web/src/appData.generated.json`).

#[test]
fn committed_web_fixtures_match_rust_reference() {
    let committed_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../apps/web/src/appData.generated.json");
    let committed =
        std::fs::read_to_string(&committed_path).expect("read apps/web/src/appData.generated.json");
    let generated = soundworks_core::web_fixtures::web_fixtures_json();
    assert_eq!(
        committed, generated,
        "apps/web/src/appData.generated.json is stale — run `npm --prefix apps/web run gen:fixtures`"
    );
}
