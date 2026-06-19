use soundworks_core::{
    AppOverview, AssetLibraryOverview, CompositionEditorOverview, CreateProjectRequest,
    ExportWorkflowOverview, ImportRuntimeArtifactRequest, LibraryMutationRequest, LibraryPlayback,
    ModelEvaluationCatalog, ModelManagerOperation, ModelManagerOverview, MvpValidationOverview,
    ProjectLibraryActionResult, ProjectLibraryStore, ProviderCatalog, ReviewWorkspaceOverview,
    RightsSafetyOverview, RuntimeJobArtifact, RuntimeJobRequest, RuntimeJobSnapshot,
    RuntimeJobStore, RuntimeOverview, SamplesStudioOverview, SfxStudioOverview, SongStudioOverview,
    TtsStudioOverview, VideoToAudioOverview, VoiceLabOverview, WorkspaceOverview,
};

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
    request: CreateProjectRequest,
) -> Result<ProjectLibraryActionResult, String> {
    create_project(request).map_err(|error| error.to_string())
}

#[tauri::command]
fn open_soundworks_project(project_id: String) -> Result<ProjectLibraryActionResult, String> {
    open_project(project_id).map_err(|error| error.to_string())
}

#[tauri::command]
fn import_runtime_artifact_to_library(
    request: ImportRuntimeArtifactRequest,
) -> Result<ProjectLibraryActionResult, String> {
    import_runtime_artifact(request).map_err(|error| error.to_string())
}

#[tauri::command]
fn mutate_library_item(
    request: LibraryMutationRequest,
) -> Result<ProjectLibraryActionResult, String> {
    mutate_item(request).map_err(|error| error.to_string())
}

#[tauri::command]
fn get_library_playback(item_id: String) -> Result<LibraryPlayback, String> {
    library_playback(item_id).map_err(|error| error.to_string())
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
fn enqueue_runtime_job(request: RuntimeJobRequest) -> Result<RuntimeJobSnapshot, String> {
    enqueue_job(request).map_err(|error| error.to_string())
}

#[tauri::command]
fn cancel_runtime_job(job_id: String) -> Result<Option<RuntimeJobSnapshot>, String> {
    cancel_job(job_id).map_err(|error| error.to_string())
}

#[tauri::command]
fn retry_runtime_job(job_id: String) -> Result<Option<RuntimeJobSnapshot>, String> {
    retry_job(job_id).map_err(|error| error.to_string())
}

#[tauri::command]
fn get_runtime_job_artifacts(job_id: String) -> Result<Vec<RuntimeJobArtifact>, String> {
    runtime_job_artifacts(job_id).map_err(|error| error.to_string())
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

pub fn enqueue_job(request: RuntimeJobRequest) -> std::io::Result<RuntimeJobSnapshot> {
    let store = RuntimeJobStore::default();
    let overview = RuntimeOverview::from_model_manager(
        &ModelManagerOverview::reference(),
        &soundworks_core::DeviceInventory::reference_mac(),
        soundworks_core::RuntimePackagingPolicy::shipped_desktop(),
        &store,
    );
    store.enqueue(&overview, request)
}

pub fn cancel_job(job_id: String) -> std::io::Result<Option<RuntimeJobSnapshot>> {
    RuntimeJobStore::default().cancel(&job_id)
}

pub fn retry_job(job_id: String) -> std::io::Result<Option<RuntimeJobSnapshot>> {
    let store = RuntimeJobStore::default();
    let overview = RuntimeOverview::from_model_manager(
        &ModelManagerOverview::reference(),
        &soundworks_core::DeviceInventory::reference_mac(),
        soundworks_core::RuntimePackagingPolicy::shipped_desktop(),
        &store,
    );
    store.retry(&overview, &job_id)
}

pub fn runtime_job_artifacts(job_id: String) -> std::io::Result<Vec<RuntimeJobArtifact>> {
    RuntimeJobStore::default().artifacts(&job_id)
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
            get_export_workflow_overview,
            get_composition_editor_overview,
            get_runtime_overview,
            enqueue_runtime_job,
            cancel_runtime_job,
            retry_runtime_job,
            get_runtime_job_artifacts,
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
            get_video_to_audio_overview
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
        review_workspace_overview, rights_safety_overview, runtime_overview,
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

    #[test]
    fn asset_library_command_returns_searchable_library() {
        let library = asset_library_overview();

        assert_eq!(library.schema_version, 1);
        assert_eq!(library.filters.supported_item_types.len(), 13);
        assert!(library
            .filters
            .facets
            .iter()
            .any(|facet| facet.id == "lifecycle"));
        assert_eq!(library.selected_item.item.id, "asset-loop-001");
    }

    #[test]
    fn workspace_command_returns_project_and_global_library_state() {
        let workspace = workspace_overview();

        assert_eq!(workspace.schema_version, 1);
        assert_eq!(workspace.active_project.project.id, "project-demo");
        assert_eq!(workspace.global_library.id, "global-library");
        assert_eq!(workspace.project_assets.len(), 10);
        assert_eq!(workspace.global_assets.len(), 3);
        assert!(workspace.source_picker.allows_global_sources);
        assert!(workspace
            .transfer_actions
            .iter()
            .any(|action| action.id == "promote-loop-to-global"));
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
    fn model_evaluation_command_returns_scorecard() {
        let catalog = model_evaluation_catalog();

        assert_eq!(catalog.schema_version, 1);
        assert_eq!(catalog.candidates.len(), 28);
        assert!(catalog
            .recommendations
            .iter()
            .any(|recommendation| recommendation.candidate_id == "kokoro-82m"));
    }

    #[test]
    fn model_manager_commands_return_cache_verification_surface() {
        let manager = model_manager_overview();

        assert_eq!(manager.schema_version, 1);
        assert_eq!(manager.summary.candidate_count, 28);
        assert!(manager.summary.verified_installed_count <= 28);
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
