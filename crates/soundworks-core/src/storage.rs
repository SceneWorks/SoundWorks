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
    SchemaMigration {
        version: 8,
        name: "sfx_studio_workflow",
        sql: "
CREATE TABLE sfx_studio_prompts (
  id TEXT PRIMARY KEY,
  text TEXT NOT NULL,
  negative_prompt TEXT NOT NULL,
  category TEXT NOT NULL,
  tags_json TEXT NOT NULL,
  reference_audio_asset_id TEXT REFERENCES audio_assets(id),
  controls_json TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE TABLE sfx_studio_variants (
  id TEXT PRIMARY KEY,
  prompt_id TEXT NOT NULL REFERENCES sfx_studio_prompts(id),
  label TEXT NOT NULL,
  workflow TEXT NOT NULL,
  asset_kind TEXT NOT NULL,
  category TEXT NOT NULL,
  duration_ms INTEGER NOT NULL,
  loudness_lufs REAL NOT NULL,
  true_peak_dbfs REAL NOT NULL,
  loopable INTEGER NOT NULL,
  loop_points_json TEXT,
  tags_json TEXT NOT NULL,
  selected_for_save INTEGER NOT NULL
);
CREATE TABLE sfx_studio_provider_scorecards (
  candidate_id TEXT PRIMARY KEY REFERENCES model_evaluation_candidates(id),
  readiness TEXT NOT NULL,
  recommended INTEGER NOT NULL,
  blockers_json TEXT NOT NULL,
  notes TEXT NOT NULL
);
CREATE TABLE sfx_studio_post_processing_actions (
  id TEXT PRIMARY KEY,
  operation TEXT NOT NULL,
  enabled INTEGER NOT NULL,
  summary TEXT NOT NULL
);
CREATE TABLE sfx_studio_submissions (
  id TEXT PRIMARY KEY,
  prompt_id TEXT NOT NULL REFERENCES sfx_studio_prompts(id),
  provider_id TEXT NOT NULL,
  model_id TEXT NOT NULL,
  recipe_id TEXT NOT NULL REFERENCES generation_recipes(id),
  job_id TEXT NOT NULL REFERENCES generation_jobs(id),
  can_submit INTEGER NOT NULL,
  blocking_reasons_json TEXT NOT NULL,
  warnings_json TEXT NOT NULL
);
CREATE TABLE sfx_studio_saved_outputs (
  submission_id TEXT NOT NULL REFERENCES sfx_studio_submissions(id),
  variant_id TEXT NOT NULL REFERENCES sfx_studio_variants(id),
  asset_id TEXT NOT NULL REFERENCES audio_assets(id),
  version_id TEXT NOT NULL REFERENCES audio_asset_versions(id),
  exported INTEGER NOT NULL,
  waveform_preview_ready INTEGER NOT NULL,
  PRIMARY KEY(submission_id, variant_id)
);
",
    },
    SchemaMigration {
        version: 9,
        name: "samples_studio_workflow",
        sql: "
CREATE TABLE samples_studio_prompts (
  id TEXT PRIMARY KEY,
  text TEXT NOT NULL,
  negative_prompt TEXT NOT NULL,
  instrument_family TEXT NOT NULL,
  articulation TEXT NOT NULL,
  genre_tags_json TEXT NOT NULL,
  reference_audio_asset_id TEXT REFERENCES audio_assets(id),
  controls_json TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE TABLE samples_studio_variants (
  id TEXT PRIMARY KEY,
  prompt_id TEXT NOT NULL REFERENCES samples_studio_prompts(id),
  label TEXT NOT NULL,
  workflow TEXT NOT NULL,
  asset_kind TEXT NOT NULL,
  instrument_family TEXT NOT NULL,
  articulation TEXT NOT NULL,
  duration_ms INTEGER NOT NULL,
  bpm REAL,
  musical_key TEXT,
  time_signature TEXT,
  loop_points_json TEXT,
  transient_one_shot INTEGER NOT NULL,
  loudness_lufs REAL NOT NULL,
  true_peak_dbfs REAL NOT NULL,
  has_clipping INTEGER NOT NULL,
  tags_json TEXT NOT NULL,
  collection_id TEXT NOT NULL,
  selected_for_pack INTEGER NOT NULL,
  favorite INTEGER NOT NULL,
  duplicate_of_variant_id TEXT REFERENCES samples_studio_variants(id)
);
CREATE TABLE samples_studio_provider_scorecards (
  candidate_id TEXT PRIMARY KEY REFERENCES model_evaluation_candidates(id),
  readiness TEXT NOT NULL,
  recommended INTEGER NOT NULL,
  blockers_json TEXT NOT NULL,
  notes TEXT NOT NULL
);
CREATE TABLE samples_studio_pack_collections (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  selected_variant_ids_json TEXT NOT NULL,
  favorite_variant_ids_json TEXT NOT NULL,
  loop_variant_ids_json TEXT NOT NULL,
  one_shot_variant_ids_json TEXT NOT NULL,
  export_formats_json TEXT NOT NULL
);
CREATE TABLE samples_studio_post_processing_actions (
  id TEXT PRIMARY KEY,
  operation TEXT NOT NULL,
  enabled INTEGER NOT NULL,
  summary TEXT NOT NULL
);
CREATE TABLE samples_studio_qa_checks (
  id TEXT PRIMARY KEY,
  status TEXT NOT NULL,
  summary TEXT NOT NULL
);
CREATE TABLE samples_studio_submissions (
  id TEXT PRIMARY KEY,
  prompt_id TEXT NOT NULL REFERENCES samples_studio_prompts(id),
  provider_id TEXT NOT NULL,
  model_id TEXT NOT NULL,
  recipe_ids_json TEXT NOT NULL,
  job_ids_json TEXT NOT NULL,
  can_submit INTEGER NOT NULL,
  blocking_reasons_json TEXT NOT NULL,
  warnings_json TEXT NOT NULL
);
CREATE TABLE samples_studio_saved_outputs (
  submission_id TEXT NOT NULL REFERENCES samples_studio_submissions(id),
  variant_id TEXT NOT NULL REFERENCES samples_studio_variants(id),
  asset_id TEXT NOT NULL REFERENCES audio_assets(id),
  version_id TEXT NOT NULL REFERENCES audio_asset_versions(id),
  exported INTEGER NOT NULL,
  waveform_preview_ready INTEGER NOT NULL,
  PRIMARY KEY(submission_id, variant_id)
);
",
    },
    SchemaMigration {
        version: 10,
        name: "song_studio_workflow",
        sql: "
CREATE TABLE song_studio_drafts (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  prompt TEXT NOT NULL,
  lyrics TEXT NOT NULL,
  style_tags_json TEXT NOT NULL,
  language TEXT NOT NULL,
  vocalist TEXT NOT NULL,
  singer_hint TEXT,
  reference_audio_asset_ids_json TEXT NOT NULL,
  controls_json TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE TABLE song_studio_sections (
  id TEXT PRIMARY KEY,
  draft_id TEXT NOT NULL REFERENCES song_studio_drafts(id),
  position INTEGER NOT NULL,
  label TEXT NOT NULL,
  bars INTEGER NOT NULL,
  lyrics TEXT,
  regenerate_locked INTEGER NOT NULL,
  UNIQUE(draft_id, position)
);
CREATE TABLE song_studio_variants (
  id TEXT PRIMARY KEY,
  draft_id TEXT NOT NULL REFERENCES song_studio_drafts(id),
  label TEXT NOT NULL,
  asset_kind TEXT NOT NULL,
  duration_ms INTEGER NOT NULL,
  bpm REAL NOT NULL,
  musical_key TEXT NOT NULL,
  vocal_mode TEXT NOT NULL,
  stem_kinds_json TEXT NOT NULL,
  loudness_lufs REAL NOT NULL,
  true_peak_dbfs REAL NOT NULL,
  lyric_alignment_score INTEGER NOT NULL,
  structure_match_score INTEGER NOT NULL,
  tags_json TEXT NOT NULL,
  selected_for_save INTEGER NOT NULL
);
CREATE TABLE song_studio_provider_scorecards (
  candidate_id TEXT PRIMARY KEY REFERENCES model_evaluation_candidates(id),
  readiness TEXT NOT NULL,
  recommended INTEGER NOT NULL,
  blockers_json TEXT NOT NULL,
  notes TEXT NOT NULL
);
CREATE TABLE song_studio_submissions (
  id TEXT PRIMARY KEY,
  draft_id TEXT NOT NULL REFERENCES song_studio_drafts(id),
  provider_id TEXT NOT NULL,
  model_id TEXT NOT NULL,
  recipe_id TEXT NOT NULL REFERENCES generation_recipes(id),
  job_id TEXT NOT NULL REFERENCES generation_jobs(id),
  can_submit INTEGER NOT NULL,
  blocking_reasons_json TEXT NOT NULL,
  warnings_json TEXT NOT NULL
);
CREATE TABLE song_studio_saved_outputs (
  submission_id TEXT NOT NULL REFERENCES song_studio_submissions(id),
  variant_id TEXT NOT NULL REFERENCES song_studio_variants(id),
  asset_id TEXT NOT NULL REFERENCES audio_assets(id),
  version_id TEXT NOT NULL REFERENCES audio_asset_versions(id),
  export_ready INTEGER NOT NULL,
  waveform_preview_ready INTEGER NOT NULL,
  PRIMARY KEY(submission_id, variant_id)
);
CREATE TABLE song_studio_export_targets (
  id TEXT PRIMARY KEY,
  label TEXT NOT NULL,
  formats_json TEXT NOT NULL,
  includes_stems INTEGER NOT NULL,
  includes_sidecar INTEGER NOT NULL,
  summary TEXT NOT NULL
);
",
    },
    SchemaMigration {
        version: 11,
        name: "review_workspace_workflow",
        sql: "
CREATE TABLE review_workspace_assets (
  asset_id TEXT PRIMARY KEY REFERENCES audio_assets(id),
  source_workflow TEXT NOT NULL,
  can_preview INTEGER NOT NULL,
  preview_status TEXT NOT NULL
);
CREATE TABLE review_preview_caches (
  version_id TEXT PRIMARY KEY REFERENCES audio_asset_versions(id),
  waveform_cache_path TEXT NOT NULL,
  spectrogram_cache_path TEXT NOT NULL,
  status TEXT NOT NULL,
  generated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE TABLE review_edit_actions (
  id TEXT PRIMARY KEY,
  kind TEXT NOT NULL,
  operation TEXT,
  enabled INTEGER NOT NULL,
  destructive INTEGER NOT NULL,
  non_destructive_save INTEGER NOT NULL,
  parameters_json TEXT NOT NULL
);
CREATE TABLE review_edit_submissions (
  id TEXT PRIMARY KEY,
  source_asset_id TEXT NOT NULL REFERENCES audio_assets(id),
  source_version_id TEXT NOT NULL REFERENCES audio_asset_versions(id),
  saved_version_id TEXT NOT NULL REFERENCES audio_asset_versions(id),
  recipe_id TEXT NOT NULL REFERENCES generation_recipes(id),
  job_id TEXT NOT NULL REFERENCES generation_jobs(id),
  can_save INTEGER NOT NULL,
  warnings_json TEXT NOT NULL,
  blocking_reasons_json TEXT NOT NULL
);
CREATE TABLE review_version_comparisons (
  id TEXT PRIMARY KEY,
  mode TEXT NOT NULL,
  left_asset_id TEXT NOT NULL REFERENCES audio_assets(id),
  left_version_id TEXT NOT NULL REFERENCES audio_asset_versions(id),
  right_asset_id TEXT NOT NULL REFERENCES audio_assets(id),
  right_version_id TEXT NOT NULL REFERENCES audio_asset_versions(id),
  metrics_json TEXT NOT NULL,
  notes_json TEXT NOT NULL
);
CREATE TABLE review_provenance_links (
  edited_version_id TEXT PRIMARY KEY REFERENCES audio_asset_versions(id),
  source_version_id TEXT NOT NULL REFERENCES audio_asset_versions(id),
  original_recipe_id TEXT NOT NULL REFERENCES generation_recipes(id),
  edit_recipe_id TEXT NOT NULL REFERENCES generation_recipes(id),
  sidecar_path TEXT NOT NULL,
  provenance_ids_json TEXT NOT NULL
);
",
    },
    SchemaMigration {
        version: 12,
        name: "rights_safety_workflow",
        sql: "
CREATE TABLE rights_consent_checks (
  id TEXT PRIMARY KEY,
  workflow TEXT NOT NULL,
  voice_profile_id TEXT NOT NULL,
  consent_status TEXT NOT NULL,
  allowed_use TEXT NOT NULL,
  decision TEXT NOT NULL,
  summary TEXT NOT NULL,
  stored_metadata_json TEXT NOT NULL
);
CREATE TABLE rights_model_use_decisions (
  candidate_id TEXT PRIMARY KEY REFERENCES model_evaluation_candidates(id),
  requested_workflow TEXT NOT NULL,
  commercial_export INTEGER NOT NULL,
  license TEXT NOT NULL,
  commercial_use TEXT NOT NULL,
  product_eligibility TEXT NOT NULL,
  runtime_path TEXT NOT NULL,
  requires_python_runtime INTEGER NOT NULL,
  decision TEXT NOT NULL,
  reasons_json TEXT NOT NULL
);
CREATE TABLE rights_content_policy_gates (
  id TEXT PRIMARY KEY,
  category TEXT NOT NULL,
  status TEXT NOT NULL,
  applies_to_json TEXT NOT NULL,
  summary TEXT NOT NULL,
  enforcement TEXT NOT NULL
);
CREATE TABLE rights_export_sidecars (
  id TEXT PRIMARY KEY,
  asset_id TEXT NOT NULL REFERENCES audio_assets(id),
  asset_kind TEXT NOT NULL,
  target TEXT NOT NULL,
  path TEXT NOT NULL UNIQUE,
  includes_recipe INTEGER NOT NULL,
  includes_model INTEGER NOT NULL,
  includes_source_media INTEGER NOT NULL,
  includes_rights INTEGER NOT NULL,
  includes_edit_chain INTEGER NOT NULL,
  disclosure_required INTEGER NOT NULL,
  watermark TEXT NOT NULL,
  rights_json TEXT NOT NULL,
  provenance_json TEXT NOT NULL
);
CREATE TABLE rights_disclosure_checks (
  id TEXT PRIMARY KEY,
  asset_id TEXT NOT NULL REFERENCES audio_assets(id),
  required INTEGER NOT NULL,
  reason TEXT NOT NULL,
  export_targets_json TEXT NOT NULL
);
CREATE TABLE rights_validation_checks (
  id TEXT PRIMARY KEY,
  status TEXT NOT NULL,
  summary TEXT NOT NULL
);
",
    },
    SchemaMigration {
        version: 13,
        name: "asset_library_workflow",
        sql: "
CREATE TABLE asset_library_items (
  item_id TEXT PRIMARY KEY,
  item_type TEXT NOT NULL,
  asset_id TEXT REFERENCES audio_assets(id),
  scope_kind TEXT NOT NULL,
  project_id TEXT,
  ownership TEXT NOT NULL,
  created_at TEXT NOT NULL,
  source_workflow TEXT,
  favorite INTEGER NOT NULL,
  rejected INTEGER NOT NULL,
  archived INTEGER NOT NULL,
  timeline_placeable INTEGER NOT NULL,
  source_picker_eligible INTEGER NOT NULL,
  composition_usage_count INTEGER NOT NULL DEFAULT 0,
  generated_tags_json TEXT NOT NULL
);
CREATE TABLE asset_library_tags (
  item_id TEXT NOT NULL REFERENCES asset_library_items(item_id),
  tag TEXT NOT NULL,
  system_generated INTEGER NOT NULL,
  PRIMARY KEY(item_id, tag, system_generated)
);
CREATE TABLE asset_library_collections (
  id TEXT PRIMARY KEY REFERENCES collections(id),
  collection_type TEXT NOT NULL,
  description TEXT NOT NULL,
  drag_into_studios_json TEXT NOT NULL
);
CREATE TABLE asset_library_collection_items (
  collection_id TEXT NOT NULL REFERENCES collections(id),
  item_id TEXT NOT NULL REFERENCES asset_library_items(item_id),
  position INTEGER NOT NULL,
  PRIMARY KEY(collection_id, item_id)
);
CREATE TABLE asset_library_saved_filters (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  scope_kind TEXT NOT NULL,
  project_id TEXT,
  query_json TEXT NOT NULL,
  include_rejected INTEGER NOT NULL,
  include_archived INTEGER NOT NULL,
  favorite_only INTEGER NOT NULL
);
CREATE TABLE asset_library_reuse_events (
  id TEXT PRIMARY KEY,
  item_id TEXT NOT NULL REFERENCES asset_library_items(item_id),
  target TEXT NOT NULL,
  creates_linked_copy INTEGER NOT NULL,
  provenance_sidecar_path TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
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
