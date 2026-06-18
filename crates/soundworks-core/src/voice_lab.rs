use crate::domain::{
    AssetCreation, AudioAsset, AudioAssetKind, AudioAssetVersion, AudioFileFormat,
    AudioFileReference, CommercialUseStatus, GenerationJob, GenerationRecipe, JobKind, JobProgress,
    JobStatus, LibraryScope, LicenseStatus, ModelDescriptor, ModelRuntime, PostProcessingOperation,
    PostProcessingStep, RecipeRequest, RecipeWorkflow, RightsMetadata, SourceReference,
    SourceReferenceType, TechnicalAudioMetadata, VoiceConsentStatus, VoiceConversionRecipe,
    VoiceProfile, VoiceUse, WatermarkStatus,
};
use crate::evaluation::{
    CommercialUseEvaluation, EvaluationLane, EvaluationStatus, ModelEvaluationCandidate,
    ModelEvaluationCatalog, ProductEligibility, ProductRuntimePath,
};
use crate::manifests::CapabilityWorkflow;
use crate::storage::{StoragePathAllocator, StoragePathError, StoragePaths};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;

pub const VOICE_LAB_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceLabOverview {
    pub schema_version: u32,
    pub modes: Vec<VoiceLabModeCard>,
    pub voice_profiles: Vec<VoiceLabProfile>,
    pub reference_clips: Vec<VoiceReferenceClip>,
    pub conversion_source: VoiceConversionSource,
    pub provider_scorecards: Vec<VoiceProviderScorecard>,
    pub selected_conversion: VoiceConversionPreview,
    pub saved_output: VoiceLabSavedOutput,
    pub safety_gates: Vec<VoiceSafetyGate>,
    pub qa_checks: Vec<VoiceQaCheck>,
}

impl VoiceLabOverview {
    pub fn reference() -> Result<Self, StoragePathError> {
        Self::from_catalog(
            &ModelEvaluationCatalog::reference(),
            &StoragePathAllocator::new("soundworks-library"),
        )
    }

