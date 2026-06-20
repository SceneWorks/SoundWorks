use soundworks_core::{
    AppOverview, AssetLibraryOverview, CompositionEditorOverview, CreateProjectRequest,
    ExportLibraryItemRequest, ExportLibraryItemResult, ExportWorkflowOverview,
    ImportRuntimeArtifactRequest, JobStatus, LibraryMutationRequest, LibraryPlayback,
    ModelEvaluationCatalog, ModelManagerOperation, ModelManagerOverview, MvpValidationOverview,
    ProjectLibraryActionResult, ProjectLibraryStore, ProviderCatalog, ReviewEditResult,
    ReviewWorkspaceOverview, RightsSafetyOverview, RuntimeEngine, RuntimeJobArtifact,
    RuntimeJobRequest, RuntimeJobSnapshot, RuntimeJobStore, RuntimeOverview, SamplesStudioOverview,
    SaveReviewEditRequest, SfxStudioOverview, SongStudioOverview, TtsStudioOverview, UiPreferences,
    UiPreferencesStore, VideoToAudioOverview, VoiceLabOverview, WorkspaceOverview,
};
use std::sync::{Arc, Mutex};

/// Shared, Tauri-managed application state. The write lock serializes every
/// store-mutating command (F-005), preventing the lost-update / TOCTOU race on the
/// shared `workspace.json` that concurrent Tauri commands could otherwise cause.
/// The `engine` is the single process-wide runtime backend (F-006/F-019): one
/// Tokio runtime, a warm Kokoro cache, and the per-job cancellation registry,
/// reused by every queued job's background worker.
pub struct AppState {
    write_lock: Mutex<()>,
    engine: Arc<RuntimeEngine>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            write_lock: Mutex::new(()),
            engine: Arc::new(RuntimeEngine::new().expect("runtime engine initializes")),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// Acquire the write lock. A poisoned lock only means a prior command panicked
/// mid-operation; atomic writes keep the on-disk state consistent, so recover the
/// guard and continue rather than propagating the poison.
fn lock_writes(state: &AppState) -> std::sync::MutexGuard<'_, ()> {
    state
        .write_lock
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
}

#[tauri::command]
fn get_app_overview() -> AppOverview {
    app_overview()
}

#[tauri::command]
fn get_provider_catalog() -> ProviderCatalog {
    provider_catalog()
}

#[tauri::command]
fn get_workspace_overview() -> WorkspaceOverview {
    workspace_overview()
}

#[tauri::command]
fn get_asset_library_overview() -> AssetLibraryOverview {
    asset_library_overview()
}

#[tauri::command]
fn create_soundworks_project(
    state: tauri::State<AppState>,
    request: CreateProjectRequest,
) -> Result<ProjectLibraryActionResult, String> {
    let _guard = lock_writes(&state);
    create_project(request).map_err(|error| error.to_string())
}

#[tauri::command]
fn open_soundworks_project(
    state: tauri::State<AppState>,
    project_id: String,
) -> Result<ProjectLibraryActionResult, String> {
    let _guard = lock_writes(&state);
    open_project(project_id).map_err(|error| error.to_string())
}

#[tauri::command]
fn import_runtime_artifact_to_library(
    state: tauri::State<AppState>,
    request: ImportRuntimeArtifactRequest,
) -> Result<ProjectLibraryActionResult, String> {
    let _guard = lock_writes(&state);
    import_runtime_artifact(request).map_err(|error| error.to_string())
}

#[tauri::command]
fn mutate_library_item(
    state: tauri::State<AppState>,
    request: LibraryMutationRequest,
) -> Result<ProjectLibraryActionResult, String> {
    let _guard = lock_writes(&state);
    mutate_item(request).map_err(|error| error.to_string())
}

#[tauri::command]
fn get_library_playback(item_id: String) -> Result<LibraryPlayback, String> {
    library_playback(item_id).map_err(|error| error.to_string())
}

#[tauri::command]
fn save_review_edit(
    state: tauri::State<AppState>,
    request: SaveReviewEditRequest,
) -> Result<ReviewEditResult, String> {
    let _guard = lock_writes(&state);
    review_edit(request).map_err(|error| error.to_string())
}

