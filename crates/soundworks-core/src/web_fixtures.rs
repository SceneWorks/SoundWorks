//! Single source of truth for the web-preview fallback fixtures (F-012).
//!
//! [`web_fixtures_json`] serializes every Rust reference fixture the web preview
//! falls back to, keyed by its TypeScript export name. The `gen_web_fixtures`
//! bin writes it to `apps/web/src/appData.generated.json` (consumed by a thin
//! `appData.ts` shim), and `tests/web_fixtures_parity.rs` asserts the committed
//! JSON matches — so the TS fallback can never silently drift from the Rust
//! fixtures again.

use crate::{
    AppOverview, AssetLibraryOverview, CompositionEditorOverview, DeviceInventory,
    ExportWorkflowOverview, ModelEvaluationCatalog, ModelManagerOverview, MvpValidationOverview,
    ReviewWorkspaceOverview, RightsSafetyOverview, RuntimeJobStore, RuntimeOverview,
    RuntimePackagingPolicy, SamplesStudioOverview, SfxStudioOverview, SongStudioOverview,
    TtsStudioOverview, VideoToAudioOverview, VoiceLabOverview, WorkspaceOverview,
};
use std::path::PathBuf;

/// Pretty-printed JSON object of all web-preview fallbacks (trailing newline
/// included), keyed by TS export name.
pub fn web_fixtures_json() -> String {
    // Build the model manager against a fixed, tilde-relative cache root rather
    // than the real `$HOME`-based default, so the committed fixtures are
    // deterministic across machines/CI and never embed an absolute home path
    // (the model cache for these demo candidates is treated as not-yet-downloaded).
    let model_manager = ModelManagerOverview::from_catalog(
        &ModelEvaluationCatalog::reference(),
        PathBuf::from("~/Library/Application Support/SoundWorks/models"),
    );

    // Empty runtime store so the generated runtime overview reports `jobs: []`,
    // matching the web fallback (mirrors the desktop `runtime_overview` command
    // running against a fresh store).
    let runtime_store =
        RuntimeJobStore::new(std::env::temp_dir().join("soundworks-gen-web-fixtures"));
    let runtime = RuntimeOverview::from_model_manager(
        &model_manager,
        &DeviceInventory::reference_mac(),
        RuntimePackagingPolicy::shipped_desktop(),
        &runtime_store,
    );

    let data = serde_json::json!({
        "fallbackOverview": AppOverview::baseline(),
        "fallbackRuntime": runtime,
        "fallbackWorkspace": WorkspaceOverview::reference().expect("workspace reference"),
        "fallbackAssetLibrary": AssetLibraryOverview::reference().expect("asset library reference"),
        "fallbackCompositionEditor": CompositionEditorOverview::reference(),
        "fallbackExportWorkflow": ExportWorkflowOverview::reference(),
        "fallbackMvpValidation": MvpValidationOverview::reference(),
        "fallbackModelManager": model_manager,
        "fallbackRightsSafety": RightsSafetyOverview::reference(),
        "fallbackReviewWorkspace": ReviewWorkspaceOverview::reference().expect("review reference"),
        "fallbackTtsStudio": TtsStudioOverview::reference().expect("tts reference"),
        "fallbackVoiceLab": VoiceLabOverview::reference().expect("voice lab reference"),
        "fallbackSfxStudio": SfxStudioOverview::reference().expect("sfx reference"),
        "fallbackSamplesStudio": SamplesStudioOverview::reference().expect("samples reference"),
        "fallbackSongStudio": SongStudioOverview::reference().expect("song reference"),
        "fallbackVideoToAudio": VideoToAudioOverview::reference().expect("video reference"),
    });

    format!(
        "{}\n",
        serde_json::to_string_pretty(&data).expect("serialize fixtures")
    )
}