    pub fn from_catalog(
        catalog: &ModelEvaluationCatalog,
        allocator: &StoragePathAllocator,
    ) -> Result<Self, StoragePathError> {
        let voice_profiles = reference_profiles();
        let reference_clips = reference_clips();
        let conversion_source = VoiceConversionSource::reference();
        let provider_scorecards = provider_scorecards(catalog);
        let selected_conversion = VoiceConversionPreview::build(
            &conversion_source,
            &voice_profiles[0],
            &provider_scorecards,
        );
        let saved_output = saved_output(&selected_conversion, allocator)?;

        Ok(Self {
            schema_version: VOICE_LAB_SCHEMA_VERSION,
            modes: mode_cards(&provider_scorecards),
            voice_profiles,
            reference_clips,
            conversion_source,
            provider_scorecards,
            selected_conversion,
            saved_output,
            safety_gates: safety_gates(),
            qa_checks: qa_checks(),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VoiceLabMode {
    ZeroShotClone,
    FewShotFineTune,
    VoiceConversion,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceLabModeCard {
    pub mode: VoiceLabMode,
    pub label: String,
    pub workflow: CapabilityWorkflow,
    pub input_asset_kinds: Vec<AudioAssetKind>,
    pub output_asset_kind: AudioAssetKind,
    pub provider_candidate_ids: Vec<String>,
    pub ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceLabProfile {
    pub profile: VoiceProfile,
    pub speaker_identity: String,
    pub language: String,
    pub source_clip_ids: Vec<String>,
    pub mode_readiness: Vec<VoiceModeReadiness>,
    pub commercial_use_allowed: bool,
    pub safety_summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceModeReadiness {
    pub mode: VoiceLabMode,
    pub ready: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceReferenceClip {
    pub id: String,
    pub asset_id: String,
    pub profile_id: String,
    pub label: String,
    pub duration_ms: u64,
    pub consent: VoiceConsentStatus,
    pub owner_attestation: String,
    pub accepted_for_modes: Vec<VoiceLabMode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceConversionSource {
    pub asset_id: String,
    pub name: String,
    pub duration_ms: u64,
    pub kind: AudioAssetKind,
}

impl VoiceConversionSource {
    fn reference() -> Self {
        Self {
            asset_id: "asset-voice-lab-source-read".to_string(),
            name: "Producer dry read".to_string(),
            duration_ms: 7_800,
            kind: AudioAssetKind::ReferenceAudio,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceProviderScorecard {
    pub candidate_id: String,
    pub name: String,
    pub provider: String,
    pub lanes: Vec<EvaluationLane>,
    pub status: EvaluationStatus,
    pub product_eligibility: ProductEligibility,
    pub readiness: VoiceProviderReadiness,
    pub runtime_path: ProductRuntimePath,
    pub commercial_use: CommercialUseEvaluation,
    pub recommended: bool,
    pub blockers: Vec<String>,
    pub notes: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VoiceProviderReadiness {
    Ready,
    NeedsRuntimePort,
    ResearchOnly,
    Blocked,
    Unsuitable,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceConversionPreview {
    pub can_submit: bool,
    pub job: GenerationJob,
    pub recipe: GenerationRecipe,
    pub blocking_reasons: Vec<String>,
    pub warnings: Vec<String>,
}

impl VoiceConversionPreview {
    fn build(
        source: &VoiceConversionSource,
        target_profile: &VoiceLabProfile,
        scorecards: &[VoiceProviderScorecard],
    ) -> Self {
        let selected_provider = scorecards
            .iter()
            .find(|scorecard| scorecard.candidate_id == "rvc")
            .expect("RVC voice conversion scorecard exists");
        let mut blocking_reasons = vec![];
        let mut warnings = vec![];

        if source.asset_id.trim().is_empty() {
            blocking_reasons.push("Source audio is required for voice conversion.".to_string());
        }

        if target_profile.profile.consent != VoiceConsentStatus::ExplicitConsentRecorded {
            blocking_reasons.push(
                "Voice conversion requires explicit consent for the target voice profile."
                    .to_string(),
            );
        }

        if !target_profile
            .profile
            .allowed_uses
            .contains(&VoiceUse::VoiceConversion)
        {
            blocking_reasons
                .push("Target voice profile is not approved for voice conversion.".to_string());
        }

        if matches!(
            selected_provider.readiness,
            VoiceProviderReadiness::Blocked
                | VoiceProviderReadiness::ResearchOnly
                | VoiceProviderReadiness::Unsuitable
        ) {
            blocking_reasons.push(format!(
                "{} is not product-runnable for conversion.",
                selected_provider.name
            ));
        }

        if selected_provider.readiness == VoiceProviderReadiness::NeedsRuntimePort {
            warnings.push(
                "RVC is represented as a gated provider spike until a packaged runtime port exists."
                    .to_string(),
            );
        }

        warnings.push(
            "Converted output keeps the source timing and is saved as a Voice clip with recipe provenance."
                .to_string(),
        );

        let can_submit = blocking_reasons.is_empty();
        let recipe = conversion_recipe(source, target_profile, selected_provider);

        Self {
            can_submit,
            job: GenerationJob {
                id: "job-voice-lab-conversion-reference".to_string(),
                recipe_id: recipe.id.clone(),
                kind: JobKind::GenerateAudio,
                status: if can_submit {
                    JobStatus::Queued
                } else {
                    JobStatus::Failed
                },
                progress: Some(JobProgress {
                    percent: if can_submit { 0.0 } else { 100.0 },
                    message: Some(if can_submit {
                        "Ready to queue speech-to-speech voice conversion.".to_string()
                    } else {
                        "Voice conversion blocked by safety validation.".to_string()
                    }),
                }),
                output_version_ids: if can_submit {
                    vec!["version-voice-lab-conversion-reference-a".to_string()]
                } else {
                    vec![]
                },
                error: if can_submit {
                    None
                } else {
                    Some(blocking_reasons.join(" "))
                },
            },
            recipe,
            blocking_reasons,
            warnings,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceLabSavedOutput {
    pub asset: AudioAsset,
    pub version: AudioAssetVersion,
    pub storage: StoragePaths,
    pub waveform_preview_ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceSafetyGate {
    pub id: String,
    pub status: VoiceGateStatus,
    pub summary: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VoiceGateStatus {
    Passed,
    Warning,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceQaCheck {
    pub id: String,
    pub label: String,
    pub status: VoiceQaStatus,
    pub target: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VoiceQaStatus {
    Ready,
    NeedsReview,
}

fn mode_cards(scorecards: &[VoiceProviderScorecard]) -> Vec<VoiceLabModeCard> {
    vec![
        VoiceLabModeCard {
            mode: VoiceLabMode::ZeroShotClone,
            label: "Zero-shot clone".to_string(),
            workflow: CapabilityWorkflow::VoiceClone,
            input_asset_kinds: vec![AudioAssetKind::ReferenceAudio],
            output_asset_kind: AudioAssetKind::VoiceClip,
            provider_candidate_ids: candidate_ids_for(scorecards, EvaluationLane::VoiceClone),
            ready: true,
        },
        VoiceLabModeCard {
            mode: VoiceLabMode::FewShotFineTune,
            label: "Few-shot fine-tune".to_string(),
            workflow: CapabilityWorkflow::VoiceClone,
            input_asset_kinds: vec![AudioAssetKind::ReferenceAudio],
            output_asset_kind: AudioAssetKind::VoiceClip,
            provider_candidate_ids: vec![
                "gpt-sovits".to_string(),
                "f5-tts".to_string(),
                "cosyvoice-2".to_string(),
            ],
            ready: true,
        },
        VoiceLabModeCard {
            mode: VoiceLabMode::VoiceConversion,
            label: "Voice conversion".to_string(),
            workflow: CapabilityWorkflow::VoiceConversion,
            input_asset_kinds: vec![AudioAssetKind::ReferenceAudio, AudioAssetKind::VoiceClip],
            output_asset_kind: AudioAssetKind::VoiceClip,
            provider_candidate_ids: candidate_ids_for(scorecards, EvaluationLane::VoiceConversion),
            ready: true,
        },
    ]
}

fn candidate_ids_for(scorecards: &[VoiceProviderScorecard], lane: EvaluationLane) -> Vec<String> {
    scorecards
        .iter()
        .filter(|scorecard| scorecard.lanes.contains(&lane))
        .map(|scorecard| scorecard.candidate_id.clone())
        .collect()
}

fn reference_profiles() -> Vec<VoiceLabProfile> {
    vec![
        VoiceLabProfile {
            profile: VoiceProfile {
                id: "voice-profile-narrator".to_string(),
                display_name: "Narrator consented profile".to_string(),
                source_reference_ids: vec![
                    "voice-ref-narrator-close".to_string(),
                    "voice-ref-narrator-energetic".to_string(),
                ],
                consent: VoiceConsentStatus::ExplicitConsentRecorded,
                allowed_uses: vec![
                    VoiceUse::Tts,
                    VoiceUse::VoiceConversion,
                    VoiceUse::FineTuning,
                    VoiceUse::ProjectOnly,
                    VoiceUse::Commercial,
                ],
                provenance_ids: vec!["provenance-voice-narrator".to_string()],
            },
            speaker_identity: "Narrator".to_string(),
            language: "en-US".to_string(),
            source_clip_ids: vec![
                "voice-ref-narrator-close".to_string(),
                "voice-ref-narrator-energetic".to_string(),
            ],
            mode_readiness: vec![
                VoiceModeReadiness {
                    mode: VoiceLabMode::ZeroShotClone,
                    ready: true,
                    reason: None,
                },
                VoiceModeReadiness {
                    mode: VoiceLabMode::FewShotFineTune,
                    ready: true,
                    reason: None,
                },
                VoiceModeReadiness {
                    mode: VoiceLabMode::VoiceConversion,
                    ready: true,
                    reason: None,
                },
            ],
            commercial_use_allowed: true,
            safety_summary:
                "Explicit consent covers TTS, fine-tuning, and conversion for this project."
                    .to_string(),
        },
        VoiceLabProfile {
            profile: VoiceProfile {
                id: "voice-profile-archival".to_string(),
                display_name: "Archival interview review".to_string(),
                source_reference_ids: vec!["voice-ref-archival-interview".to_string()],
                consent: VoiceConsentStatus::RequiresReview,
                allowed_uses: vec![VoiceUse::ProjectOnly],
                provenance_ids: vec!["provenance-voice-archival".to_string()],
            },
            speaker_identity: "Interview guest".to_string(),
            language: "en-US".to_string(),
            source_clip_ids: vec!["voice-ref-archival-interview".to_string()],
            mode_readiness: vec![
                VoiceModeReadiness {
                    mode: VoiceLabMode::ZeroShotClone,
                    ready: false,
                    reason: Some("Explicit voice consent has not been recorded.".to_string()),
                },
                VoiceModeReadiness {
                    mode: VoiceLabMode::FewShotFineTune,
                    ready: false,
                    reason: Some(
                        "Fine-tuning requires explicit consent and ownership notes.".to_string(),
                    ),
                },
                VoiceModeReadiness {
                    mode: VoiceLabMode::VoiceConversion,
                    ready: false,
                    reason: Some("Conversion is disabled until consent review passes.".to_string()),
                },
            ],
            commercial_use_allowed: false,
            safety_summary:
                "Kept visible for review, but all cloning and conversion actions are gated."
                    .to_string(),
        },
    ]
}

fn reference_clips() -> Vec<VoiceReferenceClip> {
    vec![
        VoiceReferenceClip {
            id: "voice-ref-narrator-close".to_string(),
            asset_id: "asset-narrator-close-ref".to_string(),
            profile_id: "voice-profile-narrator".to_string(),
            label: "Close mic neutral read".to_string(),
            duration_ms: 18_200,
            consent: VoiceConsentStatus::ExplicitConsentRecorded,
            owner_attestation: "speaker-signed".to_string(),
            accepted_for_modes: vec![
                VoiceLabMode::ZeroShotClone,
                VoiceLabMode::FewShotFineTune,
                VoiceLabMode::VoiceConversion,
            ],
        },
        VoiceReferenceClip {
            id: "voice-ref-narrator-energetic".to_string(),
            asset_id: "asset-narrator-energy-ref".to_string(),
            profile_id: "voice-profile-narrator".to_string(),
            label: "Energetic promo read".to_string(),
            duration_ms: 24_600,
            consent: VoiceConsentStatus::ExplicitConsentRecorded,
            owner_attestation: "speaker-signed".to_string(),
            accepted_for_modes: vec![VoiceLabMode::FewShotFineTune],
        },
        VoiceReferenceClip {
            id: "voice-ref-archival-interview".to_string(),
            asset_id: "asset-archival-interview-ref".to_string(),
            profile_id: "voice-profile-archival".to_string(),
            label: "Interview excerpt".to_string(),
            duration_ms: 31_400,
            consent: VoiceConsentStatus::RequiresReview,
            owner_attestation: "review-required".to_string(),
            accepted_for_modes: vec![],
        },
    ]
}

fn provider_scorecards(catalog: &ModelEvaluationCatalog) -> Vec<VoiceProviderScorecard> {
    let recommended_ids = catalog
        .recommendations
        .iter()
        .map(|recommendation| recommendation.candidate_id.as_str())
        .collect::<Vec<_>>();

    [
        "chatterbox",
        "chatterbox-turbo",
        "gpt-sovits",
        "f5-tts",
        "cosyvoice-2",
        "openvoice-v2",
        "rvc",
        "xtts-v2",
    ]
    .into_iter()
    .filter_map(|candidate_id| {
        catalog
            .candidates
            .iter()
            .find(|candidate| candidate.id == candidate_id)
            .map(|candidate| scorecard(candidate, recommended_ids.contains(&candidate.id.as_str())))
    })
    .collect()
}

fn scorecard(candidate: &ModelEvaluationCandidate, recommended: bool) -> VoiceProviderScorecard {
    let supports_voice_lab = candidate.lanes.iter().any(|lane| {
        matches!(
            lane,
            EvaluationLane::VoiceClone | EvaluationLane::VoiceConversion
        )
    });
    let readiness = if !supports_voice_lab {
        VoiceProviderReadiness::Unsuitable
    } else {
        match candidate.product_eligibility {
            ProductEligibility::ProductCandidate | ProductEligibility::ApiOnlyCandidate => {
                VoiceProviderReadiness::Ready
            }
            ProductEligibility::NeedsRuntimePort => VoiceProviderReadiness::NeedsRuntimePort,
            ProductEligibility::ResearchOnly => VoiceProviderReadiness::ResearchOnly,
            ProductEligibility::Blocked => VoiceProviderReadiness::Blocked,
        }
    };

    let mut blockers = candidate.blockers.clone();
    if !supports_voice_lab {
        blockers
            .push("Candidate is tracked for TTS only and is not a Voice Lab provider.".to_string());
    }

    VoiceProviderScorecard {
        candidate_id: candidate.id.clone(),
        name: candidate.name.clone(),
        provider: candidate.provider.clone(),
        lanes: candidate.lanes.clone(),
        status: candidate.status,
        product_eligibility: candidate.product_eligibility,
        readiness,
        runtime_path: candidate.runtime.product_path,
        commercial_use: candidate.license.commercial_use,
        recommended,
        blockers,
        notes: candidate.notes.clone(),
    }
}

fn conversion_recipe(
    source: &VoiceConversionSource,
    target_profile: &VoiceLabProfile,
    provider: &VoiceProviderScorecard,
) -> GenerationRecipe {
    let mut parameters = BTreeMap::new();
    parameters.insert("preserveTiming".to_string(), json!(true));
    parameters.insert("mode".to_string(), json!("speech-to-speech"));

    GenerationRecipe {
        id: "recipe-voice-lab-conversion-reference".to_string(),
        workflow: RecipeWorkflow::VoiceConversion,
        provider: ModelDescriptor {
            provider_id: provider.candidate_id.clone(),
            model_id: provider.candidate_id.clone(),
            model_version: Some("evaluation-candidate".to_string()),
            model_hash: None,
            runtime: runtime_for(provider.runtime_path),
        },
        request: RecipeRequest::VoiceConversion(VoiceConversionRecipe {
            source_audio_asset_id: source.asset_id.clone(),
            target_voice_profile_id: target_profile.profile.id.clone(),
            preserve_timing: true,
        }),
        seed: None,
        source_references: vec![
            SourceReference {
                id: "source-voice-lab-conversion-audio".to_string(),
                source_type: SourceReferenceType::Audio,
                asset_id: Some(source.asset_id.clone()),
                external_uri: None,
                ownership_note: Some(
                    "User-owned source read for speech-to-speech conversion.".to_string(),
                ),
            },
            SourceReference {
                id: "source-voice-lab-target-profile".to_string(),
                source_type: SourceReferenceType::Voice,
                asset_id: None,
                external_uri: None,
                ownership_note: Some(
                    "Explicit consent recorded for target voice profile.".to_string(),
                ),
            },
        ],
        post_processing: vec![PostProcessingStep {
            id: "post-normalize-converted-voice".to_string(),
            operation: PostProcessingOperation::Normalize,
            parameters,
        }],
        parameter_overrides: BTreeMap::new(),
        output_asset_ids: vec!["asset-voice-lab-conversion-reference".to_string()],
    }
}

fn runtime_for(path: ProductRuntimePath) -> ModelRuntime {
    match path {
        ProductRuntimePath::ManagedApi => ModelRuntime::ExternalApi,
        ProductRuntimePath::PythonPocOnly | ProductRuntimePath::Unknown => {
            ModelRuntime::ResearchOnly
        }
        ProductRuntimePath::RustNative
        | ProductRuntimePath::NativeLibraryBinding
        | ProductRuntimePath::ExternalExecutable => ModelRuntime::Local,
    }
}

fn saved_output(
    preview: &VoiceConversionPreview,
    allocator: &StoragePathAllocator,
) -> Result<VoiceLabSavedOutput, StoragePathError> {
    let scope = LibraryScope::Project {
        project_id: "project-demo".to_string(),
    };
    let storage = allocator.allocate_asset_version(
        &scope,
        AudioAssetKind::VoiceClip,
        "asset-voice-lab-conversion-reference",
        "version-voice-lab-conversion-reference-a",
        AudioFileFormat::Wav,
    )?;

    let version = AudioAssetVersion {
        id: "version-voice-lab-conversion-reference-a".to_string(),
        asset_id: "asset-voice-lab-conversion-reference".to_string(),
        version_index: 1,
        file: AudioFileReference {
            storage_path: storage.media_path.clone(),
            format: AudioFileFormat::Wav,
            codec: Some("pcm_s16le".to_string()),
            byte_size: None,
            content_hash: None,
        },
        technical: TechnicalAudioMetadata {
            sample_rate_hz: 48_000,
            bit_depth: Some(24),
            channels: 1,
            duration_ms: 7_800,
            loudness_lufs: Some(-18.0),
            true_peak_dbfs: Some(-2.0),
            has_clipping: false,
            bpm: None,
            musical_key: None,
            loop_points: None,
        },
        created_by: AssetCreation::Generated {
            recipe_id: preview.recipe.id.clone(),
            job_id: preview.job.id.clone(),
        },
        waveform_preview_cache: Some(storage.waveform_preview_path.clone()),
        spectrogram_preview_cache: Some(storage.spectrogram_preview_path.clone()),
    };

    let asset = AudioAsset {
        id: "asset-voice-lab-conversion-reference".to_string(),
        scope,
        kind: AudioAssetKind::VoiceClip,
        name: "Narrator converted read".to_string(),
        tags: vec![
            "voice-clips".to_string(),
            "voice-conversion".to_string(),
            "speech-to-speech".to_string(),
        ],
        collection_ids: vec!["collection-project-voice-lab".to_string()],
        current_version_id: version.id.clone(),
        version_ids: vec![version.id.clone()],
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
        provenance_ids: vec!["provenance-voice-lab-conversion-reference".to_string()],
    };

    Ok(VoiceLabSavedOutput {
        asset,
        version,
        storage,
        waveform_preview_ready: true,
    })
}

fn safety_gates() -> Vec<VoiceSafetyGate> {
    vec![
        VoiceSafetyGate {
            id: "voice.consent.explicit".to_string(),
            status: VoiceGateStatus::Passed,
            summary: "Clone, fine-tune, and conversion modes require explicit voice consent.".to_string(),
        },
        VoiceSafetyGate {
            id: "voice.unauthorized_clone.blocked".to_string(),
            status: VoiceGateStatus::Passed,
            summary: "Profiles marked Requires review cannot queue cloning or conversion jobs.".to_string(),
        },
        VoiceSafetyGate {
            id: "voice.conversion.source_audio".to_string(),
            status: VoiceGateStatus::Passed,
            summary: "RVC-style conversion requires source audio and a target voice profile.".to_string(),
        },
        VoiceSafetyGate {
            id: "voice.commercial_use.review".to_string(),
            status: VoiceGateStatus::Warning,
            summary: "Unknown provider licenses stay visible as blocked scorecards; noncommercial licenses require SoundWorks compatibility review.".to_string(),
        },
    ]
}

fn qa_checks() -> Vec<VoiceQaCheck> {
    vec![
        VoiceQaCheck {
            id: "qa.similarity".to_string(),
            label: "Speaker similarity".to_string(),
            status: VoiceQaStatus::Ready,
            target: "Compare converted output against the consented target profile.".to_string(),
        },
        VoiceQaCheck {
            id: "qa.intelligibility".to_string(),
            label: "Intelligibility".to_string(),
            status: VoiceQaStatus::Ready,
            target: "Confirm speech remains clear after conversion.".to_string(),
        },
        VoiceQaCheck {
            id: "qa.artifacts".to_string(),
            label: "Artifacts".to_string(),
            status: VoiceQaStatus::NeedsReview,
            target: "Review pitch tracking, breaths, and metallic artifacts before export."
                .to_string(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::{
        VoiceGateStatus, VoiceLabMode, VoiceLabOverview, VoiceProviderReadiness, VoiceQaStatus,
    };
    use crate::domain::{AudioAssetKind, RecipeRequest, RecipeWorkflow, VoiceConsentStatus};
    use crate::evaluation::EvaluationLane;

    #[test]
    fn reference_overview_represents_distinct_voice_lab_modes() {
        let overview = VoiceLabOverview::reference().expect("voice lab builds");

        let modes = overview
            .modes
            .iter()
            .map(|mode| mode.mode)
            .collect::<Vec<_>>();

        assert_eq!(
            modes,
            vec![
                VoiceLabMode::ZeroShotClone,
                VoiceLabMode::FewShotFineTune,
                VoiceLabMode::VoiceConversion
            ]
        );
        assert!(overview
            .voice_profiles
            .iter()
            .any(|profile| profile.profile.consent == VoiceConsentStatus::ExplicitConsentRecorded));
        assert!(overview
            .voice_profiles
            .iter()
            .any(|profile| !profile.commercial_use_allowed));
    }

    #[test]
    fn provider_scorecards_keep_rvc_as_conversion_not_tts() {
        let overview = VoiceLabOverview::reference().expect("voice lab builds");
        let rvc = overview
            .provider_scorecards
            .iter()
            .find(|scorecard| scorecard.candidate_id == "rvc")
            .expect("RVC scorecard exists");
        let turbo = overview
            .provider_scorecards
            .iter()
            .find(|scorecard| scorecard.candidate_id == "chatterbox-turbo")
            .expect("Chatterbox Turbo scorecard exists");

        assert_eq!(rvc.lanes, vec![EvaluationLane::VoiceConversion]);
        assert!(!rvc.lanes.contains(&EvaluationLane::Tts));
        assert_eq!(rvc.readiness, VoiceProviderReadiness::NeedsRuntimePort);
        assert_eq!(turbo.readiness, VoiceProviderReadiness::Unsuitable);
    }

    #[test]
    fn conversion_preview_uses_source_audio_and_voice_conversion_recipe() {
        let overview = VoiceLabOverview::reference().expect("voice lab builds");

        assert!(overview.selected_conversion.can_submit);
        assert!(overview.selected_conversion.blocking_reasons.is_empty());
        assert_eq!(
            overview.selected_conversion.recipe.workflow,
            RecipeWorkflow::VoiceConversion
        );

        let RecipeRequest::VoiceConversion(recipe) = &overview.selected_conversion.recipe.request
        else {
            panic!("expected voice conversion request");
        };

        assert_eq!(
            recipe.source_audio_asset_id,
            overview.conversion_source.asset_id
        );
        assert!(recipe.preserve_timing);
    }

    #[test]
    fn safety_gates_and_saved_output_capture_export_contract() {
        let overview = VoiceLabOverview::reference().expect("voice lab builds");

        assert!(overview
            .safety_gates
            .iter()
            .any(|gate| gate.id == "voice.unauthorized_clone.blocked"
                && gate.status == VoiceGateStatus::Passed));
        assert!(overview
            .qa_checks
            .iter()
            .any(|check| check.status == VoiceQaStatus::NeedsReview));
        assert_eq!(overview.saved_output.asset.kind, AudioAssetKind::VoiceClip);
        assert!(overview
            .saved_output
            .version
            .file
            .storage_path
            .contains("/voice-clips/asset-voice-lab-conversion-reference/"));
    }
}