#[tauri::command]
fn export_library_item(
    state: tauri::State<AppState>,
    request: ExportLibraryItemRequest,
) -> Result<ExportLibraryItemResult, String> {
    let _guard = lock_writes(&state);
    export_item(request).map_err(|error| error.to_string())
}

#[tauri::command]
fn get_export_workflow_overview() -> ExportWorkflowOverview {
    export_workflow_overview()
}

#[tauri::command]
fn get_composition_editor_overview() -> CompositionEditorOverview {
    composition_editor_overview()
}

#[tauri::command]
fn get_runtime_overview() -> RuntimeOverview {
    runtime_overview()
}

#[tauri::command]
fn enqueue_runtime_job(
    state: tauri::State<AppState>,
    request: RuntimeJobRequest,
) -> Result<RuntimeJobSnapshot, String> {
    let _guard = lock_writes(&state);
    enqueue_job(state.engine.clone(), request).map_err(|error| error.to_string())
}

#[tauri::command]
fn cancel_runtime_job(
    state: tauri::State<AppState>,
    job_id: String,
) -> Result<Option<RuntimeJobSnapshot>, String> {
    let _guard = lock_writes(&state);
    cancel_job(&state.engine, job_id).map_err(|error| error.to_string())
}

#[tauri::command]
fn retry_runtime_job(
    state: tauri::State<AppState>,
    job_id: String,
) -> Result<Option<RuntimeJobSnapshot>, String> {
    let _guard = lock_writes(&state);
    retry_job(state.engine.clone(), job_id).map_err(|error| error.to_string())
}

#[tauri::command]
fn get_runtime_job_artifacts(job_id: String) -> Result<Vec<RuntimeJobArtifact>, String> {
    runtime_job_artifacts(job_id).map_err(|error| error.to_string())
}

#[tauri::command]
fn get_runtime_job(job_id: String) -> Result<Option<RuntimeJobSnapshot>, String> {
    runtime_job(job_id).map_err(|error| error.to_string())
}

#[tauri::command]
fn get_model_evaluation_catalog() -> ModelEvaluationCatalog {
    model_evaluation_catalog()
}

#[tauri::command]
fn get_model_manager_overview() -> ModelManagerOverview {
    model_manager_overview()
}

#[tauri::command]
fn install_model_candidate(candidate_id: String) -> ModelManagerOperation {
    install_candidate(candidate_id)
}

#[tauri::command]
fn revalidate_model_candidate(candidate_id: String) -> ModelManagerOperation {
    revalidate_candidate(candidate_id)
}

#[tauri::command]
fn get_mvp_validation_overview() -> MvpValidationOverview {
    mvp_validation_overview()
}

#[tauri::command]
fn get_tts_studio_overview() -> TtsStudioOverview {
    tts_studio_overview()
}

#[tauri::command]
fn get_voice_lab_overview() -> VoiceLabOverview {
    voice_lab_overview()
}

#[tauri::command]
fn get_sfx_studio_overview() -> SfxStudioOverview {
    sfx_studio_overview()
}

#[tauri::command]
fn get_samples_studio_overview() -> SamplesStudioOverview {
    samples_studio_overview()
}

#[tauri::command]
fn get_song_studio_overview() -> SongStudioOverview {
    song_studio_overview()
}

#[tauri::command]
fn get_review_workspace_overview() -> ReviewWorkspaceOverview {
    review_workspace_overview()
}

#[tauri::command]
fn get_rights_safety_overview() -> RightsSafetyOverview {
    rights_safety_overview()
}

#[tauri::command]
fn get_video_to_audio_overview() -> VideoToAudioOverview {
    video_to_audio_overview()
}

/// DR-01: read the durable theme/accent preferences. Returns defaults (empty) when
/// no preference has been persisted yet.
#[tauri::command]
fn get_ui_preferences() -> UiPreferences {
    UiPreferencesStore::default().load()
}

