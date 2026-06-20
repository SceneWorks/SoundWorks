//! Durable UI preferences (theme + accent) for the desktop shell.
//!
//! DR-01 parity: SceneWorks persists the user's theme/accent server-side so the
//! choice survives launches even when the webview's localStorage origin changes.
//! SoundWorks has no server, so the durable copy lives in a small JSON file under
//! the SoundWorks app-support base directory (sibling of `library/` and `models/`),
//! written atomically like the rest of the store. The web preview has no backend
//! and falls back to localStorage only.

use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// Persisted UI preferences. Both fields are optional so a partial update from the
/// UI (which sends only the changed field) deserializes cleanly and merges, and an
/// unset preference is simply omitted from the file. Values are stored verbatim;
/// the UI validates the accent id (`isAccentId`) and theme on read, matching how
/// SceneWorks trusts its server copy.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiPreferences {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accent: Option<String>,
}

/// Store for the durable UI-preferences file.
#[derive(Debug, Clone)]
pub struct UiPreferencesStore {
    root: PathBuf,
}

impl UiPreferencesStore {
    /// `~/Library/Application Support/SoundWorks` (overridable via
    /// `SOUNDWORKS_PREFS_ROOT` for tests / alternate installs).
    pub fn default_root() -> PathBuf {
        if let Ok(root) = std::env::var("SOUNDWORKS_PREFS_ROOT") {
            return PathBuf::from(root);
        }
        std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("Library")
            .join("Application Support")
            .join("SoundWorks")
    }

    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn path(&self) -> PathBuf {
        self.root.join("ui_preferences.json")
    }

    /// Load persisted preferences, or defaults when the file is absent or unreadable
    /// (a corrupt file must never crash launch — the UI falls back to defaults).
    pub fn load(&self) -> UiPreferences {
        match fs::read(self.path()) {
            Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_default(),
            Err(_) => UiPreferences::default(),
        }
    }

    /// Merge a partial update over the persisted preferences and write the result
    /// atomically. Only `Some` fields in `patch` overwrite; `None` keeps the prior
    /// value, so the UI can persist theme and accent independently.
    pub fn merge(&self, patch: UiPreferences) -> io::Result<UiPreferences> {
        let mut current = self.load();
        if patch.theme.is_some() {
            current.theme = patch.theme;
        }
        if patch.accent.is_some() {
            current.accent = patch.accent;
        }
        self.write(&current)?;
        Ok(current)
    }

    /// Durable write — temp file + fsync + atomic rename, mirroring the project /
    /// runtime stores, so a crash mid-write can never leave a truncated file.
    fn write(&self, prefs: &UiPreferences) -> io::Result<()> {
        let path = self.path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let payload = serde_json::to_vec_pretty(prefs)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        let mut temp = path.as_os_str().to_os_string();
        temp.push(".tmp");
        let temp = PathBuf::from(temp);
        {
            let mut file = fs::File::create(&temp)?;
            file.write_all(&payload)?;
            file.sync_all()?;
        }
        fs::rename(&temp, &path)?;
        if let Some(parent) = path.parent() {
            if let Ok(dir) = fs::File::open(parent) {
                let _ = dir.sync_all();
            }
        }
        Ok(())
    }
}

impl Default for UiPreferencesStore {
    fn default() -> Self {
        Self::new(Self::default_root())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(label: &str) -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("soundworks-prefs-{label}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create test root");
        root
    }

    #[test]
    fn load_returns_default_when_absent() {
        let store = UiPreferencesStore::new(temp_root("absent"));
        assert_eq!(store.load(), UiPreferences::default());
    }

    #[test]
    fn merge_persists_and_partially_updates() {
        let store = UiPreferencesStore::new(temp_root("merge"));

        let after_theme = store
            .merge(UiPreferences {
                theme: Some("dark".into()),
                accent: None,
            })
            .expect("merge theme");
        assert_eq!(after_theme.theme.as_deref(), Some("dark"));
        assert_eq!(after_theme.accent, None);

        // A later accent-only update must preserve the previously stored theme.
        let after_accent = store
            .merge(UiPreferences {
                theme: None,
                accent: Some("violet".into()),
            })
            .expect("merge accent");
        assert_eq!(after_accent.theme.as_deref(), Some("dark"));
        assert_eq!(after_accent.accent.as_deref(), Some("violet"));

        // The merged state is what reloads from disk.
        assert_eq!(store.load(), after_accent);
    }

    #[test]
    fn write_leaves_no_temp_file() {
        let root = temp_root("notmp");
        let store = UiPreferencesStore::new(&root);
        store
            .merge(UiPreferences {
                theme: Some("light".into()),
                accent: None,
            })
            .expect("merge");
        let leftovers: Vec<_> = fs::read_dir(&root)
            .expect("read dir")
            .filter_map(Result::ok)
            .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "tmp"))
            .collect();
        assert!(
            leftovers.is_empty(),
            "no .tmp files should remain: {leftovers:?}"
        );
    }
}
