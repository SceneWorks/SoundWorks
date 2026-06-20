//! Shared building blocks for the six generation studios (tts/sfx/samples/songs/
//! voice_lab/video_to_audio). These types and helpers were previously stamped
//! out per studio (F-013/F-025/F-026); centralizing them keeps one source of
//! truth while preserving each studio's exact output. The serde field
//! names/shapes here match the former per-studio types, so the emitted JSON (and
//! the apps/web mirror) is unchanged.

use crate::domain::{GenerationJob, GenerationRecipe};
use crate::evaluation::{
    CommercialUseEvaluation, EvaluationLane, EvaluationStatus, ProductEligibility,
    ProductRuntimePath,
};
use crate::manifests::{CapabilitySafety, ModelInstall, ModelInstallStatus, ModelLicense};
use serde::{Deserialize, Serialize};

/// Provider readiness scorecard shared by the sfx/samples/songs/voice_lab
/// studios, parameterized over each studio's readiness enum `R`. (video_to_audio
/// carries an extra `supports` field and keeps its own struct; tts has no
/// scorecard.)
///
/// This is a UI/contract surface: it summarizes evaluation evidence for display
/// and is NOT the authoritative job gate (F-021). Real submission is validated
/// by `runtime::RuntimeJobStore::enqueue` (consent + blocked-model gates).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderScorecard<R> {
    pub candidate_id: String,
    pub name: String,
    pub provider: String,
    pub lanes: Vec<EvaluationLane>,
    pub status: EvaluationStatus,
    pub product_eligibility: ProductEligibility,
    pub readiness: R,
    pub runtime_path: ProductRuntimePath,
    pub commercial_use: CommercialUseEvaluation,
    pub recommended: bool,
    pub blockers: Vec<String>,
    pub notes: String,
}

/// Submission readiness preview shared by the tts/sfx/songs/voice_lab studios.
/// (samples emits a batch with plural jobs/recipes and keeps its own struct;
/// video_to_audio has its own submission shape.)
///
/// Like [`ProviderScorecard`], this is a UI contract-preview computed from
/// reference inputs (F-021): `can_submit`/`blocking_reasons` describe display
/// readiness and are NOT the authoritative gate. The real gate is
/// `runtime::RuntimeJobStore::enqueue`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StudioSubmissionPreview {
    pub can_submit: bool,
    pub job: GenerationJob,
    pub recipe: GenerationRecipe,
    pub blocking_reasons: Vec<String>,
    pub warnings: Vec<String>,
}

/// License-derived export limitations. Identical across studios.
pub(crate) fn limitations_for_license(license: ModelLicense) -> Vec<String> {
    match license {
        ModelLicense::Open | ModelLicense::CommercialAllowed | ModelLicense::ProviderTerms => {
            vec![]
        }
        ModelLicense::NonCommercial => {
            vec!["Noncommercial license requires SoundWorks compatibility review.".to_string()]
        }
        ModelLicense::Unknown => {
            vec!["License must be reviewed before production use.".to_string()]
        }
    }
}

/// Which safety conditions a studio surfaces as export limitations. Each studio
/// historically copy-pasted `limitations_for_safety` with a different subset of
/// these checks (F-025); the explicit flags make the per-studio differences
/// visible and preserve the exact push order (consent, commercial-use,
/// provenance, then the shared disallowed-uses line) so output is unchanged.
pub(crate) struct SafetyLimitationOptions {
    pub check_voice_consent: bool,
    pub check_commercial_use: bool,
    pub check_provenance_sidecar: bool,
}

pub(crate) fn limitations_for_safety(
    safety: &CapabilitySafety,
    options: SafetyLimitationOptions,
) -> Vec<String> {
    let mut limitations = vec![];

    if options.check_voice_consent && safety.requires_voice_consent {
        limitations.push("Voice profile consent is required before generation.".to_string());
    }

    if options.check_commercial_use && !safety.commercial_use_allowed {
        limitations.push("Model use terms require review before export.".to_string());
    }

    if options.check_provenance_sidecar && !safety.provenance_sidecar {
        limitations.push("Provider must preserve provenance before export.".to_string());
    }

    if !safety.disallowed_uses.is_empty() {
        limitations.push(format!(
            "Disallowed uses: {}.",
            safety.disallowed_uses.join(", ")
        ));
    }

    limitations
}

/// Whether an installed model is runnable from a studio. Identical across studios.
pub(crate) trait StudioInstallStatus {
    fn is_runnable_for_studio(&self) -> bool;
}

impl StudioInstallStatus for ModelInstall {
    fn is_runnable_for_studio(&self) -> bool {
        matches!(
            self.status,
            ModelInstallStatus::Installed
                | ModelInstallStatus::Packaged
                | ModelInstallStatus::External
        )
    }
}

/// Kebab-case display label for an enum, derived from its serde representation
/// rather than `Debug` formatting (F-026). The studios' display enums all derive
/// `#[serde(rename_all = "kebab-case")]`, so this yields the same string the old
/// `format!("{:?}", x).to_case_label()` produced, without coupling display
/// labels to `Debug` output.
pub(crate) fn kebab_label<T: Serialize>(value: &T) -> String {
    match serde_json::to_value(value) {
        Ok(serde_json::Value::String(label)) => label,
        _ => String::new(),
    }
}