/// DR-01: persist a partial theme/accent update. Merges over the stored copy under
/// the write lock so concurrent toggles cannot lose an update.
#[tauri::command]
fn set_ui_preferences(
    state: tauri::State<AppState>,
    preferences: UiPreferences,
) -> Result<UiPreferences, String> {
    let _guard = lock_writes(&state);
    UiPreferencesStore::default()
        .merge(preferences)
        .map_err(|error| error.to_string())
}

pub fn app_overview() -> AppOverview {
    let mut overview = AppOverview::baseline();
    let store = ProjectLibraryStore::default();
    if let Ok(workspace) = store.workspace_overview() {
        overview.workspace = soundworks_core::WorkspaceSummary::from_overview(&workspace);
    }
    if let Ok(library) = store.asset_library_overview(None) {
        overview.asset_library = soundworks_core::AssetLibrarySummary::from_overview(&library);
    }
    overview
}

pub fn provider_catalog() -> ProviderCatalog {
    ProviderCatalog::reference()
}

pub fn workspace_overview() -> WorkspaceOverview {
    ProjectLibraryStore::default()
        .workspace_overview()
        .unwrap_or_else(|_| WorkspaceOverview::reference().expect("reference workspace is valid"))
}

pub fn asset_library_overview() -> AssetLibraryOverview {
    ProjectLibraryStore::default()
        .asset_library_overview(None)
        .unwrap_or_else(|_| {
            AssetLibraryOverview::reference().expect("reference Asset Library is valid")
        })
}

pub fn create_project(
    request: CreateProjectRequest,
) -> std::io::Result<ProjectLibraryActionResult> {
    ProjectLibraryStore::default().create_project(request)
}

pub fn open_project(project_id: String) -> std::io::Result<ProjectLibraryActionResult> {
    ProjectLibraryStore::default().open_project(&project_id)
}

pub fn import_runtime_artifact(
    request: ImportRuntimeArtifactRequest,
) -> std::io::Result<ProjectLibraryActionResult> {
    ProjectLibraryStore::default().import_runtime_artifact(request)
}

pub fn mutate_item(request: LibraryMutationRequest) -> std::io::Result<ProjectLibraryActionResult> {
    ProjectLibraryStore::default().mutate_library_item(request)
}

pub fn library_playback(item_id: String) -> std::io::Result<LibraryPlayback> {
    ProjectLibraryStore::default().playback_for_item(&item_id)
}

pub fn review_edit(request: SaveReviewEditRequest) -> std::io::Result<ReviewEditResult> {
    ProjectLibraryStore::default().save_review_edit(request)
}

pub fn export_item(request: ExportLibraryItemRequest) -> std::io::Result<ExportLibraryItemResult> {
    ProjectLibraryStore::default().export_library_item(request)
}

pub fn export_workflow_overview() -> ExportWorkflowOverview {
    ExportWorkflowOverview::reference()
}

pub fn composition_editor_overview() -> CompositionEditorOverview {
    CompositionEditorOverview::reference()
}

pub fn runtime_overview() -> RuntimeOverview {
    RuntimeOverview::from_model_manager(
        &ModelManagerOverview::reference(),
        &soundworks_core::DeviceInventory::reference_mac(),
        soundworks_core::RuntimePackagingPolicy::shipped_desktop(),
        &RuntimeJobStore::default(),
    )
}

fn build_runtime_overview(store: &RuntimeJobStore) -> RuntimeOverview {
    RuntimeOverview::from_model_manager(
        &ModelManagerOverview::reference(),
        &soundworks_core::DeviceInventory::reference_mac(),
        soundworks_core::RuntimePackagingPolicy::shipped_desktop(),
        store,
    )
}

/// Spawn a background worker that runs a queued job to completion on the shared
/// engine and then releases its cancellation token (F-006). Job records live in
/// per-job directories, so the worker writes outside the command write lock
/// without contending on the shared `workspace.json`.
fn spawn_runtime_worker(engine: Arc<RuntimeEngine>, job_id: String) {
    std::thread::spawn(move || {
        let store = RuntimeJobStore::default();
        let ctx = engine.context_for(&job_id);
        let _ = store.run_job(&job_id, &ctx);
        engine.forget(&job_id);
    });
}

