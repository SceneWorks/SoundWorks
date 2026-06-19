use crate::domain::{
    AssetCreation, AudioAsset, AudioAssetKind, AudioAssetVersion, AudioFileFormat,
    AudioFileReference, CommercialUseStatus, GenerationJob, GenerationRecipe, JobKind, JobProgress,
    JobStatus, LibraryScope, LicenseStatus, ModelDescriptor, ModelRuntime, PostProcessingOperation,
    PostProcessingStep, RecipeRequest, RecipeWorkflow, RightsMetadata, SourceReference,
    SourceReferenceType, TechnicalAudioMetadata, TtsRecipe, VoiceConsentStatus, VoiceProfile,
    VoiceUse, WatermarkStatus,
};
use crate::manifests::{
    CapabilityInput, CapabilitySafety, CapabilityWorkflow, ChannelLayout, ModelLicense,
    ProviderCatalog,
};
use crate::runtime::RuntimeOverview;
use crate::storage::{StoragePathAllocator, StoragePathError, StoragePaths};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;

pub const TTS_STUDIO_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TtsStudioOverview {
    pub schema_version: u32,
    pub script: TtsScript,
    pub speakers: Vec<TtsSpeaker>,
    pub voice_profiles: Vec<VoiceProfile>,
    pub provider_options: Vec<TtsProviderOption>,
    pub selected_provider: TtsProviderSelection,
    pub controls: TtsControls,
    pub generation_plan: TtsGenerationPlan,
    pub submission: TtsSubmissionPreview,
    pub saved_output: TtsSavedOutput,
    pub validation_checks: Vec<TtsValidationCheck>,
}

impl TtsStudioOverview {
    pub fn reference() -> Result<Self, StoragePathError> {
        Self::from_catalog(
            &ProviderCatalog::reference(),
            &RuntimeOverview::reference(),
            &StoragePathAllocator::new("soundworks-library"),
        )
    }

