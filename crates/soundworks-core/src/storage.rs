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
  export_candidate INTEGER NOT NULL,
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
    SchemaMigration {
        version: 14,
        name: "export_workflow",
        sql: "
CREATE TABLE export_presets (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  target TEXT NOT NULL,
  default_format TEXT NOT NULL,
  formats_json TEXT NOT NULL,
  source_kinds_json TEXT NOT NULL,
  asset_kinds_json TEXT NOT NULL,
  package_artifacts_json TEXT NOT NULL,
  sample_rate_hz INTEGER NOT NULL,
  bit_depth INTEGER,
  include_sidecar INTEGER NOT NULL,
  include_stems INTEGER NOT NULL,
  normalize_loudness INTEGER NOT NULL,
  target_lufs REAL,
  preserve_loop_metadata INTEGER NOT NULL,
  preserve_bpm_key_metadata INTEGER NOT NULL
);
CREATE TABLE export_submissions (
  id TEXT PRIMARY KEY,
  preset_id TEXT NOT NULL REFERENCES export_presets(id),
  source_kind TEXT NOT NULL,
  source_id TEXT NOT NULL,
  asset_ids_json TEXT NOT NULL,
  collection_ids_json TEXT NOT NULL,
  formats_json TEXT NOT NULL,
  can_export INTEGER NOT NULL,
  blocking_reasons_json TEXT NOT NULL,
  warnings_json TEXT NOT NULL,
  output_paths_json TEXT NOT NULL,
  sidecar_path TEXT NOT NULL
);
CREATE TABLE export_sidecars (
  id TEXT PRIMARY KEY,
  submission_id TEXT NOT NULL REFERENCES export_submissions(id),
  asset_id TEXT NOT NULL,
  asset_kind TEXT NOT NULL,
  target TEXT NOT NULL,
  path TEXT NOT NULL UNIQUE,
  includes_recipe INTEGER NOT NULL,
  includes_model INTEGER NOT NULL,
  includes_source_media INTEGER NOT NULL,
  includes_rights INTEGER NOT NULL,
  includes_edit_chain INTEGER NOT NULL,
  disclosure_required INTEGER NOT NULL,
  event_count INTEGER NOT NULL
);
CREATE TABLE export_daw_handoffs (
  id TEXT PRIMARY KEY,
  preset_id TEXT NOT NULL REFERENCES export_presets(id),
  package_path TEXT NOT NULL UNIQUE,
  normalized_filename_template TEXT NOT NULL,
  includes_zip_bundle INTEGER NOT NULL,
  includes_stems INTEGER NOT NULL,
  includes_cue_markers INTEGER NOT NULL,
  includes_loop_markers INTEGER NOT NULL,
  includes_bpm_key_metadata INTEGER NOT NULL,
  includes_lyrics_text INTEGER NOT NULL,
  includes_midi INTEGER NOT NULL,
  stem_kinds_json TEXT NOT NULL
);
CREATE TABLE export_sceneworks_handoffs (
  id TEXT PRIMARY KEY,
  preset_id TEXT NOT NULL REFERENCES export_presets(id),
  package_path TEXT NOT NULL UNIQUE,
  rendered_mixdown_path TEXT NOT NULL,
  package_manifest_path TEXT NOT NULL,
  provenance_sidecar_path TEXT NOT NULL,
  includes_optional_stems INTEGER NOT NULL,
  optional_stem_paths_json TEXT NOT NULL,
  import_strategy TEXT NOT NULL,
  attachment_mode TEXT NOT NULL,
  intended_project_id TEXT,
  intended_video_asset_id TEXT,
  scene_works_project_path TEXT,
  target_video_sidecar_path TEXT,
  scene_works_asset_type TEXT NOT NULL,
  scene_works_mime_type TEXT NOT NULL,
  duration_ms INTEGER NOT NULL,
  target_video_duration_ms INTEGER NOT NULL,
  start_offset_ms INTEGER NOT NULL,
  sample_rate_hz INTEGER NOT NULL,
  channels INTEGER NOT NULL,
  loudness_lufs REAL,
  true_peak_dbfs REAL,
  marker_count INTEGER NOT NULL,
  section_count INTEGER NOT NULL,
  replace_existing_audio INTEGER NOT NULL,
  round_trip_recipe_url TEXT NOT NULL,
  source_evidence_json TEXT NOT NULL,
  compatibility_checks_json TEXT NOT NULL,
  attachment_steps_json TEXT NOT NULL
);
",
    },
    SchemaMigration {
        version: 15,
        name: "mvp_validation_matrix",
        sql: "
CREATE TABLE mvp_validation_demo_workflows (
  id TEXT PRIMARY KEY,
  workflow TEXT NOT NULL,
  title TEXT NOT NULL,
  goal TEXT NOT NULL,
  required_artifacts_json TEXT NOT NULL,
  acceptance_json TEXT NOT NULL
);
CREATE TABLE mvp_validation_regression_fixtures (
  id TEXT PRIMARY KEY,
  workflow TEXT NOT NULL,
  name TEXT NOT NULL,
  input_contract TEXT NOT NULL,
  expected_outputs_json TEXT NOT NULL,
  automated_check_ids_json TEXT NOT NULL
);
CREATE TABLE mvp_validation_checks (
  id TEXT PRIMARY KEY,
  category TEXT NOT NULL,
  status TEXT NOT NULL,
  required_for_mvp INTEGER NOT NULL,
  summary TEXT NOT NULL,
  evidence TEXT NOT NULL
);
CREATE TABLE mvp_validation_manual_scorecards (
  id TEXT PRIMARY KEY,
  workflow TEXT NOT NULL,
  status TEXT NOT NULL,
  required_for_mvp INTEGER NOT NULL,
  scoring_axes_json TEXT NOT NULL,
  pass_threshold TEXT NOT NULL,
  reviewer_notes TEXT NOT NULL
);
CREATE TABLE mvp_validation_stress_cases (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  workflow TEXT NOT NULL,
  status TEXT NOT NULL,
  required_for_mvp INTEGER NOT NULL,
  scenario TEXT NOT NULL,
  expected_behavior TEXT NOT NULL
);
CREATE TABLE mvp_validation_known_limitations (
  id TEXT PRIMARY KEY,
  area TEXT NOT NULL,
  summary TEXT NOT NULL,
  mitigation TEXT NOT NULL,
  blocks_mvp INTEGER NOT NULL
);
CREATE TABLE mvp_validation_requirement_coverage (
  requirement_id TEXT PRIMARY KEY,
  epic_requirement TEXT NOT NULL,
  demo_workflow_ids_json TEXT NOT NULL,
  fixture_ids_json TEXT NOT NULL,
  check_ids_json TEXT NOT NULL,
  status TEXT NOT NULL
);
CREATE TABLE mvp_validation_release_gates (
  id TEXT PRIMARY KEY,
  ready_for_mvp INTEGER NOT NULL,
  required_workflow_count INTEGER NOT NULL,
  covered_workflow_count INTEGER NOT NULL,
  required_automated_check_count INTEGER NOT NULL,
  passed_automated_check_count INTEGER NOT NULL,
  required_manual_scorecard_count INTEGER NOT NULL,
  passed_manual_scorecard_count INTEGER NOT NULL,
  required_stress_case_count INTEGER NOT NULL,
  passed_stress_case_count INTEGER NOT NULL,
  blocking_items_json TEXT NOT NULL
);
",
    },
    SchemaMigration {
        version: 16,
        name: "composition_editor_workflow",
        sql: "
CREATE TABLE composition_editor_sessions (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES projects(id),
  composition_id TEXT NOT NULL REFERENCES compositions(id),
  selected_tool TEXT NOT NULL,
  selected_clip_id TEXT NOT NULL,
  playback_cursor_ms INTEGER NOT NULL,
  zoom_percent INTEGER NOT NULL,
  snap_grid_ms INTEGER NOT NULL,
  loop_enabled INTEGER NOT NULL,
  loop_range_json TEXT NOT NULL
);
CREATE TABLE composition_editor_tracks (
  id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL REFERENCES composition_editor_sessions(id),
  track_id TEXT NOT NULL,
  role TEXT NOT NULL,
  gain_db REAL NOT NULL,
  pan REAL NOT NULL,
  muted INTEGER NOT NULL,
  soloed INTEGER NOT NULL,
  effect_chain_json TEXT NOT NULL,
  send_targets_json TEXT NOT NULL
);
CREATE TABLE composition_editor_clips (
  id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL REFERENCES composition_editor_sessions(id),
  track_id TEXT NOT NULL,
  asset_id TEXT NOT NULL REFERENCES audio_assets(id),
  version_id TEXT NOT NULL REFERENCES audio_asset_versions(id),
  source_scope_json TEXT NOT NULL,
  timeline_start_ms INTEGER NOT NULL,
  source_range_json TEXT NOT NULL,
  fade_in_ms INTEGER NOT NULL,
  fade_out_ms INTEGER NOT NULL,
  gain_db REAL NOT NULL,
  pan REAL NOT NULL,
  lane INTEGER NOT NULL,
  edit_capabilities_json TEXT NOT NULL
);
CREATE TABLE composition_editor_mixer_state (
  session_id TEXT PRIMARY KEY REFERENCES composition_editor_sessions(id),
  master_gain_db REAL NOT NULL,
  target_lufs REAL NOT NULL,
  true_peak_ceiling_dbfs REAL NOT NULL,
  render_ready INTEGER NOT NULL,
  loudness_check TEXT NOT NULL,
  warnings_json TEXT NOT NULL
);
CREATE TABLE composition_editor_component_decisions (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  source_url TEXT NOT NULL,
  license TEXT NOT NULL,
  fit TEXT NOT NULL,
  strengths_json TEXT NOT NULL,
  risks_json TEXT NOT NULL,
  prototype_evidence TEXT NOT NULL,
  decision TEXT NOT NULL
);
CREATE TABLE composition_editor_render_plans (
  session_id TEXT PRIMARY KEY REFERENCES composition_editor_sessions(id),
  can_render_mixdown INTEGER NOT NULL,
  preset_ids_json TEXT NOT NULL,
  mixdown_path TEXT NOT NULL,
  stem_paths_json TEXT NOT NULL,
  provenance_sidecar_path TEXT NOT NULL,
  required_provenance_fields_json TEXT NOT NULL,
  scene_works_ready INTEGER NOT NULL,
  scene_works_warning TEXT NOT NULL
);
",
    },
    SchemaMigration {
        version: 17,
        name: "video_to_audio_workflow",
        sql: "
CREATE TABLE video_to_audio_sources (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES projects(id),
  video_asset_id TEXT NOT NULL REFERENCES audio_assets(id),
  filename TEXT NOT NULL,
  duration_ms INTEGER NOT NULL,
  frame_rate TEXT NOT NULL,
  resolution TEXT NOT NULL,
  has_source_audio INTEGER NOT NULL,
  image_reference_ids_json TEXT NOT NULL,
  reference_audio_asset_ids_json TEXT NOT NULL,
  ownership_attestation TEXT NOT NULL
);
CREATE TABLE video_to_audio_target_ranges (
  id TEXT PRIMARY KEY,
  source_id TEXT NOT NULL REFERENCES video_to_audio_sources(id),
  label TEXT NOT NULL,
  start_ms INTEGER NOT NULL,
  end_ms INTEGER NOT NULL,
  object_label TEXT,
  region_json TEXT,
  requested_action TEXT NOT NULL
);
CREATE TABLE video_to_audio_detected_events (
  id TEXT PRIMARY KEY,
  source_id TEXT NOT NULL REFERENCES video_to_audio_sources(id),
  label TEXT NOT NULL,
  at_ms INTEGER NOT NULL,
  confidence REAL NOT NULL,
  object_label TEXT,
  region_json TEXT,
  requested_sound TEXT NOT NULL
);
CREATE TABLE video_to_audio_provider_scorecards (
  candidate_id TEXT PRIMARY KEY REFERENCES model_evaluation_candidates(id),
  readiness TEXT NOT NULL,
  recommended INTEGER NOT NULL,
  supports_json TEXT NOT NULL,
  blockers_json TEXT NOT NULL,
  notes TEXT NOT NULL
);
CREATE TABLE video_to_audio_submissions (
  id TEXT PRIMARY KEY,
  source_id TEXT NOT NULL REFERENCES video_to_audio_sources(id),
  provider_id TEXT NOT NULL,
  model_id TEXT NOT NULL,
  recipe_id TEXT NOT NULL REFERENCES generation_recipes(id),
  job_id TEXT NOT NULL REFERENCES generation_jobs(id),
  can_submit INTEGER NOT NULL,
  blocking_reasons_json TEXT NOT NULL,
  warnings_json TEXT NOT NULL
);
CREATE TABLE video_to_audio_sync_previews (
  id TEXT PRIMARY KEY,
  submission_id TEXT NOT NULL REFERENCES video_to_audio_submissions(id),
  duration_ms INTEGER NOT NULL,
  sample_rate_hz INTEGER NOT NULL,
  channel_layout TEXT NOT NULL,
  waveform_preview_path TEXT NOT NULL,
  sync_points_json TEXT NOT NULL,
  segments_json TEXT NOT NULL,
  warnings_json TEXT NOT NULL
);
CREATE TABLE video_to_audio_saved_outputs (
  submission_id TEXT PRIMARY KEY REFERENCES video_to_audio_submissions(id),
  asset_id TEXT NOT NULL REFERENCES audio_assets(id),
  version_id TEXT NOT NULL REFERENCES audio_asset_versions(id),
  waveform_preview_ready INTEGER NOT NULL,
  synchronized_to_video INTEGER NOT NULL
);
CREATE TABLE video_to_audio_export_packages (
  id TEXT PRIMARY KEY,
  submission_id TEXT NOT NULL REFERENCES video_to_audio_submissions(id),
  mixdown_path TEXT NOT NULL,
  sidecar_path TEXT NOT NULL,
  destination_targets_json TEXT NOT NULL,
  required_fields_json TEXT NOT NULL,
  includes_sync_points INTEGER NOT NULL,
  includes_source_media_refs INTEGER NOT NULL,
  includes_detected_events INTEGER NOT NULL,
  includes_rights INTEGER NOT NULL
);
CREATE TABLE video_to_audio_safety_gates (
  id TEXT PRIMARY KEY,
  status TEXT NOT NULL,
  summary TEXT NOT NULL,
  enforcement TEXT NOT NULL
);
",
    },
    SchemaMigration {
        version: 18,
        name: "workspace_global_library",
        sql: "
CREATE TABLE workspace_records (
  id TEXT PRIMARY KEY,
  global_library_id TEXT NOT NULL,
  recent_project_ids_json TEXT NOT NULL,
  active_project_id TEXT NOT NULL REFERENCES projects(id)
);
CREATE TABLE workspace_project_cards (
  project_id TEXT PRIMARY KEY REFERENCES projects(id),
  opened_at TEXT NOT NULL,
  asset_count INTEGER NOT NULL,
  composition_count INTEGER NOT NULL,
  local_recipe_count INTEGER NOT NULL,
  linked_global_asset_count INTEGER NOT NULL,
  can_open INTEGER NOT NULL,
  can_create_from_template INTEGER NOT NULL,
  status TEXT NOT NULL
);
CREATE TABLE workspace_global_libraries (
  id TEXT PRIMARY KEY,
  label TEXT NOT NULL,
  asset_count INTEGER NOT NULL,
  reusable_voice_count INTEGER NOT NULL,
  reusable_preset_count INTEGER NOT NULL,
  reusable_collection_count INTEGER NOT NULL,
  storage_root TEXT NOT NULL,
  can_browse INTEGER NOT NULL
);
CREATE TABLE workspace_scope_controls (
  id TEXT PRIMARY KEY,
  label TEXT NOT NULL,
  scope_kind TEXT NOT NULL,
  project_id TEXT,
  active INTEGER NOT NULL,
  item_count INTEGER NOT NULL,
  empty_state TEXT NOT NULL
);
CREATE TABLE workspace_source_picker_policies (
  id TEXT PRIMARY KEY,
  active_project_id TEXT NOT NULL REFERENCES projects(id),
  default_scope_json TEXT NOT NULL,
  allows_global_sources INTEGER NOT NULL,
  import_modes_json TEXT NOT NULL,
  target_surfaces_json TEXT NOT NULL,
  provenance_requirements_json TEXT NOT NULL
);
CREATE TABLE workspace_transfer_actions (
  id TEXT PRIMARY KEY,
  label TEXT NOT NULL,
  mode TEXT NOT NULL,
  source_item_id TEXT NOT NULL REFERENCES asset_library_items(item_id),
  target_project_id TEXT REFERENCES projects(id),
  target_scope_json TEXT NOT NULL,
  preserves_provenance INTEGER NOT NULL,
  creates_new_asset_id INTEGER NOT NULL,
  creates_reuse_event INTEGER NOT NULL,
  enabled INTEGER NOT NULL,
  summary TEXT NOT NULL
);
CREATE TABLE workspace_composition_asset_links (
  id TEXT PRIMARY KEY,
  composition_id TEXT NOT NULL REFERENCES compositions(id),
  project_id TEXT NOT NULL REFERENCES projects(id),
  asset_id TEXT NOT NULL REFERENCES audio_assets(id),
  version_id TEXT NOT NULL REFERENCES audio_asset_versions(id),
  source_scope_json TEXT NOT NULL,
  project_usage TEXT NOT NULL,
  preserves_original_asset_id INTEGER NOT NULL,
  provenance_sidecar_path TEXT NOT NULL,
  warning TEXT
);
CREATE TABLE workspace_parity_notes (
  id TEXT PRIMARY KEY,
  area TEXT NOT NULL,
  convention TEXT NOT NULL,
  soundworks_application TEXT NOT NULL
);
CREATE TABLE workspace_validation_checks (
  id TEXT PRIMARY KEY,
  passed INTEGER NOT NULL,
  summary TEXT NOT NULL
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