pub fn enqueue_job(
    engine: Arc<RuntimeEngine>,
    request: RuntimeJobRequest,
) -> std::io::Result<RuntimeJobSnapshot> {
    let store = RuntimeJobStore::default();
    let overview = build_runtime_overview(&store);
    let queued = store.enqueue(&overview, request)?;
    if queued.status == JobStatus::Queued {
        spawn_runtime_worker(engine, queued.id.clone());
    }
    Ok(queued)
}

pub fn cancel_job(
    engine: &RuntimeEngine,
    job_id: String,
) -> std::io::Result<Option<RuntimeJobSnapshot>> {
    // Signal any in-flight worker to stop, then persist the cancellation for a
    // job still sitting in the queue.
    engine.request_cancel(&job_id);
    RuntimeJobStore::default().cancel(&job_id)
}

pub fn retry_job(
    engine: Arc<RuntimeEngine>,
    job_id: String,
) -> std::io::Result<Option<RuntimeJobSnapshot>> {
    let store = RuntimeJobStore::default();
    let overview = build_runtime_overview(&store);
    let retried = store.retry(&overview, &job_id)?;
    if let Some(job) = &retried {
        if job.status == JobStatus::Queued {
            spawn_runtime_worker(engine, job.id.clone());
        }
    }
    Ok(retried)
}

pub fn runtime_job_artifacts(job_id: String) -> std::io::Result<Vec<RuntimeJobArtifact>> {
    RuntimeJobStore::default().artifacts(&job_id)
}

pub fn runtime_job(job_id: String) -> std::io::Result<Option<RuntimeJobSnapshot>> {
    RuntimeJobStore::default().job(&job_id)
}

pub fn model_evaluation_catalog() -> ModelEvaluationCatalog {
    ModelEvaluationCatalog::reference()
}

pub fn model_manager_overview() -> ModelManagerOverview {
    ModelManagerOverview::reference()
}

pub fn install_candidate(candidate_id: String) -> ModelManagerOperation {
    ModelManagerOperation::install(&candidate_id)
}

pub fn revalidate_candidate(candidate_id: String) -> ModelManagerOperation {
    ModelManagerOperation::revalidate(&candidate_id)
}

pub fn mvp_validation_overview() -> MvpValidationOverview {
    MvpValidationOverview::reference()
}

pub fn tts_studio_overview() -> TtsStudioOverview {
    TtsStudioOverview::reference().expect("reference TTS studio is valid")
}

pub fn voice_lab_overview() -> VoiceLabOverview {
    VoiceLabOverview::reference().expect("reference Voice Lab is valid")
}

pub fn sfx_studio_overview() -> SfxStudioOverview {
    SfxStudioOverview::reference().expect("reference SFX Studio is valid")
}

pub fn samples_studio_overview() -> SamplesStudioOverview {
    SamplesStudioOverview::from_catalogs(
        &ProviderCatalog::reference(),
        &runtime_overview(),
        &ModelEvaluationCatalog::reference(),
        &soundworks_core::StoragePathAllocator::new("soundworks-library"),
    )
    .expect("Samples Studio is valid")
}

pub fn song_studio_overview() -> SongStudioOverview {
    SongStudioOverview::reference().expect("reference Song Studio is valid")
}

pub fn review_workspace_overview() -> ReviewWorkspaceOverview {
    ReviewWorkspaceOverview::reference().expect("reference Review workspace is valid")
}

pub fn rights_safety_overview() -> RightsSafetyOverview {
    RightsSafetyOverview::reference()
}

pub fn video_to_audio_overview() -> VideoToAudioOverview {
    VideoToAudioOverview::reference().expect("reference Video to Audio is valid")
}

