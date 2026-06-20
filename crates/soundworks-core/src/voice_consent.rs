//! Durable per-profile voice-consent overrides (UX-08).
//!
//! F-003 admits a voice job only when the target profile's recorded consent is
//! `ExplicitConsentRecorded`. Before UX-08, consent was readable only from the
//! fixture catalog (`voice_lab::profile_consent`) — there was no way for a user to
//! *record* consent, so every user-created profile was permanently blocked. This
//! store is the durable write side: a small JSON map of profile id -> consent
//! status, written atomically alongside `ui_preferences.json`, that
//! `voice_lab::profile_consent` consults before falling back to the fixture
//! catalog. The web preview has no backend and simply cannot record consent.

use crate::domain::VoiceConsentStatus;
use crate::ui_preferences::UiPreferencesStore;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// Persisted consent overrides keyed by voice-profile id. A `BTreeMap` keeps the
/// on-disk file deterministic.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceConsentRecords {
    #[serde(default)]
    pub profiles: BTreeMap<String, VoiceConsentStatus>,
}

/// Store for the durable voice-consent file.
#[derive(Debug, Clone)]
pub struct VoiceConsentStore {
    root: PathBuf,
}

impl VoiceConsentStore {
    /// Sits beside `ui_preferences.json` under the SoundWorks app-support base
    /// (overridable via `SOUNDWORKS_VOICE_CONSENT_ROOT` for tests).
    pub fn default_root() -> PathBuf {
        if let Ok(root) = std::env::var("SOUNDWORKS_VOICE_CONSENT_ROOT") {
            return PathBuf::from(root);
        }
        UiPreferencesStore::default_root()
    }

    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn path(&self) -> PathBuf {
        self.root.join("voice_consent.json")
    }

    /// Load persisted records, or defaults when the file is absent or corrupt (a
    /// bad file must never block generation admission at launch).
    pub fn load(&self) -> VoiceConsentRecords {
        match fs::read(self.path()) {
            Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_default(),
            Err(_) => VoiceConsentRecords::default(),
        }
    }

    /// The recorded consent override for a profile, if any.
    pub fn consent_for(&self, profile_id: &str) -> Option<VoiceConsentStatus> {
        self.load().profiles.get(profile_id).copied()
    }

    /// Record a profile's consent status and write the file atomically. Returns
    /// the updated records.
    pub fn record(
        &self,
        profile_id: &str,
        status: VoiceConsentStatus,
    ) -> io::Result<VoiceConsentRecords> {
        let mut records = self.load();
        records.profiles.insert(profile_id.to_string(), status);
        self.write(&records)?;
        Ok(records)
    }

    /// Durable write — temp file + fsync + atomic rename, matching the other
    /// SoundWorks stores so a crash mid-write can never leave a truncated file.
    fn write(&self, records: &VoiceConsentRecords) -> io::Result<()> {
        let path = self.path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let payload = serde_json::to_vec_pretty(records)
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

impl Default for VoiceConsentStore {
    fn default() -> Self {
        Self::new(Self::default_root())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(label: &str) -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("soundworks-consent-{label}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create test root");
        root
    }

    #[test]
    fn consent_for_is_none_when_absent() {
        let store = VoiceConsentStore::new(temp_root("absent"));
        assert_eq!(store.consent_for("profile-x"), None);
    }

    #[test]
    fn record_persists_and_overrides() {
        let store = VoiceConsentStore::new(temp_root("record"));
        store
            .record("profile-x", VoiceConsentStatus::ExplicitConsentRecorded)
            .expect("record consent");
        assert_eq!(
            store.consent_for("profile-x"),
            Some(VoiceConsentStatus::ExplicitConsentRecorded)
        );
        // A later record overwrites the prior status and reloads from disk.
        store
            .record("profile-x", VoiceConsentStatus::Prohibited)
            .expect("record prohibited");
        assert_eq!(
            VoiceConsentStore::new(store.root()).consent_for("profile-x"),
            Some(VoiceConsentStatus::Prohibited)
        );
    }
}