    pub fn from_catalog(
        catalog: &ProviderCatalog,
        runtime: &RuntimeOverview,
        allocator: &StoragePathAllocator,
    ) -> Result<Self, StoragePathError> {
        let script = reference_script();
        let speakers = reference_speakers();
        let provider_options = provider_options(catalog, runtime);
        let selected_provider = provider_options
            .iter()
            .find(|option| option.runnable)
            .map(TtsProviderSelection::from_option)
            .unwrap_or_else(|| TtsProviderSelection {
                provider_id: "unavailable".to_string(),
                model_id: "unavailable".to_string(),
                model_version: None,
                runtime: ModelRuntime::ResearchOnly,
                accepted: false,
                blocker: Some("No runnable TTS provider is registered.".to_string()),
            });
        let controls = TtsControls::reference();
        let generation_plan = TtsGenerationPlan::from_script(&script, &speakers, &controls);
        let submission = TtsSubmissionPreview::build(
            &script,
            &speakers,
            &provider_options,
            &selected_provider,
            &controls,
            &generation_plan,
        );
        let saved_output = saved_output(&script, &selected_provider, allocator)?;

        Ok(Self {
            schema_version: TTS_STUDIO_SCHEMA_VERSION,
            script,
            speakers,
            voice_profiles: reference_voice_profiles(),
            provider_options,
            selected_provider,
            controls,
            generation_plan,
            submission,
            saved_output,
            validation_checks: validation_checks(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TtsScript {
    pub id: String,
    pub title: String,
    pub language: String,
    pub segments: Vec<TtsScriptSegment>,
    pub pronunciation_dictionary: Vec<PronunciationEntry>,
}

impl TtsScript {
    pub fn plain_text(&self) -> String {
        self.segments
            .iter()
            .map(|segment| segment.text.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn speaker_labels(&self) -> Vec<String> {
        let mut labels = vec![];

        for segment in &self.segments {
            if !labels.contains(&segment.speaker_label) {
                labels.push(segment.speaker_label.clone());
            }
        }

        labels
    }

    pub fn estimated_duration_ms(&self) -> u64 {
        self.segments
            .iter()
            .map(|segment| {
                segment
                    .target_duration_ms
                    .unwrap_or(estimate_segment_ms(&segment.text))
            })
            .sum()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TtsScriptSegment {
    pub id: String,
    pub position: u16,
    pub speaker_label: String,
    pub text: String,
    pub scene_label: Option<String>,
    pub target_duration_ms: Option<u64>,
    pub regenerate_policy: SegmentRegeneratePolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SegmentRegeneratePolicy {
    RegenerateIndependently,
    KeepTimingWithNeighbors,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PronunciationEntry {
    pub term: String,
    pub pronunciation: String,
    pub applies_to_language: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TtsSpeaker {
    pub label: String,
    pub role: String,
    pub voice_profile_id: String,
    pub language: String,
    pub consent_required: bool,
    pub consent_status: VoiceConsentStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TtsProviderOption {
    pub provider_id: String,
    pub model_id: String,
    pub model_version: Option<String>,
    pub display_name: String,
    pub runtime: ModelRuntime,
    pub install_status: String,
    pub runnable: bool,
    pub output_format: AudioFileFormat,
    pub sample_rate_hz: u32,
    pub channel_layout: ChannelLayout,
    pub supported_languages: Vec<String>,
    pub max_speakers: Option<u16>,
    pub max_duration_ms: Option<u64>,
    pub commercial_use_allowed: bool,
    pub requires_voice_consent: bool,
    pub watermark: WatermarkStatus,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TtsProviderSelection {
    pub provider_id: String,
    pub model_id: String,
    pub model_version: Option<String>,
    pub runtime: ModelRuntime,
    pub accepted: bool,
    pub blocker: Option<String>,
}

impl TtsProviderSelection {
    fn from_option(option: &TtsProviderOption) -> Self {
        Self {
            provider_id: option.provider_id.clone(),
            model_id: option.model_id.clone(),
            model_version: option.model_version.clone(),
            runtime: option.runtime,
            accepted: option.runnable,
            blocker: if option.runnable {
                None
            } else {
                option.limitations.first().cloned()
            },
        }
    }

    fn descriptor(&self) -> ModelDescriptor {
        ModelDescriptor {
            provider_id: self.provider_id.clone(),
            model_id: self.model_id.clone(),
            model_version: self.model_version.clone(),
            model_hash: None,
            runtime: self.runtime,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TtsControls {
    pub speed: f32,
    pub style: String,
    pub emotion: Option<String>,
    pub target_loudness_lufs: f32,
    pub normalize_output: bool,
    pub preserve_segment_timing: bool,
    pub promote_to_project_library: bool,
}

impl TtsControls {
    fn reference() -> Self {
        Self {
            speed: 1.0,
            style: "clear narration".to_string(),
            emotion: Some("warm".to_string()),
            target_loudness_lufs: -18.0,
            normalize_output: true,
            preserve_segment_timing: true,
            promote_to_project_library: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TtsGenerationPlan {
    pub chunks: Vec<TtsGenerationChunk>,
    pub stitching: TtsStitchingPlan,
    pub estimated_total_duration_ms: u64,
    pub preserves_speaker_consistency: bool,
}

impl TtsGenerationPlan {
    fn from_script(script: &TtsScript, speakers: &[TtsSpeaker], controls: &TtsControls) -> Self {
        let chunks = script
            .segments
            .iter()
            .map(|segment| TtsGenerationChunk {
                id: format!("chunk-{}", segment.id),
                segment_ids: vec![segment.id.clone()],
                speaker_label: segment.speaker_label.clone(),
                voice_profile_id: speakers
                    .iter()
                    .find(|speaker| speaker.label == segment.speaker_label)
                    .map(|speaker| speaker.voice_profile_id.clone()),
                target_duration_ms: segment
                    .target_duration_ms
                    .unwrap_or_else(|| estimate_segment_ms(&segment.text)),
                regenerate_policy: segment.regenerate_policy,
            })
            .collect::<Vec<_>>();

        Self {
            estimated_total_duration_ms: chunks.iter().map(|chunk| chunk.target_duration_ms).sum(),
            chunks,
            stitching: TtsStitchingPlan {
                crossfade_ms: 35,
                preserve_segment_timing: controls.preserve_segment_timing,
                silence_trim: true,
                normalize_loudness_lufs: Some(controls.target_loudness_lufs),
            },
            preserves_speaker_consistency: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TtsGenerationChunk {
    pub id: String,
    pub segment_ids: Vec<String>,
    pub speaker_label: String,
    pub voice_profile_id: Option<String>,
    pub target_duration_ms: u64,
    pub regenerate_policy: SegmentRegeneratePolicy,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TtsStitchingPlan {
    pub crossfade_ms: u16,
    pub preserve_segment_timing: bool,
    pub silence_trim: bool,
    pub normalize_loudness_lufs: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TtsSubmissionPreview {
    pub can_submit: bool,
    pub job: GenerationJob,
    pub recipe: GenerationRecipe,
    pub blocking_reasons: Vec<String>,
    pub warnings: Vec<String>,
}

impl TtsSubmissionPreview {
    fn build(
        script: &TtsScript,
        speakers: &[TtsSpeaker],
        provider_options: &[TtsProviderOption],
        selected_provider: &TtsProviderSelection,
        controls: &TtsControls,
        generation_plan: &TtsGenerationPlan,
    ) -> Self {
        let mut blocking_reasons = vec![];
        let mut warnings = vec![];
        let selected_option = provider_options.iter().find(|option| {
            option.provider_id == selected_provider.provider_id
                && option.model_id == selected_provider.model_id
        });

        if script.plain_text().trim().is_empty() {
            blocking_reasons.push("Script is empty.".to_string());
        }

        if selected_option.is_none() || !selected_provider.accepted {
            blocking_reasons.push(selected_provider.blocker.clone().unwrap_or_else(|| {
                "Selected provider cannot currently accept TTS jobs.".to_string()
            }));
        }

        if let Some(option) = selected_option {
            if option.requires_voice_consent
                && speakers.iter().any(|speaker| {
                    speaker.consent_status != VoiceConsentStatus::ExplicitConsentRecorded
                })
            {
                blocking_reasons.push(
                    "Voice-clone capable TTS cannot run until every selected voice profile has explicit consent."
                        .to_string(),
                );
            }

            if let Some(max_speakers) = option.max_speakers {
                if speakers.len() > usize::from(max_speakers) {
                    blocking_reasons.push(format!(
                        "{} supports at most {max_speakers} speaker(s) for one request.",
                        option.display_name
                    ));
                }
            }

            if let Some(max_duration_ms) = option.max_duration_ms {
                if generation_plan.estimated_total_duration_ms > max_duration_ms {
                    blocking_reasons.push(format!(
                        "{} supports up to {} ms; current script estimates {} ms.",
                        option.display_name,
                        max_duration_ms,
                        generation_plan.estimated_total_duration_ms
                    ));
                }
            }

            if !option.commercial_use_allowed {
                warnings
                    .push("Selected provider requires license review before export.".to_string());
            }
        }

        if controls.normalize_output {
            warnings.push(
                "Post-processing will normalize dialogue loudness after stitching.".to_string(),
            );
        }

        let can_submit = blocking_reasons.is_empty();
        let recipe = generation_recipe(script, speakers, selected_provider, controls);

        Self {
            can_submit,
            job: GenerationJob {
                id: "job-tts-studio-reference".to_string(),
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
                        "Ready to queue TTS generation.".to_string()
                    } else {
                        "Submission blocked by validation.".to_string()
                    }),
                }),
                output_version_ids: if can_submit {
                    vec!["version-tts-studio-reference-a".to_string()]
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
pub struct TtsSavedOutput {
    pub asset: AudioAsset,
    pub version: AudioAssetVersion,
    pub storage: StoragePaths,
    pub promoted_to_project_library: bool,
    pub waveform_preview_ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TtsValidationCheck {
    pub id: String,
    pub status: TtsValidationStatus,
    pub summary: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TtsValidationStatus {
    Passed,
    Warning,
    Failed,
}

fn provider_options(
    catalog: &ProviderCatalog,
    runtime: &RuntimeOverview,
) -> Vec<TtsProviderOption> {
    let mut options = vec![];

    for provider in &catalog.providers {
        for model in &provider.models {
            let Some(capability) = model.capabilities.iter().find(|capability| {
                capability.workflow == CapabilityWorkflow::Tts
                    && capability.inputs.contains(&CapabilityInput::Script)
                    && capability
                        .outputs
                        .asset_kinds
                        .contains(&AudioAssetKind::VoiceClip)
            }) else {
                continue;
            };
            let state = runtime
                .model_states
                .iter()
                .find(|state| state.provider_id == provider.id && state.model_id == model.id);
            let mut limitations = vec![];

            if !model.install.is_runnable_for_studio() {
                limitations
                    .push("Model is not installed, packaged, or externally managed.".to_string());
            }

            if let Some(state) = state {
                limitations.extend(state.reasons.clone());
            } else {
                limitations.push("Runtime has not reported this provider/model pair.".to_string());
            }

            limitations.extend(limitations_for_license(model.requirements.license));
            limitations.extend(limitations_for_safety(&capability.safety));

            options.push(TtsProviderOption {
                provider_id: provider.id.clone(),
                model_id: model.id.clone(),
                model_version: model.version.clone(),
                display_name: format!("{} / {}", provider.name, model.name),
                runtime: model.runtime,
                install_status: format!("{:?}", model.install.status).to_case_label(),
                runnable: state.map_or(false, |state| {
                    matches!(
                        state.availability,
                        crate::runtime::RuntimeAvailability::Installed
                            | crate::runtime::RuntimeAvailability::Available
                    )
                }),
                output_format: capability.defaults.format,
                sample_rate_hz: capability.defaults.sample_rate_hz,
                channel_layout: capability.defaults.channel_layout,
                supported_languages: capability.limits.supported_languages.clone(),
                max_speakers: capability.limits.max_speakers,
                max_duration_ms: capability.limits.max_duration_ms,
                commercial_use_allowed: capability.safety.commercial_use_allowed,
                requires_voice_consent: capability.safety.requires_voice_consent,
                watermark: capability.safety.watermark,
                limitations,
            });
        }
    }

    options
}

fn limitations_for_license(license: ModelLicense) -> Vec<String> {
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

fn limitations_for_safety(safety: &CapabilitySafety) -> Vec<String> {
    let mut limitations = vec![];

    if safety.requires_voice_consent {
        limitations.push("Voice profile consent is required before generation.".to_string());
    }

    if !safety.disallowed_uses.is_empty() {
        limitations.push(format!(
            "Disallowed uses: {}.",
            safety.disallowed_uses.join(", ")
        ));
    }

    limitations
}

fn generation_recipe(
    script: &TtsScript,
    speakers: &[TtsSpeaker],
    provider: &TtsProviderSelection,
    controls: &TtsControls,
) -> GenerationRecipe {
    let mut parameters = BTreeMap::new();
    parameters.insert("speed".to_string(), json!(controls.speed));
    parameters.insert("style".to_string(), json!(controls.style));
    parameters.insert("emotion".to_string(), json!(controls.emotion));
    parameters.insert(
        "preserveSegmentTiming".to_string(),
        json!(controls.preserve_segment_timing),
    );

    GenerationRecipe {
        id: "recipe-tts-studio-reference".to_string(),
        workflow: RecipeWorkflow::Tts,
        provider: provider.descriptor(),
        request: RecipeRequest::Tts(TtsRecipe {
            script: script.plain_text(),
            language: Some(script.language.clone()),
            speaker_labels: script.speaker_labels(),
            voice_profile_id: speakers
                .first()
                .map(|speaker| speaker.voice_profile_id.clone()),
            pronunciation_notes: script
                .pronunciation_dictionary
                .iter()
                .map(|entry| format!("{}={}", entry.term, entry.pronunciation))
                .collect(),
            target_duration_ms: Some(script.estimated_duration_ms()),
        }),
        seed: None,
        source_references: speakers
            .iter()
            .map(|speaker| SourceReference {
                id: format!("source-{}", speaker.voice_profile_id),
                source_type: SourceReferenceType::Voice,
                asset_id: None,
                external_uri: None,
                ownership_note: Some(format!(
                    "{} consent status: {:?}",
                    speaker.label, speaker.consent_status
                )),
            })
            .collect(),
        post_processing: vec![PostProcessingStep {
            id: "post-normalize-dialogue".to_string(),
            operation: PostProcessingOperation::Normalize,
            parameters,
        }],
        parameter_overrides: BTreeMap::new(),
        output_asset_ids: vec!["asset-tts-studio-reference".to_string()],
    }
}

fn saved_output(
    script: &TtsScript,
    provider: &TtsProviderSelection,
    allocator: &StoragePathAllocator,
) -> Result<TtsSavedOutput, StoragePathError> {
    let scope = LibraryScope::Project {
        project_id: "project-demo".to_string(),
    };
    let storage = allocator.allocate_asset_version(
        &scope,
        AudioAssetKind::VoiceClip,
        "asset-tts-studio-reference",
        "version-tts-studio-reference-a",
        AudioFileFormat::Wav,
    )?;

    let version = AudioAssetVersion {
        id: "version-tts-studio-reference-a".to_string(),
        asset_id: "asset-tts-studio-reference".to_string(),
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
            duration_ms: script.estimated_duration_ms(),
            loudness_lufs: Some(-18.0),
            true_peak_dbfs: Some(-2.0),
            has_clipping: false,
            bpm: None,
            musical_key: None,
            loop_points: None,
        },
        created_by: AssetCreation::Generated {
            recipe_id: "recipe-tts-studio-reference".to_string(),
            job_id: "job-tts-studio-reference".to_string(),
        },
        waveform_preview_cache: Some(storage.waveform_preview_path.clone()),
        spectrogram_preview_cache: Some(storage.spectrogram_preview_path.clone()),
    };

    let asset = AudioAsset {
        id: "asset-tts-studio-reference".to_string(),
        scope,
        kind: AudioAssetKind::VoiceClip,
        name: "TTS Studio narration draft".to_string(),
        tags: vec![
            "voice-clips".to_string(),
            "tts".to_string(),
            "multi-speaker".to_string(),
        ],
        collection_ids: vec!["collection-project-narration".to_string()],
        current_version_id: version.id.clone(),
        version_ids: vec![version.id.clone()],
        rights: RightsMetadata {
            license_status: LicenseStatus::ProviderLicensed,
            commercial_use: CommercialUseStatus::Allowed,
            voice_consent: VoiceConsentStatus::ExplicitConsentRecorded,
            ai_disclosure_required: true,
            watermark: WatermarkStatus::SidecarOnly,
            reference_media_ownership: Some(format!(
                "{}:{}",
                provider.provider_id, provider.model_id
            )),
        },
        provenance_ids: vec!["provenance-tts-studio-reference".to_string()],
    };

    Ok(TtsSavedOutput {
        asset,
        version,
        storage,
        promoted_to_project_library: true,
        waveform_preview_ready: true,
    })
}

fn reference_script() -> TtsScript {
    TtsScript {
        id: "script-launch-read".to_string(),
        title: "Launch read".to_string(),
        language: "en-US".to_string(),
        segments: vec![
            TtsScriptSegment {
                id: "seg-001".to_string(),
                position: 1,
                speaker_label: "Narrator".to_string(),
                text: "SoundWorks keeps the voice pass close to the edit.".to_string(),
                scene_label: Some("Intro".to_string()),
                target_duration_ms: Some(3100),
                regenerate_policy: SegmentRegeneratePolicy::KeepTimingWithNeighbors,
            },
            TtsScriptSegment {
                id: "seg-002".to_string(),
                position: 2,
                speaker_label: "Producer".to_string(),
                text: "Try the warmer take, but keep the last phrase locked to picture."
                    .to_string(),
                scene_label: Some("Direction".to_string()),
                target_duration_ms: Some(3900),
                regenerate_policy: SegmentRegeneratePolicy::RegenerateIndependently,
            },
            TtsScriptSegment {
                id: "seg-003".to_string(),
                position: 3,
                speaker_label: "Narrator".to_string(),
                text: "When it lands, save it as a project voice clip with the recipe attached."
                    .to_string(),
                scene_label: Some("Outro".to_string()),
                target_duration_ms: Some(4300),
                regenerate_policy: SegmentRegeneratePolicy::KeepTimingWithNeighbors,
            },
        ],
        pronunciation_dictionary: vec![PronunciationEntry {
            term: "SoundWorks".to_string(),
            pronunciation: "sound works".to_string(),
            applies_to_language: "en-US".to_string(),
        }],
    }
}

fn reference_speakers() -> Vec<TtsSpeaker> {
    vec![
        TtsSpeaker {
            label: "Narrator".to_string(),
            role: "Primary narration".to_string(),
            voice_profile_id: "voice-profile-narrator".to_string(),
            language: "en-US".to_string(),
            consent_required: true,
            consent_status: VoiceConsentStatus::ExplicitConsentRecorded,
        },
        TtsSpeaker {
            label: "Producer".to_string(),
            role: "Direction callout".to_string(),
            voice_profile_id: "voice-profile-producer".to_string(),
            language: "en-US".to_string(),
            consent_required: true,
            consent_status: VoiceConsentStatus::ExplicitConsentRecorded,
        },
    ]
}

fn reference_voice_profiles() -> Vec<VoiceProfile> {
    reference_speakers()
        .into_iter()
        .map(|speaker| VoiceProfile {
            id: speaker.voice_profile_id,
            display_name: format!("{} consented profile", speaker.label),
            source_reference_ids: vec![format!(
                "source-reference-{}",
                speaker.label.to_lowercase()
            )],
            consent: speaker.consent_status,
            allowed_uses: vec![VoiceUse::Tts, VoiceUse::ProjectOnly, VoiceUse::Commercial],
            provenance_ids: vec![format!("provenance-voice-{}", speaker.label.to_lowercase())],
        })
        .collect()
}

fn validation_checks() -> Vec<TtsValidationCheck> {
    vec![
        TtsValidationCheck {
            id: "tts.script_segments".to_string(),
            status: TtsValidationStatus::Passed,
            summary: "Script is segmented by speaker and scene for per-segment regeneration."
                .to_string(),
        },
        TtsValidationCheck {
            id: "tts.consent_gate".to_string(),
            status: TtsValidationStatus::Passed,
            summary: "Voice-clone capable generation requires explicit consent before submission."
                .to_string(),
        },
        TtsValidationCheck {
            id: "tts.asset_promotion".to_string(),
            status: TtsValidationStatus::Passed,
            summary:
                "Successful output is represented as a project Voice clip with recipe provenance."
                    .to_string(),
        },
    ]
}

fn estimate_segment_ms(text: &str) -> u64 {
    let word_count = text.split_whitespace().count().max(1) as u64;
    500 + word_count * 360
}

trait StudioInstallStatus {
    fn is_runnable_for_studio(&self) -> bool;
}

impl StudioInstallStatus for crate::manifests::ModelInstall {
    fn is_runnable_for_studio(&self) -> bool {
        matches!(
            self.status,
            crate::manifests::ModelInstallStatus::Installed
                | crate::manifests::ModelInstallStatus::Packaged
                | crate::manifests::ModelInstallStatus::External
        )
    }
}

trait CaseLabel {
    fn to_case_label(self) -> String;
}

impl CaseLabel for String {
    fn to_case_label(self) -> String {
        let mut label = String::new();
        for (index, character) in self.chars().enumerate() {
            if index > 0 && character.is_uppercase() {
                label.push('-');
            }
            label.push(character.to_ascii_lowercase());
        }
        label
    }
}

#[cfg(test)]
mod tests {
    use super::{TtsStudioOverview, TtsValidationStatus};

    #[test]
    fn reference_studio_preserves_multispeaker_script_contract() {
        let overview = TtsStudioOverview::reference().expect("tts studio builds");

        assert_eq!(overview.schema_version, 1);
        assert_eq!(overview.script.segments.len(), 3);
        assert_eq!(
            overview.script.speaker_labels(),
            vec!["Narrator".to_string(), "Producer".to_string()]
        );
        assert_eq!(overview.generation_plan.chunks.len(), 3);
        assert!(overview.generation_plan.preserves_speaker_consistency);
    }

    #[test]
    fn consent_gate_and_provider_limitations_are_visible_before_submission() {
        let overview = TtsStudioOverview::reference().expect("tts studio builds");

        assert!(overview
            .provider_options
            .iter()
            .any(|option| option.requires_voice_consent));
        assert!(overview
            .provider_options
            .iter()
            .flat_map(|option| option.limitations.iter())
            .any(|limitation| limitation.contains("Voice profile consent")));
        assert!(!overview.submission.can_submit);
        assert!(overview
            .submission
            .blocking_reasons
            .iter()
            .any(|reason| reason.contains("No runnable TTS provider is registered")));
    }

    #[test]
    fn submitted_tts_output_is_saved_as_project_voice_clip() {
        let overview = TtsStudioOverview::reference().expect("tts studio builds");

        assert_eq!(
            overview.saved_output.asset.kind.storage_dir(),
            "voice-clips"
        );
        assert_eq!(
            overview.saved_output.asset.current_version_id,
            overview.saved_output.version.id
        );
        assert!(overview
            .saved_output
            .version
            .file
            .storage_path
            .contains("/voice-clips/asset-tts-studio-reference/"));
        assert_eq!(
            overview.submission.recipe.output_asset_ids,
            vec![overview.saved_output.asset.id.clone()]
        );
        assert!(overview
            .validation_checks
            .iter()
            .all(|check| check.status == TtsValidationStatus::Passed));
    }
}