pub fn builder() -> tauri::Builder<tauri::Wry> {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            get_app_overview,
            get_provider_catalog,
            get_workspace_overview,
            get_asset_library_overview,
            create_soundworks_project,
            open_soundworks_project,
            import_runtime_artifact_to_library,
            mutate_library_item,
            get_library_playback,
            save_review_edit,
            export_library_item,
            get_export_workflow_overview,
            get_composition_editor_overview,
            get_runtime_overview,
            enqueue_runtime_job,
            cancel_runtime_job,
            retry_runtime_job,
            get_runtime_job_artifacts,
            get_runtime_job,
            get_model_evaluation_catalog,
            get_model_manager_overview,
            install_model_candidate,
            revalidate_model_candidate,
            get_mvp_validation_overview,
            get_tts_studio_overview,
            get_voice_lab_overview,
            get_sfx_studio_overview,
            get_samples_studio_overview,
            get_song_studio_overview,
            get_review_workspace_overview,
            get_rights_safety_overview,
            get_video_to_audio_overview,
            get_ui_preferences,
            set_ui_preferences
        ])
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    builder()
        .run(tauri::generate_context!())
        .expect("failed to run SoundWorks desktop app");
}

#[cfg(test)]
mod tests {
    use super::{
        app_overview, asset_library_overview, composition_editor_overview,
        export_workflow_overview, install_candidate, model_evaluation_catalog,
        model_manager_overview, mvp_validation_overview, provider_catalog, revalidate_candidate,
        review_workspace_overview, rights_safety_overview, runtime_job, runtime_overview,
        samples_studio_overview, sfx_studio_overview, song_studio_overview, tts_studio_overview,
        video_to_audio_overview, voice_lab_overview, workspace_overview,
    };

    #[test]
    fn app_overview_command_returns_soundworks() {
        let overview = app_overview();

        assert_eq!(overview.product_name, "SoundWorks");
        assert_eq!(overview.commands[0].name, "get_app_overview");
    }

    #[test]
    fn provider_catalog_command_returns_reference_manifests() {
        let catalog = provider_catalog();

        assert_eq!(catalog.schema_version, 1);
        assert_eq!(catalog.model_count(), 4);
    }

    /// F-009: point the library/runtime commands at an isolated, empty temp root
    /// so the production (persisted-only) path is deterministic and never reads the
    /// developer's real SoundWorks library. Set once for the whole test binary.
    fn isolated_library() {
        use std::sync::OnceLock;
        static INIT: OnceLock<()> = OnceLock::new();
        INIT.get_or_init(|| {
            let dir = std::env::temp_dir()
                .join(format!("soundworks-desktop-test-{}", std::process::id()));
            let _ = std::fs::create_dir_all(&dir);
            std::env::set_var("SOUNDWORKS_LIBRARY_ROOT", &dir);
            std::env::set_var("SOUNDWORKS_RUNTIME_ROOT", dir.join("runtime"));
        });
    }

    #[test]
    fn asset_library_command_returns_searchable_library() {
        isolated_library();
        let library = asset_library_overview();

        // Production default merges no fixtures: the empty library has no selection
        // but still exposes the full filter contract and the standard scopes.
        assert_eq!(library.schema_version, 1);
        assert_eq!(library.filters.supported_item_types.len(), 13);
        assert!(library.selected_item.is_none());
        assert!(library
            .scopes
            .iter()
            .any(|scope| scope.id == "project-demo"));
        assert!(library
            .scopes
            .iter()
            .any(|scope| scope.id == "global-library"));
    }

    #[test]
    fn workspace_command_returns_project_and_global_library_state() {
        isolated_library();
        let workspace = workspace_overview();

        // Production default: the seeded demo project exists, but the library
        // contains only persisted assets (none in a fresh root).
        assert_eq!(workspace.schema_version, 1);
        assert_eq!(workspace.active_project.project.id, "project-demo");
        assert_eq!(workspace.global_library.id, "global-library");
        assert!(workspace.project_assets.is_empty());
        assert!(workspace.global_assets.is_empty());
        assert!(workspace.source_picker.allows_global_sources);
    }

    #[test]
    fn demo_flag_restores_the_fixture_catalog() {
        // F-009: with the opt-in demo flag set, the fabricated catalog returns for
        // demos/screenshots. Asserted on the core builder so it is independent of
        // process-global env state.
        let demo =
            soundworks_core::AssetLibraryOverview::reference_with_persisted_items(vec![], None)
                .expect("demo library builds");
        assert!(demo.selected_item.is_some());
        assert!(demo.items.iter().any(|item| item.id == "asset-loop-001"));
    }

