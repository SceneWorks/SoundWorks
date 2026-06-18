use soundworks_core::{
    AppOverview, ModelEvaluationCatalog, ProviderCatalog, ReviewWorkspaceOverview,
    RightsSafetyOverview, RuntimeOverview, SamplesStudioOverview, SfxStudioOverview,
    SongStudioOverview, TtsStudioOverview, VoiceLabOverview,
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
fn get_runtime_overview() -> RuntimeOverview {
    runtime_overview()
}

#[tauri::command]
fn get_model_evaluation_catalog() -> ModelEvaluationCatalog {
    model_evaluation_catalog()
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

pub fn app_overview() -> AppOverview {
    AppOverview::baseline()
}

pub fn provider_catalog() -> ProviderCatalog {
    ProviderCatalog::reference()
}

pub fn runtime_overview() -> RuntimeOverview {
    RuntimeOverview::reference()
}

pub fn model_evaluation_catalog() -> ModelEvaluationCatalog {
    ModelEvaluationCatalog::reference()
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
    SamplesStudioOverview::reference().expect("reference Samples Studio is valid")
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

pub fn builder() -> tauri::Builder<tauri::Wry> {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_app_overview,
            get_provider_catalog,
            get_runtime_overview,
            get_model_evaluation_catalog,
            get_tts_studio_overview,
            get_voice_lab_overview,
            get_sfx_studio_overview,
            get_samples_studio_overview,
            get_song_studio_overview,
            get_review_workspace_overview,
            get_rights_safety_overview
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
        app_overview, model_evaluation_catalog, provider_catalog, review_workspace_overview,
        rights_safety_overview, runtime_overview, samples_studio_overview, sfx_studio_overview,
        song_studio_overview, tts_studio_overview, voice_lab_overview,
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
        assert_eq!(catalog.model_count(), 3);
    }

    #[test]
    fn runtime_overview_command_returns_worker_state() {
        let runtime = runtime_overview();

        assert_eq!(runtime.schema_version, 1);
        assert_eq!(runtime.status_counts.installed, 3);
        assert!(runtime
            .jobs
            .iter()
            .any(|job| job.id == "job-runtime-reference-generate"));
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
    fn tts_studio_command_returns_submission_contract() {
        let overview = tts_studio_overview();

        assert_eq!(overview.schema_version, 1);
        assert!(overview.submission.can_submit);
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
        assert!(overview.submission.can_submit);
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
        assert!(overview.submission.can_submit);
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
