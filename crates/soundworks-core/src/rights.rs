use crate::domain::{
    AudioAssetKind, CommercialUseStatus, ExportTarget, LicenseStatus, ProvenanceEvent,
    ProvenanceEventType, ProvenanceRecord, RightsMetadata, VoiceConsentStatus, WatermarkStatus,
};
use crate::evaluation::{
    CommercialUseEvaluation, EvaluationStatus, ModelEvaluationCatalog, ProductEligibility,
    ProductRuntimePath,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;

pub const RIGHTS_SAFETY_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RightsSafetyOverview {
    pub schema_version: u32,
    pub policy: RightsPolicy,
    pub consent_checks: Vec<ConsentCheck>,
    pub model_use_decisions: Vec<ModelUseDecision>,
    pub content_policy_gates: Vec<ContentPolicyGate>,
    pub export_sidecars: Vec<ProvenanceSidecar>,
    pub disclosure_checks: Vec<DisclosureCheck>,
    pub validation_checks: Vec<RightsValidationCheck>,
}

impl RightsSafetyOverview {
    pub fn reference() -> Self {
        Self::from_catalog(&ModelEvaluationCatalog::reference())
    }

    pub fn from_catalog(catalog: &ModelEvaluationCatalog) -> Self {
        Self {
            schema_version: RIGHTS_SAFETY_SCHEMA_VERSION,
            policy: RightsPolicy::reference(),
            consent_checks: consent_checks(),
            model_use_decisions: model_use_decisions(catalog),
            content_policy_gates: content_policy_gates(),
            export_sidecars: export_sidecars(),
            disclosure_checks: disclosure_checks(),
            validation_checks: validation_checks(),
        }
    }

    pub fn can_export(&self) -> bool {
        self.content_policy_gates
            .iter()
            .all(|gate| gate.status != PolicyGateStatus::Blocked)
            && self
                .model_use_decisions
                .iter()
                .filter(|decision| decision.export_candidate)
                .all(|decision| decision.decision != PolicyDecision::Blocked)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RightsPolicy {
    pub name: String,
    pub voice_consent_required_for: Vec<String>,
    pub export_requires: Vec<String>,
    pub blocked_prompt_categories: Vec<RiskCategory>,
    pub warning_prompt_categories: Vec<RiskCategory>,
    pub watermark_policy: WatermarkPolicy,
    pub provenance_sidecar_required: bool,
}

impl RightsPolicy {
    fn reference() -> Self {
        Self {
            name: "SoundWorks launch rights policy".to_string(),
            voice_consent_required_for: vec![
                "voice-clone".to_string(),
                "voice-conversion".to_string(),
                "few-shot-fine-tune".to_string(),
            ],
            export_requires: vec![
                "explicit voice consent when voice material is used".to_string(),
                "SoundWorks non-commercial use compatibility or provider-terms-reviewed model license".to_string(),
                "provenance sidecar with model, prompt, source media, recipe, and edit chain"
                    .to_string(),
                "AI disclosure flag when generated or AI-edited audio leaves SoundWorks"
                    .to_string(),
            ],
            blocked_prompt_categories: vec![
                RiskCategory::PublicFigureVoiceClone,
                RiskCategory::UnauthorizedVoiceReference,
                RiskCategory::IncompatibleModelLicense,
            ],
            warning_prompt_categories: vec![
                RiskCategory::ArtistStyleImitation,
                RiskCategory::CopyrightedLyrics,
                RiskCategory::WatermarkUnavailable,
            ],
            watermark_policy: WatermarkPolicy::AdvisoryUntilProviderSupport,
            provenance_sidecar_required: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WatermarkPolicy {
    AdvisoryUntilProviderSupport,
    RequiredForExport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RiskCategory {
    VoiceConsent,
    PublicFigureVoiceClone,
    UnauthorizedVoiceReference,
    ArtistStyleImitation,
    CopyrightedLyrics,
    IncompatibleModelLicense,
    AiDisclosure,
    WatermarkUnavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsentCheck {
    pub id: String,
    pub workflow: String,
    pub voice_profile_id: String,
    pub consent_status: VoiceConsentStatus,
    pub allowed_use: String,
    pub decision: PolicyDecision,
    pub summary: String,
    pub stored_metadata: RightsMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelUseDecision {
    pub candidate_id: String,
    pub name: String,
    pub requested_workflow: String,
    pub export_candidate: bool,
    pub license: String,
    pub commercial_use: CommercialUseEvaluation,
    pub product_eligibility: ProductEligibility,
    pub runtime_path: ProductRuntimePath,
    pub requires_python_runtime: bool,
    pub decision: PolicyDecision,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentPolicyGate {
    pub id: String,
    pub category: RiskCategory,
    pub status: PolicyGateStatus,
    pub applies_to: Vec<String>,
    pub summary: String,
    pub enforcement: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PolicyGateStatus {
    Passed,
    Warning,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PolicyDecision {
    Allowed,
    Warn,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProvenanceSidecar {
    pub id: String,
    pub asset_id: String,
    pub asset_kind: AudioAssetKind,
    pub target: ExportTarget,
    pub path: String,
    pub includes_recipe: bool,
    pub includes_model: bool,
    pub includes_source_media: bool,
    pub includes_rights: bool,
    pub includes_edit_chain: bool,
    pub disclosure_required: bool,
    pub watermark: WatermarkStatus,
    pub rights: RightsMetadata,
    pub provenance: ProvenanceRecord,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisclosureCheck {
    pub id: String,
    pub asset_id: String,
    pub required: bool,
    pub reason: String,
    pub export_targets: Vec<ExportTarget>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RightsValidationCheck {
    pub id: String,
    pub status: RightsValidationStatus,
    pub summary: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RightsValidationStatus {
    Passed,
    Warning,
    Failed,
}

fn consent_checks() -> Vec<ConsentCheck> {
    vec![
        ConsentCheck {
            id: "consent.voice-clone.narrator".to_string(),
            workflow: "voice-clone".to_string(),
            voice_profile_id: "voice-profile-narrator".to_string(),
            consent_status: VoiceConsentStatus::ExplicitConsentRecorded,
            allowed_use: "approved voice clone and conversion".to_string(),
            decision: PolicyDecision::Allowed,
            summary: "Narrator profile can queue clone, fine-tune, and conversion workflows because explicit consent metadata is stored."
                .to_string(),
            stored_metadata: RightsMetadata {
                license_status: LicenseStatus::UserOwned,
                commercial_use: CommercialUseStatus::Allowed,
                voice_consent: VoiceConsentStatus::ExplicitConsentRecorded,
                ai_disclosure_required: true,
                watermark: WatermarkStatus::SidecarOnly,
                reference_media_ownership: Some("speaker-signed profile release".to_string()),
            },
        },
        ConsentCheck {
            id: "consent.voice-conversion.guest".to_string(),
            workflow: "voice-conversion".to_string(),
            voice_profile_id: "voice-profile-guest-review".to_string(),
            consent_status: VoiceConsentStatus::RequiresReview,
            allowed_use: "review-only voice conversion".to_string(),
            decision: PolicyDecision::Blocked,
            summary: "Guest voice conversion is blocked until the speaker consent record is completed."
                .to_string(),
            stored_metadata: RightsMetadata {
                license_status: LicenseStatus::Unknown,
                commercial_use: CommercialUseStatus::RequiresReview,
                voice_consent: VoiceConsentStatus::RequiresReview,
                ai_disclosure_required: true,
                watermark: WatermarkStatus::Unsupported,
                reference_media_ownership: Some("pending speaker attestation".to_string()),
            },
        },
        ConsentCheck {
            id: "consent.public-figure.clone".to_string(),
            workflow: "voice-clone".to_string(),
            voice_profile_id: "voice-profile-public-figure-blocked".to_string(),
            consent_status: VoiceConsentStatus::Prohibited,
            allowed_use: "none".to_string(),
            decision: PolicyDecision::Blocked,
            summary: "Public-figure or celebrity voice cloning is blocked rather than queued for review."
                .to_string(),
            stored_metadata: RightsMetadata {
                license_status: LicenseStatus::Restricted,
                commercial_use: CommercialUseStatus::Disallowed,
                voice_consent: VoiceConsentStatus::Prohibited,
                ai_disclosure_required: true,
                watermark: WatermarkStatus::Unsupported,
                reference_media_ownership: Some("unauthorized identity reference".to_string()),
            },
        },
    ]
}

fn model_use_decisions(catalog: &ModelEvaluationCatalog) -> Vec<ModelUseDecision> {
    catalog
        .candidates
        .iter()
        .map(|candidate| {
            let mut reasons = vec![];
            let mut decision = PolicyDecision::Allowed;

            match candidate.license.commercial_use {
                CommercialUseEvaluation::Allowed => reasons.push(
                    "License evidence supports SoundWorks export consideration.".to_string(),
                ),
                CommercialUseEvaluation::ProviderTerms => {
                    decision = PolicyDecision::Warn;
                    reasons.push(
                        "Provider terms must be reviewed and attached before SoundWorks export."
                            .to_string(),
                    );
                }
                CommercialUseEvaluation::NonCommercial => {
                    reasons.push(
                        "Noncommercial model terms fit SoundWorks' non-commercial posture when other export gates pass."
                            .to_string(),
                    );
                }
                CommercialUseEvaluation::Unknown => {
                    decision = PolicyDecision::Blocked;
                    reasons.push(
                        "Unknown model-use terms block SoundWorks export until reviewed."
                            .to_string(),
                    );
                }
            }

            if candidate.product_eligibility != ProductEligibility::ProductCandidate {
                decision = PolicyDecision::Blocked;
                reasons.push(
                    "Only product-candidate models with a cleared runtime path can be SoundWorks export choices."
                        .to_string(),
                );
            }

            if candidate.runtime.requires_python_runtime {
                decision = PolicyDecision::Blocked;
                reasons.push(
                    "Python runtime dependency is not allowed in shipped SoundWorks export paths."
                        .to_string(),
                );
            }

            if candidate.status == EvaluationStatus::Blocked {
                decision = PolicyDecision::Blocked;
                reasons.push("Evaluation status is blocked.".to_string());
            }

            ModelUseDecision {
                candidate_id: candidate.id.clone(),
                name: candidate.name.clone(),
                requested_workflow: candidate
                    .lanes
                    .first()
                    .map(|lane| format!("{lane:?}"))
                    .unwrap_or_else(|| "Unknown".to_string()),
                export_candidate: decision != PolicyDecision::Blocked,
                license: candidate.license.label.clone(),
                commercial_use: candidate.license.commercial_use,
                product_eligibility: candidate.product_eligibility,
                runtime_path: candidate.runtime.product_path,
                requires_python_runtime: candidate.runtime.requires_python_runtime,
                decision,
                reasons,
            }
        })
        .collect()
}

fn content_policy_gates() -> Vec<ContentPolicyGate> {
    vec![
        ContentPolicyGate {
            id: "gate.voice.public-figure".to_string(),
            category: RiskCategory::PublicFigureVoiceClone,
            status: PolicyGateStatus::Blocked,
            applies_to: vec!["voice-clone".to_string(), "voice-conversion".to_string()],
            summary: "Public-figure or celebrity voice imitation cannot be submitted.".to_string(),
            enforcement: "Disable generation and require a new, consented voice profile."
                .to_string(),
        },
        ContentPolicyGate {
            id: "gate.voice.reference-rights".to_string(),
            category: RiskCategory::UnauthorizedVoiceReference,
            status: PolicyGateStatus::Blocked,
            applies_to: vec!["source-voice".to_string(), "reference-audio".to_string()],
            summary: "Voice references without owner attestation are blocked before queueing."
                .to_string(),
            enforcement: "Require consent metadata on the voice profile and generated output."
                .to_string(),
        },
        ContentPolicyGate {
            id: "gate.music.style-imitation".to_string(),
            category: RiskCategory::ArtistStyleImitation,
            status: PolicyGateStatus::Warning,
            applies_to: vec![
                "song".to_string(),
                "loop".to_string(),
                "instrument-sample".to_string(),
            ],
            summary: "Artist/style imitation prompts require visible review and provenance notes."
                .to_string(),
            enforcement: "Warn before generation and include the reviewed prompt in the sidecar."
                .to_string(),
        },
        ContentPolicyGate {
            id: "gate.music.copyrighted-lyrics".to_string(),
            category: RiskCategory::CopyrightedLyrics,
            status: PolicyGateStatus::Warning,
            applies_to: vec!["song".to_string()],
            summary: "Copyrighted or third-party lyrics require rights review before export."
                .to_string(),
            enforcement: "Allow draft generation only; block export until cleared.".to_string(),
        },
        ContentPolicyGate {
            id: "gate.disclosure.ai-audio".to_string(),
            category: RiskCategory::AiDisclosure,
            status: PolicyGateStatus::Passed,
            applies_to: vec!["export".to_string(), "sidecar".to_string()],
            summary: "Generated and edited audio carries an AI disclosure flag in export metadata."
                .to_string(),
            enforcement: "Write disclosureRequired=true into every generated export sidecar."
                .to_string(),
        },
    ]
}

fn export_sidecars() -> Vec<ProvenanceSidecar> {
    vec![
        ProvenanceSidecar {
            id: "sidecar-voice-export".to_string(),
            asset_id: "asset-voice-lab-conversion-reference".to_string(),
            asset_kind: AudioAssetKind::VoiceClip,
            target: ExportTarget::AudioFile,
            path: "soundworks-library/projects/project-demo/voice-clips/asset-voice-lab-conversion-reference/version-voice-lab-conversion-reference-a/metadata/recipe-provenance.json"
                .to_string(),
            includes_recipe: true,
            includes_model: true,
            includes_source_media: true,
            includes_rights: true,
            includes_edit_chain: false,
            disclosure_required: true,
            watermark: WatermarkStatus::SidecarOnly,
            rights: RightsMetadata {
                license_status: LicenseStatus::UserOwned,
                commercial_use: CommercialUseStatus::Allowed,
                voice_consent: VoiceConsentStatus::ExplicitConsentRecorded,
                ai_disclosure_required: true,
                watermark: WatermarkStatus::SidecarOnly,
                reference_media_ownership: Some(
                    "speaker-signed target profile plus user-owned source audio".to_string(),
                ),
            },
            provenance: provenance_record(
                "provenance-voice-export",
                "asset-voice-lab-conversion-reference",
                vec![
                    ("rights-reviewed", ProvenanceEventType::RightsReviewed, "Explicit voice consent and model-use rights checked."),
                    ("generated", ProvenanceEventType::Generated, "RVC-style conversion recipe and source audio IDs attached."),
                    ("exported", ProvenanceEventType::Exported, "WAV export wrote recipe, model, source media, rights, and disclosure metadata."),
                ],
            ),
        },
        ProvenanceSidecar {
            id: "sidecar-song-stem-export".to_string(),
            asset_id: "asset-song-city-lights-full".to_string(),
            asset_kind: AudioAssetKind::Song,
            target: ExportTarget::StemFolder,
            path: "soundworks-library/projects/project-demo/songs/asset-song-city-lights-full/version-song-city-lights-full-a/metadata/recipe-provenance.json"
                .to_string(),
            includes_recipe: true,
            includes_model: true,
            includes_source_media: true,
            includes_rights: true,
            includes_edit_chain: true,
            disclosure_required: true,
            watermark: WatermarkStatus::SidecarOnly,
            rights: RightsMetadata {
                license_status: LicenseStatus::ProviderLicensed,
                commercial_use: CommercialUseStatus::RequiresReview,
                voice_consent: VoiceConsentStatus::NotVoiceMaterial,
                ai_disclosure_required: true,
                watermark: WatermarkStatus::SidecarOnly,
                reference_media_ownership: Some(
                    "original prompt and lyrics drafted inside SoundWorks".to_string(),
                ),
            },
            provenance: provenance_record(
                "provenance-song-stem-export",
                "asset-song-city-lights-full",
                vec![
                    ("rights-reviewed", ProvenanceEventType::RightsReviewed, "Provider terms and originality disclosure reviewed before export."),
                    ("generated", ProvenanceEventType::Generated, "Song recipe, sections, lyrics, stems, and model ID attached."),
                    ("edited", ProvenanceEventType::Edited, "Review workspace normalization and trim chain attached."),
                    ("exported", ProvenanceEventType::Exported, "Stem folder export wrote rights and provenance sidecar."),
                ],
            ),
        },
    ]
}

fn provenance_record(
    id: &str,
    subject_id: &str,
    events: Vec<(&str, ProvenanceEventType, &str)>,
) -> ProvenanceRecord {
    ProvenanceRecord {
        id: id.to_string(),
        subject_id: subject_id.to_string(),
        events: events
            .into_iter()
            .map(|(id, event_type, summary)| {
                let mut metadata = BTreeMap::new();
                metadata.insert("id".to_string(), json!(id));
                metadata.insert("author".to_string(), json!("soundworks-policy"));

                ProvenanceEvent {
                    event_type,
                    actor: "system".to_string(),
                    summary: summary.to_string(),
                    metadata,
                }
            })
            .collect(),
    }
}

fn disclosure_checks() -> Vec<DisclosureCheck> {
    vec![
        DisclosureCheck {
            id: "disclosure.voice.generated".to_string(),
            asset_id: "asset-voice-lab-conversion-reference".to_string(),
            required: true,
            reason: "Voice conversion output is generated from a source clip and consented target profile."
                .to_string(),
            export_targets: vec![ExportTarget::AudioFile, ExportTarget::SceneWorksVideoTrack],
        },
        DisclosureCheck {
            id: "disclosure.song.generated".to_string(),
            asset_id: "asset-song-city-lights-full".to_string(),
            required: true,
            reason: "Full-song and stem exports need AI-generation disclosure and model provenance."
                .to_string(),
            export_targets: vec![ExportTarget::StemFolder, ExportTarget::DawHandoff],
        },
    ]
}

fn validation_checks() -> Vec<RightsValidationCheck> {
    vec![
        RightsValidationCheck {
            id: "validation.voice-consent".to_string(),
            status: RightsValidationStatus::Passed,
            summary: "Voice clone and conversion requests have allow/block decisions derived from consent metadata."
                .to_string(),
        },
        RightsValidationCheck {
            id: "validation.model-license".to_string(),
            status: RightsValidationStatus::Passed,
            summary: "SoundWorks export decisions include model license, product eligibility, and runtime dependency blockers."
                .to_string(),
        },
        RightsValidationCheck {
            id: "validation.provenance-sidecar".to_string(),
            status: RightsValidationStatus::Passed,
            summary: "Export sidecars include recipe, model, source media, rights, disclosure, and edit-chain fields."
                .to_string(),
        },
        RightsValidationCheck {
            id: "validation.watermark-policy".to_string(),
            status: RightsValidationStatus::Warning,
            summary: "Watermark embedding remains advisory until provider support is selected; sidecar disclosure is mandatory now."
                .to_string(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::{
        PolicyDecision, PolicyGateStatus, RightsSafetyOverview, RightsValidationStatus,
        RiskCategory, WatermarkPolicy,
    };
    use crate::domain::{VoiceConsentStatus, WatermarkStatus};
    use crate::evaluation::{CommercialUseEvaluation, REQUIRED_CANDIDATE_IDS};

    #[test]
    fn consent_checks_block_voice_workflows_without_explicit_metadata() {
        let overview = RightsSafetyOverview::reference();

        assert!(overview.consent_checks.iter().any(|check| {
            check.workflow == "voice-clone"
                && check.consent_status == VoiceConsentStatus::ExplicitConsentRecorded
                && check.decision == PolicyDecision::Allowed
        }));
        assert!(overview.consent_checks.iter().any(|check| {
            check.workflow == "voice-conversion"
                && check.consent_status == VoiceConsentStatus::RequiresReview
                && check.decision == PolicyDecision::Blocked
        }));
        assert!(overview.content_policy_gates.iter().any(|gate| {
            gate.category == RiskCategory::PublicFigureVoiceClone
                && gate.status == PolicyGateStatus::Blocked
        }));
    }

    #[test]
    fn export_blocks_unknown_research_only_or_python_runtime_models() {
        let overview = RightsSafetyOverview::reference();
        let chattts = overview
            .model_use_decisions
            .iter()
            .find(|decision| decision.candidate_id == "chattts")
            .expect("ChatTTS decision exists");
        let diffrhythm = overview
            .model_use_decisions
            .iter()
            .find(|decision| decision.candidate_id == "diffrhythm-2")
            .expect("DiffRhythm decision exists");

        assert_eq!(
            chattts.commercial_use,
            CommercialUseEvaluation::NonCommercial
        );
        assert_eq!(chattts.decision, PolicyDecision::Blocked);
        assert!(chattts
            .reasons
            .iter()
            .any(|reason| reason.contains("Only product-candidate models")));
        assert_eq!(diffrhythm.decision, PolicyDecision::Blocked);
        assert!(diffrhythm
            .reasons
            .iter()
            .any(|reason| reason.contains("Unknown model-use terms")));
        assert!(!overview.can_export());
    }

    #[test]
    fn model_use_decisions_cover_every_required_candidate() {
        let overview = RightsSafetyOverview::reference();

        for candidate_id in REQUIRED_CANDIDATE_IDS {
            assert!(
                overview
                    .model_use_decisions
                    .iter()
                    .any(|decision| decision.candidate_id == *candidate_id),
                "missing model-use decision for {candidate_id}"
            );
        }
        assert_eq!(
            overview.model_use_decisions.len(),
            REQUIRED_CANDIDATE_IDS.len()
        );
    }

    #[test]
    fn provenance_sidecars_include_required_export_audit_fields() {
        let overview = RightsSafetyOverview::reference();

        assert!(overview.export_sidecars.iter().all(|sidecar| {
            sidecar.includes_recipe
                && sidecar.includes_model
                && sidecar.includes_source_media
                && sidecar.includes_rights
                && sidecar.disclosure_required
                && sidecar.path.ends_with("/metadata/recipe-provenance.json")
        }));
        assert!(overview
            .export_sidecars
            .iter()
            .any(|sidecar| sidecar.watermark == WatermarkStatus::SidecarOnly));
    }

    #[test]
    fn validation_contract_covers_safety_decisions_and_watermark_policy() {
        let overview = RightsSafetyOverview::reference();

        assert_eq!(
            overview.policy.watermark_policy,
            WatermarkPolicy::AdvisoryUntilProviderSupport
        );
        assert!(overview
            .validation_checks
            .iter()
            .any(|check| check.status == RightsValidationStatus::Warning
                && check.summary.contains("Watermark")));
        assert!(overview
            .disclosure_checks
            .iter()
            .all(|check| check.required));
    }
}