    #[test]
    fn export_workflow_command_returns_presets_and_sidecars() {
        let exports = export_workflow_overview();

        assert_eq!(exports.schema_version, 1);
        assert_eq!(exports.presets.len(), 7);
        assert!(exports.selected_export.can_export);
        assert!(exports.validation_checks.iter().all(|check| check.passed));
    }

    #[test]
    fn video_to_audio_command_returns_sync_contract() {
        let video = video_to_audio_overview();

        assert_eq!(video.schema_version, 1);
        assert_eq!(video.target_ranges.len(), 3);
        assert_eq!(video.sync_preview.sync_points.len(), 5);
        assert!(video.submission.can_submit);
        assert_eq!(
            video.saved_output.asset.kind,
            soundworks_core::AudioAssetKind::Sfx
        );
    }

    #[test]
    fn composition_editor_command_returns_timeline_and_component_decision() {
        let editor = composition_editor_overview();

        assert_eq!(editor.schema_version, 1);
        assert_eq!(editor.tracks.len(), 4);
        assert_eq!(editor.timeline.selected_clip_id, "clip-voice-intro");
        assert!(editor.mixer.render_ready);
        assert!(editor
            .component_decisions
            .iter()
            .any(|decision| decision.id == "waveform-playlist"));
    }

    #[test]
    fn runtime_overview_command_returns_worker_state() {
        let runtime = runtime_overview();

        assert_eq!(
            runtime.schema_version,
            soundworks_core::RUNTIME_SCHEMA_VERSION
        );
        assert!(runtime
            .model_states
            .iter()
            .all(|state| !state.model_id.starts_with("reference-")));
        assert!(runtime.model_states.iter().any(|state| {
            state.provider_id == "soundworks-native"
                && state.model_id == "native-procedural-sfx"
                && state
                    .workflows
                    .contains(&soundworks_core::CapabilityWorkflow::Sfx)
        }));
        assert!(runtime
            .jobs
            .iter()
            .all(|job| !job.id.starts_with("job-runtime-reference")));
        assert!(runtime
            .validation_checks
            .iter()
            .any(|check| check.id == "runtime.job_store"));
    }

    #[test]
    fn runtime_job_command_returns_none_for_unknown_id() {
        isolated_library();

        // UX-F1: the single-job polling command resolves a known job from the
        // store and returns None (not an error) for an id with no record, so the
        // web polling loop can stop cleanly. Traversal ids are still rejected by
        // the shared read_job guard, surfaced here as an Err.
        let missing = runtime_job("job-does-not-exist".to_string());
        assert!(matches!(missing, Ok(None)));
        assert!(runtime_job("../escape".to_string()).is_err());
    }

    #[test]
    fn model_evaluation_command_returns_scorecard() {
        let catalog = model_evaluation_catalog();

        assert_eq!(catalog.schema_version, 1);
        for &id in soundworks_core::REQUIRED_CANDIDATE_IDS {
            assert!(
                catalog
                    .candidates
                    .iter()
                    .any(|candidate| candidate.id == id),
                "missing {id}"
            );
        }
        assert!(catalog
            .recommendations
            .iter()
            .any(|recommendation| recommendation.candidate_id == "kokoro-82m"));
    }

