use crate::domain::{AudioAssetKind, AudioFileFormat, LibraryScope};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const SCHEMA_MIGRATIONS: &[SchemaMigration] = &[
    SchemaMigration {
        version: 1,
        name: "soundworks_core_domain",
        sql: "
CREATE TABLE projects (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  storage_root TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE TABLE audio_assets (
  id TEXT PRIMARY KEY,
  scope_kind TEXT NOT NULL,
  project_id TEXT,
  kind TEXT NOT NULL,
  name TEXT NOT NULL,
  current_version_id TEXT NOT NULL,
  rights_json TEXT NOT NULL,
  provenance_json TEXT NOT NULL
);
CREATE TABLE audio_asset_versions (
  id TEXT PRIMARY KEY,
  asset_id TEXT NOT NULL REFERENCES audio_assets(id),
  version_index INTEGER NOT NULL,
  file_json TEXT NOT NULL,
  technical_json TEXT NOT NULL,
  created_by_json TEXT NOT NULL,
  UNIQUE(asset_id, version_index)
);
CREATE TABLE generation_recipes (
  id TEXT PRIMARY KEY,
  workflow TEXT NOT NULL,
  provider_json TEXT NOT NULL,
  request_json TEXT NOT NULL,
  seed INTEGER,
  references_json TEXT NOT NULL,
  post_processing_json TEXT NOT NULL,
  overrides_json TEXT NOT NULL,
  outputs_json TEXT NOT NULL
);
CREATE TABLE generation_jobs (
  id TEXT PRIMARY KEY,
  recipe_id TEXT NOT NULL REFERENCES generation_recipes(id),
  kind TEXT NOT NULL,
  status TEXT NOT NULL,
  progress_json TEXT,
  outputs_json TEXT NOT NULL,
  error TEXT
);
CREATE TABLE voice_profiles (
  id TEXT PRIMARY KEY,
  display_name TEXT NOT NULL,
  source_reference_ids_json TEXT NOT NULL,
  consent TEXT NOT NULL,
  allowed_uses_json TEXT NOT NULL,
  provenance_ids_json TEXT NOT NULL
);
CREATE TABLE compositions (
  id TEXT PRIMARY KEY,
  scope_kind TEXT NOT NULL,
  project_id TEXT,
  name TEXT NOT NULL,
  composition_json TEXT NOT NULL
);
CREATE TABLE collections (
  id TEXT PRIMARY KEY,
  scope_kind TEXT NOT NULL,
  project_id TEXT,
  name TEXT NOT NULL,
  asset_ids_json TEXT NOT NULL
);
CREATE TABLE prompt_presets (
  id TEXT PRIMARY KEY,
  workflow TEXT NOT NULL,
  name TEXT NOT NULL,
  prompt_template TEXT NOT NULL,
  defaults_json TEXT NOT NULL
);
",
    },
    SchemaMigration {
        version: 2,
        name: "storage_paths_and_sidecars",
        sql: "
CREATE TABLE storage_paths (
  version_id TEXT PRIMARY KEY REFERENCES audio_asset_versions(id),
  media_path TEXT NOT NULL UNIQUE,
  waveform_preview_path TEXT NOT NULL UNIQUE,
  spectrogram_preview_path TEXT NOT NULL UNIQUE,
  sidecar_path TEXT NOT NULL UNIQUE
);
",
    },
    SchemaMigration {
        version: 3,
        name: "provider_model_manifests",
        sql: "
CREATE TABLE provider_manifests (
  id TEXT PRIMARY KEY,
  manifest_version TEXT NOT NULL,
  source_json TEXT NOT NULL,
  provider_json TEXT NOT NULL
);
CREATE TABLE model_manifests (
  id TEXT PRIMARY KEY,
  provider_id TEXT NOT NULL REFERENCES provider_manifests(id),
  runtime TEXT NOT NULL,
  install_json TEXT NOT NULL,
  requirements_json TEXT NOT NULL,
  capabilities_json TEXT NOT NULL,
  default_priority INTEGER NOT NULL DEFAULT 0
);
",
    },
    SchemaMigration {
        version: 4,
        name: "worker_runtime_state",
        sql: "
CREATE TABLE runtime_model_states (
  provider_id TEXT NOT NULL,
  model_id TEXT NOT NULL,
  availability TEXT NOT NULL,
  install_status TEXT NOT NULL,
  cache_json TEXT NOT NULL,
  compatibility_json TEXT NOT NULL,
  health TEXT NOT NULL,
  reasons_json TEXT NOT NULL,
  PRIMARY KEY(provider_id, model_id)
);
CREATE TABLE runtime_jobs (
  id TEXT PRIMARY KEY,
  kind TEXT NOT NULL,
  status TEXT NOT NULL,
  provider_id TEXT NOT NULL,
  model_id TEXT NOT NULL,
  progress_json TEXT,
  cancellation TEXT NOT NULL,
  retry_count INTEGER NOT NULL DEFAULT 0,
  log_tail_json TEXT NOT NULL,
  actionable_error_json TEXT
);
CREATE TABLE runtime_validation_checks (
  id TEXT PRIMARY KEY,
  status TEXT NOT NULL,
  summary TEXT NOT NULL,
  recovery TEXT
);
",
    },
    SchemaMigration {
        version: 5,
        name: "model_evaluation_scorecards",
        sql: "
CREATE TABLE model_evaluation_candidates (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  provider TEXT NOT NULL,
  lanes_json TEXT NOT NULL,
  sources_json TEXT NOT NULL,
  license_json TEXT NOT NULL,
  runtime_json TEXT NOT NULL,
  status TEXT NOT NULL,
  product_eligibility TEXT NOT NULL,
  evidence_level TEXT NOT NULL,
  blockers_json TEXT NOT NULL,
  notes TEXT NOT NULL
);
CREATE TABLE model_evaluation_fixtures (
  id TEXT PRIMARY KEY,
  lane TEXT NOT NULL,
  name TEXT NOT NULL,
  prompt_or_script TEXT NOT NULL,
  expected_outputs_json TEXT NOT NULL,
  measurements_json TEXT NOT NULL
);
CREATE TABLE model_evaluation_recommendations (
  lane TEXT PRIMARY KEY,
  candidate_id TEXT NOT NULL REFERENCES model_evaluation_candidates(id),
  status TEXT NOT NULL,
  rationale TEXT NOT NULL,
  required_next_evidence_json TEXT NOT NULL
);
",
    },
    SchemaMigration {
        version: 6,
        name: "tts_studio_workflow",
        sql: "
CREATE TABLE tts_scripts (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  language TEXT NOT NULL,
  pronunciation_dictionary_json TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE TABLE tts_script_segments (
  id TEXT PRIMARY KEY,
  script_id TEXT NOT NULL REFERENCES tts_scripts(id),
  position INTEGER NOT NULL,
  speaker_label TEXT NOT NULL,
  text TEXT NOT NULL,
  scene_label TEXT,
  target_duration_ms INTEGER,
  regenerate_policy TEXT NOT NULL,
  UNIQUE(script_id, position)
);
CREATE TABLE tts_speakers (
  script_id TEXT NOT NULL REFERENCES tts_scripts(id),
  label TEXT NOT NULL,
  role TEXT NOT NULL,
  voice_profile_id TEXT NOT NULL REFERENCES voice_profiles(id),
  language TEXT NOT NULL,
  consent_required INTEGER NOT NULL,
  consent_status TEXT NOT NULL,
  PRIMARY KEY(script_id, label)
);
CREATE TABLE tts_generation_submissions (
  id TEXT PRIMARY KEY,
  script_id TEXT NOT NULL REFERENCES tts_scripts(id),
  provider_id TEXT NOT NULL,
  model_id TEXT NOT NULL,
  recipe_id TEXT NOT NULL REFERENCES generation_recipes(id),
  job_id TEXT NOT NULL REFERENCES generation_jobs(id),
  can_submit INTEGER NOT NULL,
  blocking_reasons_json TEXT NOT NULL,
  warnings_json TEXT NOT NULL
);
CREATE TABLE tts_saved_outputs (
  submission_id TEXT PRIMARY KEY REFERENCES tts_generation_submissions(id),
  asset_id TEXT NOT NULL REFERENCES audio_assets(id),
  version_id TEXT NOT NULL REFERENCES audio_asset_versions(id),
  promoted_to_project_library INTEGER NOT NULL,
  waveform_preview_ready INTEGER NOT NULL
);
",
    },
    SchemaMigration {
        version: 7,
        name: "voice_lab_workflow",
        sql: "
CREATE TABLE voice_lab_profiles (
  id TEXT PRIMARY KEY REFERENCES voice_profiles(id),
  speaker_identity TEXT NOT NULL,
  language TEXT NOT NULL,
  source_clip_ids_json TEXT NOT NULL,
  mode_readiness_json TEXT NOT NULL,
  commercial_use_allowed INTEGER NOT NULL,
  safety_summary TEXT NOT NULL
);
CREATE TABLE voice_lab_reference_clips (
  id TEXT PRIMARY KEY,
  asset_id TEXT NOT NULL REFERENCES audio_assets(id),
  profile_id TEXT NOT NULL REFERENCES voice_profiles(id),
  label TEXT NOT NULL,
  duration_ms INTEGER NOT NULL,
  consent TEXT NOT NULL,
  owner_attestation TEXT NOT NULL,
  accepted_for_modes_json TEXT NOT NULL
);
CREATE TABLE voice_lab_provider_scorecards (
  candidate_id TEXT PRIMARY KEY REFERENCES model_evaluation_candidates(id),
  readiness TEXT NOT NULL,
  recommended INTEGER NOT NULL,
  blockers_json TEXT NOT NULL,
  notes TEXT NOT NULL
);
CREATE TABLE voice_lab_safety_gates (
  id TEXT PRIMARY KEY,
  status TEXT NOT NULL,
  summary TEXT NOT NULL
);
CREATE TABLE voice_lab_qa_checks (
  id TEXT PRIMARY KEY,
  label TEXT NOT NULL,
  status TEXT NOT NULL,
  target TEXT NOT NULL
);
CREATE TABLE voice_lab_conversion_submissions (
  id TEXT PRIMARY KEY,
  source_audio_asset_id TEXT NOT NULL REFERENCES audio_assets(id),
  target_voice_profile_id TEXT NOT NULL REFERENCES voice_profiles(id),
  provider_candidate_id TEXT NOT NULL REFERENCES model_evaluation_candidates(id),
  recipe_id TEXT NOT NULL REFERENCES generation_recipes(id),
  job_id TEXT NOT NULL REFERENCES generation_jobs(id),
  output_asset_id TEXT NOT NULL REFERENCES audio_assets(id),
  can_submit INTEGER NOT NULL,
  blocking_reasons_json TEXT NOT NULL,
  warnings_json TEXT NOT NULL
);
",
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaMigration {
    pub version: u32,
    pub name: &'static str,
    pub sql: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoragePaths {
    pub media_path: String,
    pub waveform_preview_path: String,
    pub spectrogram_preview_path: String,
    pub sidecar_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoragePathAllocator {
    root: String,
}

impl StoragePathAllocator {
    pub fn new(root: impl Into<String>) -> Self {
        Self { root: root.into() }
    }

    pub fn allocate_asset_version(
        &self,
        scope: &LibraryScope,
        kind: AudioAssetKind,
        asset_id: &str,
        version_id: &str,
        format: AudioFileFormat,
    ) -> Result<StoragePaths, StoragePathError> {
        let root = clean_path_segment(&self.root)?;
        let asset_id = clean_path_segment(asset_id)?;
        let version_id = clean_path_segment(version_id)?;
        let scope_prefix = clean_scope_prefix(scope)?;
        let kind_dir = kind.storage_dir();
        let extension = format.extension();
        let version_base = format!("{root}/{scope_prefix}/{kind_dir}/{asset_id}/{version_id}");

        Ok(StoragePaths {
            media_path: format!("{version_base}/media.{extension}"),
            waveform_preview_path: format!("{version_base}/previews/waveform.json"),
            spectrogram_preview_path: format!("{version_base}/previews/spectrogram.bin"),
            sidecar_path: format!("{version_base}/metadata/recipe-provenance.json"),
        })
    }
}

fn clean_scope_prefix(scope: &LibraryScope) -> Result<String, StoragePathError> {
    match scope {
        LibraryScope::GlobalLibrary => Ok("global".to_string()),
        LibraryScope::Project { project_id } => {
            Ok(format!("projects/{}", clean_path_segment(project_id)?))
        }
    }
}

fn clean_path_segment(value: &str) -> Result<String, StoragePathError> {
    if value.is_empty() {
        return Err(StoragePathError::EmptySegment);
    }

    if value == "." || value == ".." {
        return Err(StoragePathError::UnsafeSegment(value.to_string()));
    }

    let safe = value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'));

    if safe {
        Ok(value.to_string())
    } else {
        Err(StoragePathError::UnsafeSegment(value.to_string()))
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum StoragePathError {
    #[error("storage path segment cannot be empty")]
    EmptySegment,
    #[error("unsafe storage path segment: {0}")]
    UnsafeSegment(String),
}
