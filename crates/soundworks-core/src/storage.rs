use crate::domain::{AudioAssetKind, AudioFileFormat, LibraryScope};
use serde::{Deserialize, Serialize};
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;

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

/// Join caller-influenced `segments` under `root`, validating each segment with
/// [`clean_path_segment`] so path traversal (`..`, `/`, `\`) is structurally
/// impossible, then asserting — as defense-in-depth against symlinks in the
/// existing tree — that the resolved path stays under `root`.
///
/// This is the single choke point every real read/write path that incorporates an
/// externally supplied identifier (`item_id`, `project_id`, `version_id`, `job_id`,
/// `export_id`) must route through.
pub fn sanitized_join(root: &Path, segments: &[&str]) -> io::Result<PathBuf> {
    let mut path = root.to_path_buf();
    for segment in segments {
        let clean = clean_path_segment(segment)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error.to_string()))?;
        path.push(clean);
    }
    // Segments are already free of separators and `..`, so a freshly built path is
    // contained by construction. Canonicalizing the existing portion additionally
    // rejects escapes through a pre-existing symlink. Canonicalization is skipped
    // (not failed) when the root does not yet exist on disk.
    if let (Ok(canonical_root), Ok(canonical_existing)) =
        (root.canonicalize(), deepest_existing(&path).canonicalize())
    {
        if !canonical_existing.starts_with(&canonical_root) {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!("path escapes storage root: {}", path.display()),
            ));
        }
    }
    Ok(path)
}

/// Walk up from `path` to the deepest ancestor that exists on disk, so a target
/// that has not been created yet can still be containment-checked via its parent.
fn deepest_existing(path: &Path) -> &Path {
    let mut current = path;
    while !current.exists() {
        match current.parent() {
            Some(parent) => current = parent,
            None => return current,
        }
    }
    current
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

#[cfg(test)]
mod tests {
    use super::sanitized_join;

    fn temp_root(label: &str) -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!(
            "soundworks-sanitize-{label}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create test root");
        root
    }

    #[test]
    fn sanitized_join_accepts_clean_segments_under_root() {
        let root = temp_root("clean");
        let joined = sanitized_join(&root, &["assets", "asset-001", "asset-record.json"])
            .expect("clean segments are accepted");
        assert!(joined.starts_with(&root));
        assert!(joined.ends_with("assets/asset-001/asset-record.json"));
    }

    #[test]
    fn sanitized_join_rejects_traversal_and_separators() {
        let root = temp_root("reject");
        for bad in [
            "..",
            ".",
            "../etc",
            "../../etc/passwd",
            "a/b",
            "a\\b",
            "",
            "with space",
            "tab\tchar",
        ] {
            assert!(
                sanitized_join(&root, &[bad]).is_err(),
                "segment {bad:?} must be rejected"
            );
        }
    }

    #[test]
    fn sanitized_join_rejects_traversal_in_any_position() {
        let root = temp_root("position");
        assert!(sanitized_join(&root, &["assets", "..", "asset-record.json"]).is_err());
        assert!(sanitized_join(&root, &["assets", "../../escape"]).is_err());
    }
}
