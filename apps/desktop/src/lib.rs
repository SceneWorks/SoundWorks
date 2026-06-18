use soundworks_core::{
    AppOverview, ModelEvaluationCatalog, ProviderCatalog, RuntimeOverview, TtsStudioOverview,
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

pub fn builder() -> tauri::Builder<tauri::Wry> {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_app_overview,
            get_provider_catalog,
            get_runtime_overview,
            get_model_evaluation_catalog,
            get_tts_studio_overview
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
        app_overview, model_evaluation_catalog, provider_catalog, runtime_overview,
        tts_studio_overview,
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
}
