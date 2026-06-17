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