    #[test]
    fn model_manager_commands_return_cache_verification_surface() {
        let manager = model_manager_overview();

        assert_eq!(manager.schema_version, 1);
        for &id in soundworks_core::REQUIRED_CANDIDATE_IDS {
            assert!(
                manager
                    .candidates
                    .iter()
                    .any(|candidate| candidate.candidate_id == id),
                "missing {id}"
            );
        }
        assert!(
            manager.summary.verified_installed_count
                <= soundworks_core::REQUIRED_CANDIDATE_IDS.len()
        );
        assert!(manager
            .candidates
            .iter()
            .any(|candidate| candidate.candidate_id == "kokoro-82m"));

        let install = install_candidate("kokoro-82m".to_string());
        assert_eq!(install.candidate_id, "kokoro-82m");
        assert_eq!(
            install.status,
            soundworks_core::ModelManagerOperationStatus::Failed
        );

        let revalidate = revalidate_candidate("kokoro-82m".to_string());
        assert_eq!(revalidate.candidate_id, "kokoro-82m");
        assert!(matches!(
            revalidate.status,
            soundworks_core::ModelManagerOperationStatus::Failed
                | soundworks_core::ModelManagerOperationStatus::Succeeded
        ));
    }

    #[test]
    fn mvp_validation_command_returns_release_gate() {
        let overview = mvp_validation_overview();

        assert_eq!(overview.schema_version, 1);
        assert_eq!(overview.demo_workflows.len(), 12);
        assert_eq!(overview.regression_fixtures.len(), 12);
        assert!(!overview.release_gate.ready_for_mvp);
        assert!(overview
            .release_gate
            .blocking_items
            .iter()
            .any(|item| item.contains("stress cases")));
        assert_eq!(overview.release_gate.satisfied_runtime_evidence_count, 0);
        assert_eq!(overview.release_gate.fixture_only_evidence_count, 5);
    }

    #[test]
    fn tts_studio_command_returns_submission_contract() {
        let overview = tts_studio_overview();

        assert_eq!(overview.schema_version, 1);
        assert!(!overview.submission.can_submit);
        assert_eq!(overview.script.segments.len(), 3);
        assert_eq!(overview.saved_output.asset.id, "asset-tts-studio-reference");
    }

    #[test]
    fn voice_lab_command_returns_conversion_contract() {
        let overview = voice_lab_overview();

        assert_eq!(overview.schema_version, 1);
        assert_eq!(overview.modes.len(), 3);
        assert!(overview.selected_conversion.can_submit);
        assert_eq!(
            overview.saved_output.asset.id,
            "asset-voice-lab-conversion-reference"
        );
    }

    #[test]
    fn sfx_studio_command_returns_variant_contract() {
        let overview = sfx_studio_overview();

        assert_eq!(overview.schema_version, 1);
        assert_eq!(overview.variants.len(), 3);
        assert!(!overview.submission.can_submit);
        assert_eq!(overview.saved_outputs.len(), 2);
    }

    #[test]
    fn samples_studio_command_returns_pack_contract() {
        let overview = samples_studio_overview();

        assert_eq!(overview.schema_version, 1);
        assert_eq!(overview.variants.len(), 4);
        assert!(overview.submission.can_submit);
        assert_eq!(overview.saved_outputs.len(), 3);
        assert_eq!(overview.pack.collection_id, "collection-neon-bass-pack");
    }

    #[test]
    fn song_studio_command_returns_complete_song_contract() {
        let overview = song_studio_overview();

        assert_eq!(overview.schema_version, 1);
        assert_eq!(overview.arrangement.section_count, 4);
        assert!(!overview.submission.can_submit);
        assert_eq!(overview.saved_outputs.len(), 2);
        assert_eq!(overview.export_targets.len(), 3);
    }

    #[test]
    fn review_workspace_command_returns_edit_contract() {
        let overview = review_workspace_overview();

        assert_eq!(overview.schema_version, 1);
        assert_eq!(overview.assets.len(), 5);
        assert_eq!(overview.edit_actions.len(), 8);
        assert!(overview.edit_submission.can_save);
        assert_eq!(
            overview.edit_submission.saved_version.id,
            "version-loop-001-b-review-edit"
        );
        assert!(overview.provenance.inspectable);
    }

    #[test]
    fn rights_safety_command_returns_policy_contract() {
        let overview = rights_safety_overview();

        assert_eq!(overview.schema_version, 1);
        assert!(overview
            .consent_checks
            .iter()
            .any(|check| check.decision == soundworks_core::PolicyDecision::Blocked));
        assert!(overview
            .export_sidecars
            .iter()
            .all(|sidecar| sidecar.includes_rights));
    }
}
