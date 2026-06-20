//! Write `apps/web/src/appData.generated.json` from the Rust reference fixtures
//! so the web-preview fallback data has a single source of truth (F-012) instead
//! of a 6,800-line hand-maintained TypeScript clone.
//!
//! Run from the repo root:
//!   cargo run -p soundworks-core --bin gen_web_fixtures -- apps/web/src/appData.generated.json
//! or via `npm --prefix apps/web run gen:fixtures`.
//!
//! `tests/web_fixtures_parity.rs` asserts the committed JSON is up to date, so a
//! stale fixture fails the test suite rather than drifting silently.

fn main() {
    let out = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "apps/web/src/appData.generated.json".to_string());
    std::fs::write(&out, soundworks_core::web_fixtures::web_fixtures_json())
        .expect("write generated fixtures");
    eprintln!("wrote {out}");
}
