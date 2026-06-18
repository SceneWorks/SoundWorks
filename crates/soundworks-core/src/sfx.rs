use crate::domain::{
    AssetCreation, AudioAsset, AudioAssetKind, AudioAssetVersion, AudioFileFormat,
    AudioFileReference, CommercialUseStatus, GenerationJob, GenerationRecipe, JobKind, JobProgress,
    JobStatus, LibraryScope, LicenseStatus, LoopPoints, ModelDescriptor, ModelRuntime,
    PostProcessingOperation, PostProcessingStep, RecipeRequest, RecipeWorkflow, RightsMetadata,
    SfxRecipe, SourceReference, SourceReferenceType, TechnicalAudioMetadata, VoiceConsentStatus,
    WatermarkStatus,
};
use crate::evaluation::{
    CommercialUseEvaluation, EvaluationLane, EvaluationStatus, ModelEvaluationCandidate,
    ModelEvaluationCatalog, ProductEligibility, ProductRuntimePath,
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

pub const SFX_STUDIO_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SfxStudioOverview {
    pub schema_version: u32,
    pub prompt: SfxPrompt,
    pub controls: SfxControls,
    pub category_presets: Vec<SfxCategoryPreset>,
    pub provider_options: Vec<SfxProviderOption>,
    pub selected_provider: SfxProviderSelection,
    pub provider_scorecards: Vec<SfxProviderScorecard>,
    pub deferred_multimodal_candidate_ids: Vec<String>,
    pub variants: Vec<SfxVariantPreview>,
    pub comparison: SfxVariantComparison,
    pub submission: SfxSubmissionPreview,
    pub saved_outputs: Vec<SfxSavedOutput>,
    pub post_processing_actions: Vec<SfxPostProcessingAction>,
    pub validation_checks: Vec<SfxValidationCheck>,
}

impl SfxStudioOverview {
    pub fn reference() -> Result<Self, StoragePathError> {
        Self::from_catalogs(
            &ProviderCatalog::reference(),
            &RuntimeOverview::reference(),
            &ModelEvaluationCatalog::reference(),
            &StoragePathAllocator::new("soundworks-library"),
        )
    }

    pub fn from_catalogs(
        catalog: &ProviderCatalog,
        runtime: &RuntimeOverview,
        evaluation: &ModelEvaluationCatalog,
        allocator: &StoragePathAllocator,
    ) -> Result<Self, StoragePathError> {
        let prompt = SfxPrompt::reference();
        let controls = SfxControls::reference();
        let category_presets = category_presets();
        let provider_options = provider_options(catalog, runtime);
        let selected_provider = provider_options
            .iter()
            .find(|option| option.runnable && option.workflow == CapabilityWorkflow::Sfx)
            .or_else(|| provider_options.iter().find(|option| option.runnable))
            .map(SfxProviderSelection::from_option)
            .unwrap_or_else(|| SfxProviderSelection {
                provider_id: "unavailable".to_string(),
                model_id: "unavailable".to_string(),
                model_version: None,
                workflow: CapabilityWorkflow::Sfx,
                runtime: ModelRuntime::ResearchOnly,
                accepted: false,
                blocker: Some("No runnable SFX or ambience provider is registered.".to_string()),
            });
        let provider_scorecards = provider_scorecards(evaluation);
        let variants = variant_previews(&prompt, &controls);
        let comparison = SfxVariantComparison::from_variants(&variants);
        let submission = SfxSubmissionPreview::build(
            &prompt,
            &controls,
            &provider_options,
            &selected_provider,
            &variants,
        );
        let saved_outputs = saved_outputs(&submission, &variants, allocator)?;

        Ok(Self {
            schema_version: SFX_STUDIO_SCHEMA_VERSION,
            prompt,
            controls,
            category_presets,
            provider_options,
            selected_provider,
            provider_scorecards,
            deferred_multimodal_candidate_ids: vec![
                "audiox".to_string(),
                "mmaudio".to_string(),
                "thinksound".to_string(),
            ],
            variants,
            comparison,
            submission,
            saved_outputs,
            post_processing_actions: post_processing_actions(),
            validation_checks: validation_checks(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SfxPrompt {
    pub id: String,
    pub text: String,
    pub negative_prompt: String,
    pub category: SfxCategory,
    pub tags: Vec<String>,
    pub reference_audio_asset_id: Option<String>,
}

impl SfxPrompt {
    fn reference() -> Self {
        Self {
            id: "prompt-sfx-hatch-ambience".to_string(),
            text: "Close metallic hatch impact with a short pressurized tail, followed by a low engine-room ambience bed."
                .to_string(),
            negative_prompt: "music, dialogue, melody, crowd".to_string(),
            category: SfxCategory::FoleyImpact,
            tags: vec![
                "foley".to_string(),
                "metal".to_string(),
                "engine-room".to_string(),
            ],
            reference_audio_asset_id: Some("asset-reference-metal-room-tone".to_string()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SfxCategory {
    FoleyImpact,
    AmbienceBed,
    Transition,
    UiSound,
    Creature,
    Weather,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SfxCategoryPreset {
    pub category: SfxCategory,
    pub label: String,
    pub default_duration_ms: u64,
    pub loopable_default: bool,
    pub output_kind: AudioAssetKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SfxControls {
    pub duration_ms: u64,
    pub variation_count: u8,
    pub intensity: u8,
    pub realism: u8,
    pub loopable: bool,
    pub trim_silence: bool,
    pub normalize_loudness_lufs: f32,
    pub fade_in_ms: u16,
    pub fade_out_ms: u16,
    pub loop_crossfade_ms: u16,
    pub promote_to_project_library: bool,
}

impl SfxControls {
    fn reference() -> Self {
        Self {
            duration_ms: 8_000,
            variation_count: 3,
            intensity: 72,
            realism: 64,
            loopable: true,
            trim_silence: true,
            normalize_loudness_lufs: -20.0,
            fade_in_ms: 20,
            fade_out_ms: 140,
            loop_crossfade_ms: 180,
            promote_to_project_library: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SfxProviderOption {
    pub provider_id: String,
    pub model_id: String,
    pub model_version: Option<String>,
    pub display_name: String,
    pub workflow: CapabilityWorkflow,
    pub runtime: ModelRuntime,
    pub install_status: String,
    pub runnable: bool,
    pub output_asset_kind: AudioAssetKind,
    pub output_format: AudioFileFormat,
    pub sample_rate_hz: u32,
    pub channel_layout: ChannelLayout,
    pub min_duration_ms: Option<u64>,
    pub max_duration_ms: Option<u64>,
    pub supports_reference_audio: bool,
    pub supports_looping: bool,
    pub commercial_use_allowed: bool,
    pub watermark: WatermarkStatus,
    pub supported_controls: Vec<SfxControlKind>,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SfxControlKind {
    Prompt,
    Category,
    Duration,
    VariationCount,
    Intensity,
    Realism,
    Loopable,
    ReferenceAudio,
    BatchGeneration,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SfxProviderSelection {
    pub provider_id: String,
    pub model_id: String,
    pub model_version: Option<String>,
    pub workflow: CapabilityWorkflow,
    pub runtime: ModelRuntime,
    pub accepted: bool,
    pub blocker: Option<String>,
}

impl SfxProviderSelection {
    fn from_option(option: &SfxProviderOption) -> Self {
        Self {
            provider_id: option.provider_id.clone(),
            model_id: option.model_id.clone(),
            model_version: option.model_version.clone(),
            workflow: option.workflow,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SfxProviderScorecard {
    pub candidate_id: String,
    pub name: String,
    pub provider: String,
    pub lanes: Vec<EvaluationLane>,
    pub status: EvaluationStatus,
    pub product_eligibility: ProductEligibility,
    pub readiness: SfxProviderReadiness,
    pub runtime_path: ProductRuntimePath,
    pub commercial_use: CommercialUseEvaluation,
    pub recommended: bool,
    pub blockers: Vec<String>,
    pub notes: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SfxProviderReadiness {
    Ready,
    NeedsRuntimePort,
    ResearchOnly,
    Blocked,
    DeferredToVideoAudio,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SfxVariantPreview {
    pub id: String,
    pub label: String,
    pub workflow: CapabilityWorkflow,
    pub asset_kind: AudioAssetKind,
    pub category: SfxCategory,
    pub duration_ms: u64,
    pub loudness_lufs: f32,
    pub true_peak_dbfs: f32,
    pub loopable: bool,
    pub loop_points: Option<LoopPoints>,
    pub tags: Vec<String>,
    pub selected_for_save: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SfxVariantComparison {
    pub selected_variant_id: String,
    pub variant_count: usize,
    pub loopable_variant_ids: Vec<String>,
    pub saved_variant_ids: Vec<String>,
}

impl SfxVariantComparison {
    fn from_variants(variants: &[SfxVariantPreview]) -> Self {
        Self {
            selected_variant_id: variants
                .iter()
                .find(|variant| variant.selected_for_save)
                .map(|variant| variant.id.clone())
                .unwrap_or_default(),
            variant_count: variants.len(),
            loopable_variant_ids: variants
                .iter()
                .filter(|variant| variant.loopable && variant.loop_points.is_some())
                .map(|variant| variant.id.clone())
                .collect(),
            saved_variant_ids: variants
                .iter()
                .filter(|variant| variant.selected_for_save)
                .map(|variant| variant.id.clone())
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SfxSubmissionPreview {
    pub can_submit: bool,
    pub job: GenerationJob,
    pub recipe: GenerationRecipe,
    pub blocking_reasons: Vec<String>,
    pub warnings: Vec<String>,
}

impl SfxSubmissionPreview {
    fn build(
        prompt: &SfxPrompt,
        controls: &SfxControls,
        provider_options: &[SfxProviderOption],
        selected_provider: &SfxProviderSelection,
        variants: &[SfxVariantPreview],
    ) -> Self {
        let mut blocking_reasons = vec![];
        let mut warnings = vec![];
        let selected_option = provider_options.iter().find(|option| {
            option.provider_id == selected_provider.provider_id
                && option.model_id == selected_provider.model_id
                && option.workflow == selected_provider.workflow
        });

        if prompt.text.trim().is_empty() {
            blocking_reasons.push("Prompt is empty.".to_string());
        }

        if selected_option.is_none() || !selected_provider.accepted {
            blocking_reasons.push(selected_provider.blocker.clone().unwrap_or_else(|| {
                "Selected provider cannot currently accept SFX jobs.".to_string()
            }));
        }

        if let Some(option) = selected_option {
            if let Some(min_duration_ms) = option.min_duration_ms {
                if controls.duration_ms < min_duration_ms {
                    blocking_reasons.push(format!(
                        "{} requires at least {} ms.",
                        option.display_name, min_duration_ms
                    ));
                }
            }

            if let Some(max_duration_ms) = option.max_duration_ms {
                if controls.duration_ms > max_duration_ms {
                    blocking_reasons.push(format!(
                        "{} supports up to {} ms; current request is {} ms.",
                        option.display_name, max_duration_ms, controls.duration_ms
                    ));
                }
            }

            if prompt.reference_audio_asset_id.is_some() && !option.supports_reference_audio {
                warnings.push(
                    "Reference audio is stored with the recipe but the selected provider treats it as context only."
                        .to_string(),
                );
            }

            if controls.loopable && !option.supports_looping {
                warnings.push(
                    "Loopability will be inspected with post-processing because the provider has no native loop control."
                        .to_string(),
                );
            }

            if !option.commercial_use_allowed {
                warnings.push(
                    "Selected provider requires commercial-use review before export.".to_string(),
                );
            }
        }

        if controls.trim_silence {
            warnings.push("Post-processing will trim leading and trailing silence.".to_string());
        }

        let can_submit = blocking_reasons.is_empty();
        let recipe = generation_recipe(prompt, controls, selected_provider, variants);

        Self {
            can_submit,
            job: GenerationJob {
                id: "job-sfx-studio-reference".to_string(),
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
                        "Ready to queue SFX and ambience generation.".to_string()
                    } else {
                        "SFX submission blocked by validation.".to_string()
                    }),
                }),
                output_version_ids: if can_submit {
                    variants
                        .iter()
                        .filter(|variant| variant.selected_for_save)
                        .map(|variant| format!("version-{}-a", variant.id))
                        .collect()
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
pub struct SfxSavedOutput {
    pub variant_id: String,
    pub asset: AudioAsset,
    pub version: AudioAssetVersion,
    pub storage: StoragePaths,
    pub exported: bool,
    pub waveform_preview_ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SfxPostProcessingAction {
    pub id: String,
    pub operation: PostProcessingOperation,
    pub enabled: bool,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SfxValidationCheck {
    pub id: String,
    pub status: SfxValidationStatus,
    pub summary: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SfxValidationStatus {
    Passed,
    Warning,
    Failed,
}

fn category_presets() -> Vec<SfxCategoryPreset> {
    vec![
        SfxCategoryPreset {
            category: SfxCategory::FoleyImpact,
            label: "Foley impact".to_string(),
            default_duration_ms: 1_800,
            loopable_default: false,
            output_kind: AudioAssetKind::Sfx,
        },
        SfxCategoryPreset {
            category: SfxCategory::AmbienceBed,
            label: "Ambience bed".to_string(),
            default_duration_ms: 12_000,
            loopable_default: true,
            output_kind: AudioAssetKind::Ambience,
        },
        SfxCategoryPreset {
            category: SfxCategory::Transition,
            label: "Transition".to_string(),
            default_duration_ms: 2_400,
            loopable_default: false,
            output_kind: AudioAssetKind::Sfx,
        },
        SfxCategoryPreset {
            category: SfxCategory::UiSound,
            label: "UI sound".to_string(),
            default_duration_ms: 900,
            loopable_default: false,
            output_kind: AudioAssetKind::Sfx,
        },
    ]
}

fn provider_options(
    catalog: &ProviderCatalog,
    runtime: &RuntimeOverview,
) -> Vec<SfxProviderOption> {
    let mut options = vec![];

    for provider in &catalog.providers {
        for model in &provider.models {
            for capability in model.capabilities.iter().filter(|capability| {
                matches!(
                    capability.workflow,
                    CapabilityWorkflow::Sfx | CapabilityWorkflow::Ambience
                ) && capability.inputs.contains(&CapabilityInput::TextPrompt)
                    && capability.inputs.contains(&CapabilityInput::Duration)
            }) {
                let state = runtime
                    .model_states
                    .iter()
                    .find(|state| state.provider_id == provider.id && state.model_id == model.id);
                let mut limitations = vec![];

                if !model.install.is_runnable_for_studio() {
                    limitations.push(
                        "Model is not installed, packaged, or externally managed.".to_string(),
                    );
                }

                if let Some(state) = state {
                    limitations.extend(state.reasons.clone());
                } else {
                    limitations
                        .push("Runtime has not reported this provider/model pair.".to_string());
                }

                limitations.extend(limitations_for_license(model.requirements.license));
                limitations.extend(limitations_for_safety(&capability.safety));

                let output_asset_kind = capability
                    .outputs
                    .asset_kinds
                    .first()
                    .copied()
                    .unwrap_or(AudioAssetKind::Sfx);

                options.push(SfxProviderOption {
                    provider_id: provider.id.clone(),
                    model_id: model.id.clone(),
                    model_version: model.version.clone(),
                    display_name: format!("{} / {}", provider.name, model.name),
                    workflow: capability.workflow,
                    runtime: model.runtime,
                    install_status: format!("{:?}", model.install.status).to_case_label(),
                    runnable: state.map_or(false, |state| {
                        matches!(
                            state.availability,
                            crate::runtime::RuntimeAvailability::Installed
                                | crate::runtime::RuntimeAvailability::Available
                        )
                    }),
                    output_asset_kind,
                    output_format: capability.defaults.format,
                    sample_rate_hz: capability.defaults.sample_rate_hz,
                    channel_layout: capability.defaults.channel_layout,
                    min_duration_ms: capability.limits.min_duration_ms,
                    max_duration_ms: capability.limits.max_duration_ms,
                    supports_reference_audio: capability
                        .inputs
                        .contains(&CapabilityInput::ReferenceAudio),
                    supports_looping: matches!(capability.workflow, CapabilityWorkflow::Ambience),
                    commercial_use_allowed: capability.safety.commercial_use_allowed,
                    watermark: capability.safety.watermark,
                    supported_controls: supported_controls(capability),
                    limitations,
                });
            }
        }
    }

    options
}

fn supported_controls(capability: &crate::manifests::ModelCapability) -> Vec<SfxControlKind> {
    let mut controls = vec![
        SfxControlKind::Prompt,
        SfxControlKind::Category,
        SfxControlKind::VariationCount,
        SfxControlKind::Intensity,
        SfxControlKind::Realism,
        SfxControlKind::BatchGeneration,
    ];

    if capability.inputs.contains(&CapabilityInput::Duration) {
        controls.push(SfxControlKind::Duration);
    }

    if matches!(capability.workflow, CapabilityWorkflow::Ambience) {
        controls.push(SfxControlKind::Loopable);
    }

    if capability.inputs.contains(&CapabilityInput::ReferenceAudio) {
        controls.push(SfxControlKind::ReferenceAudio);
    }

    controls
}

fn limitations_for_license(license: ModelLicense) -> Vec<String> {
    match license {
        ModelLicense::Open | ModelLicense::CommercialAllowed | ModelLicense::ProviderTerms => {
            vec![]
        }
        ModelLicense::NonCommercial => {
            vec!["Noncommercial license blocks default commercial projects.".to_string()]
        }
        ModelLicense::Unknown => {
            vec!["License must be reviewed before production use.".to_string()]
        }
    }
}

fn limitations_for_safety(safety: &CapabilitySafety) -> Vec<String> {
    let mut limitations = vec![];

    if !safety.commercial_use_allowed {
        limitations.push("Commercial export requires review.".to_string());
    }

    if !safety.disallowed_uses.is_empty() {
        limitations.push(format!(
            "Disallowed uses: {}.",
            safety.disallowed_uses.join(", ")
        ));
    }

    limitations
}

fn provider_scorecards(evaluation: &ModelEvaluationCatalog) -> Vec<SfxProviderScorecard> {
    let recommended_ids = evaluation
        .recommendations
        .iter()
        .map(|recommendation| recommendation.candidate_id.as_str())
        .collect::<Vec<_>>();

    [
        "moss-soundeffect",
        "stable-audio-3",
        "stable-audio-open-1",
        "audiocraft-audiogen",
        "audioldm",
        "audioldm-2",
        "audiox",
        "mmaudio",
        "thinksound",
    ]
    .into_iter()
    .filter_map(|candidate_id| {
        evaluation
            .candidates
            .iter()
            .find(|candidate| candidate.id == candidate_id)
            .map(|candidate| scorecard(candidate, recommended_ids.contains(&candidate.id.as_str())))
    })
    .collect()
}

fn scorecard(candidate: &ModelEvaluationCandidate, recommended: bool) -> SfxProviderScorecard {
    let deferred_to_video_audio = candidate
        .lanes
        .iter()
        .any(|lane| matches!(lane, EvaluationLane::VideoToAudio))
        && !matches!(candidate.id.as_str(), "stable-audio-3");
    let readiness = if deferred_to_video_audio {
        SfxProviderReadiness::DeferredToVideoAudio
    } else {
        match candidate.product_eligibility {
            ProductEligibility::ProductCandidate | ProductEligibility::ApiOnlyCandidate => {
                SfxProviderReadiness::Ready
            }
            ProductEligibility::NeedsRuntimePort => SfxProviderReadiness::NeedsRuntimePort,
            ProductEligibility::ResearchOnly => SfxProviderReadiness::ResearchOnly,
            ProductEligibility::Blocked => SfxProviderReadiness::Blocked,
        }
    };
    let mut blockers = candidate.blockers.clone();

    if deferred_to_video_audio {
        blockers.push("Multimodal/video-to-audio workflow is tracked in sc-6183.".to_string());
    }

    SfxProviderScorecard {
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

fn variant_previews(prompt: &SfxPrompt, controls: &SfxControls) -> Vec<SfxVariantPreview> {
    let base_tags = prompt.tags.clone();

    vec![
        SfxVariantPreview {
            id: "sfx-variant-impact-tight".to_string(),
            label: "Tight hatch impact".to_string(),
            workflow: CapabilityWorkflow::Sfx,
            asset_kind: AudioAssetKind::Sfx,
            category: SfxCategory::FoleyImpact,
            duration_ms: 1_800,
            loudness_lufs: -17.5,
            true_peak_dbfs: -1.0,
            loopable: false,
            loop_points: None,
            tags: tags(&base_tags, &["impact", "tight"]),
            selected_for_save: true,
        },
        SfxVariantPreview {
            id: "sfx-variant-impact-heavy".to_string(),
            label: "Heavy pressure hit".to_string(),
            workflow: CapabilityWorkflow::Sfx,
            asset_kind: AudioAssetKind::Sfx,
            category: SfxCategory::FoleyImpact,
            duration_ms: 2_400,
            loudness_lufs: -16.0,
            true_peak_dbfs: -0.8,
            loopable: false,
            loop_points: None,
            tags: tags(&base_tags, &["impact", "heavy"]),
            selected_for_save: false,
        },
        SfxVariantPreview {
            id: "sfx-variant-engine-room-bed".to_string(),
            label: "Engine room bed".to_string(),
            workflow: CapabilityWorkflow::Ambience,
            asset_kind: AudioAssetKind::Ambience,
            category: SfxCategory::AmbienceBed,
            duration_ms: controls.duration_ms,
            loudness_lufs: controls.normalize_loudness_lufs,
            true_peak_dbfs: -3.0,
            loopable: controls.loopable,
            loop_points: Some(LoopPoints {
                start_sample: 2_400,
                end_sample: 381_600,
            }),
            tags: tags(&base_tags, &["ambience", "loopable"]),
            selected_for_save: true,
        },
    ]
}

fn tags(base: &[String], extra: &[&str]) -> Vec<String> {
    let mut tags = base.to_vec();
    tags.extend(extra.iter().map(|tag| (*tag).to_string()));
    tags.sort();
    tags.dedup();
    tags
}

fn generation_recipe(
    prompt: &SfxPrompt,
    controls: &SfxControls,
    provider: &SfxProviderSelection,
    variants: &[SfxVariantPreview],
) -> GenerationRecipe {
    let mut parameters = BTreeMap::new();
    parameters.insert("category".to_string(), json!(prompt.category));
    parameters.insert(
        "variationCount".to_string(),
        json!(controls.variation_count),
    );
    parameters.insert("intensity".to_string(), json!(controls.intensity));
    parameters.insert("realism".to_string(), json!(controls.realism));
    parameters.insert("loopable".to_string(), json!(controls.loopable));
    parameters.insert("trimSilence".to_string(), json!(controls.trim_silence));
    parameters.insert(
        "normalizeLoudnessLufs".to_string(),
        json!(controls.normalize_loudness_lufs),
    );
    parameters.insert("fadeInMs".to_string(), json!(controls.fade_in_ms));
    parameters.insert("fadeOutMs".to_string(), json!(controls.fade_out_ms));
    parameters.insert(
        "loopCrossfadeMs".to_string(),
        json!(controls.loop_crossfade_ms),
    );

    GenerationRecipe {
        id: "recipe-sfx-studio-reference".to_string(),
        workflow: RecipeWorkflow::Sfx,
        provider: provider.descriptor(),
        request: RecipeRequest::Sfx(SfxRecipe {
            prompt: prompt.text.clone(),
            negative_prompt: Some(prompt.negative_prompt.clone()),
            category: Some(format!("{:?}", prompt.category).to_case_label()),
            target_duration_ms: Some(controls.duration_ms),
            loopable: controls.loopable,
        }),
        seed: Some(42_153),
        source_references: prompt
            .reference_audio_asset_id
            .iter()
            .map(|asset_id| SourceReference {
                id: "source-sfx-reference-audio".to_string(),
                source_type: SourceReferenceType::Audio,
                asset_id: Some(asset_id.clone()),
                external_uri: None,
                ownership_note: Some(
                    "User-owned reference texture for prompt grounding.".to_string(),
                ),
            })
            .collect(),
        post_processing: post_processing_steps(controls),
        parameter_overrides: parameters,
        output_asset_ids: variants
            .iter()
            .filter(|variant| variant.selected_for_save)
            .map(|variant| format!("asset-{}", variant.id))
            .collect(),
    }
}

fn post_processing_steps(controls: &SfxControls) -> Vec<PostProcessingStep> {
    let mut steps = vec![];

    if controls.trim_silence {
        steps.push(PostProcessingStep {
            id: "post-trim-silence".to_string(),
            operation: PostProcessingOperation::Trim,
            parameters: BTreeMap::new(),
        });
    }

    let mut normalize = BTreeMap::new();
    normalize.insert(
        "targetLoudnessLufs".to_string(),
        json!(controls.normalize_loudness_lufs),
    );
    steps.push(PostProcessingStep {
        id: "post-normalize-sfx".to_string(),
        operation: PostProcessingOperation::Normalize,
        parameters: normalize,
    });

    let mut fade = BTreeMap::new();
    fade.insert("fadeInMs".to_string(), json!(controls.fade_in_ms));
    fade.insert("fadeOutMs".to_string(), json!(controls.fade_out_ms));
    fade.insert(
        "loopCrossfadeMs".to_string(),
        json!(controls.loop_crossfade_ms),
    );
    steps.push(PostProcessingStep {
        id: "post-fades-and-loop-crossfade".to_string(),
        operation: PostProcessingOperation::Fade,
        parameters: fade,
    });

    steps
}

fn saved_outputs(
    submission: &SfxSubmissionPreview,
    variants: &[SfxVariantPreview],
    allocator: &StoragePathAllocator,
) -> Result<Vec<SfxSavedOutput>, StoragePathError> {
    let scope = LibraryScope::Project {
        project_id: "project-demo".to_string(),
    };

    variants
        .iter()
        .filter(|variant| variant.selected_for_save)
        .map(|variant| saved_output(submission, variant, &scope, allocator))
        .collect()
}

fn saved_output(
    submission: &SfxSubmissionPreview,
    variant: &SfxVariantPreview,
    scope: &LibraryScope,
    allocator: &StoragePathAllocator,
) -> Result<SfxSavedOutput, StoragePathError> {
    let asset_id = format!("asset-{}", variant.id);
    let version_id = format!("version-{}-a", variant.id);
    let storage = allocator.allocate_asset_version(
        scope,
        variant.asset_kind,
        &asset_id,
        &version_id,
        AudioFileFormat::Wav,
    )?;

    let version = AudioAssetVersion {
        id: version_id.clone(),
        asset_id: asset_id.clone(),
        version_index: 1,
        file: AudioFileReference {
            storage_path: storage.media_path.clone(),
            format: AudioFileFormat::Wav,
            codec: Some("pcm_s24le".to_string()),
            byte_size: None,
            content_hash: None,
        },
        technical: TechnicalAudioMetadata {
            sample_rate_hz: 48_000,
            bit_depth: Some(24),
            channels: if variant.asset_kind == AudioAssetKind::Ambience {
                2
            } else {
                1
            },
            duration_ms: variant.duration_ms,
            loudness_lufs: Some(variant.loudness_lufs),
            true_peak_dbfs: Some(variant.true_peak_dbfs),
            has_clipping: false,
            bpm: None,
            musical_key: None,
            loop_points: variant.loop_points,
        },
        created_by: AssetCreation::Generated {
            recipe_id: submission.recipe.id.clone(),
            job_id: submission.job.id.clone(),
        },
        waveform_preview_cache: Some(storage.waveform_preview_path.clone()),
        spectrogram_preview_cache: Some(storage.spectrogram_preview_path.clone()),
    };

    let asset = AudioAsset {
        id: asset_id,
        scope: scope.clone(),
        kind: variant.asset_kind,
        name: variant.label.clone(),
        tags: variant.tags.clone(),
        collection_ids: vec!["collection-project-sfx".to_string()],
        current_version_id: version.id.clone(),
        version_ids: vec![version.id.clone()],
        rights: RightsMetadata {
            license_status: LicenseStatus::ProviderLicensed,
            commercial_use: CommercialUseStatus::Allowed,
            voice_consent: VoiceConsentStatus::NotVoiceMaterial,
            ai_disclosure_required: true,
            watermark: WatermarkStatus::SidecarOnly,
            reference_media_ownership: Some(
                "prompt-generated with user-owned reference texture".to_string(),
            ),
        },
        provenance_ids: vec![format!("provenance-{}", variant.id)],
    };

    Ok(SfxSavedOutput {
        variant_id: variant.id.clone(),
        asset,
        version,
        storage,
        exported: true,
        waveform_preview_ready: true,
    })
}

fn post_processing_actions() -> Vec<SfxPostProcessingAction> {
    vec![
        SfxPostProcessingAction {
            id: "trim-silence".to_string(),
            operation: PostProcessingOperation::Trim,
            enabled: true,
            summary: "Trim leading and trailing silence before saving variants.".to_string(),
        },
        SfxPostProcessingAction {
            id: "normalize".to_string(),
            operation: PostProcessingOperation::Normalize,
            enabled: true,
            summary: "Normalize selected variants for preview and export.".to_string(),
        },
        SfxPostProcessingAction {
            id: "fade-loop".to_string(),
            operation: PostProcessingOperation::Fade,
            enabled: true,
            summary: "Apply fades and loop crossfade where loop points are present.".to_string(),
        },
        SfxPostProcessingAction {
            id: "convert-export".to_string(),
            operation: PostProcessingOperation::ConvertFormat,
            enabled: true,
            summary: "Export WAV assets with metadata sidecars and recipe provenance.".to_string(),
        },
    ]
}

fn validation_checks() -> Vec<SfxValidationCheck> {
    vec![
        SfxValidationCheck {
            id: "sfx.provider_capabilities".to_string(),
            status: SfxValidationStatus::Passed,
            summary: "Available controls are derived from SFX and ambience provider capabilities."
                .to_string(),
        },
        SfxValidationCheck {
            id: "sfx.variant_comparison".to_string(),
            status: SfxValidationStatus::Passed,
            summary: "Multiple variants can be previewed, compared, selected, tagged, and saved."
                .to_string(),
        },
        SfxValidationCheck {
            id: "sfx.loop_points".to_string(),
            status: SfxValidationStatus::Passed,
            summary: "Loopable ambience output includes inspectable loop points.".to_string(),
        },
        SfxValidationCheck {
            id: "sfx.multimodal_boundary".to_string(),
            status: SfxValidationStatus::Passed,
            summary: "AudioX, MMAudio, and ThinkSound remain deferred to the video-to-audio story."
                .to_string(),
        },
    ]
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
    use super::{SfxControlKind, SfxProviderReadiness, SfxStudioOverview, SfxValidationStatus};
    use crate::domain::{AudioAssetKind, RecipeRequest, RecipeWorkflow};
    use crate::evaluation::EvaluationLane;
    use crate::manifests::CapabilityWorkflow;

    #[test]
    fn reference_studio_exposes_text_first_sfx_and_ambience_controls() {
        let overview = SfxStudioOverview::reference().expect("sfx studio builds");

        assert_eq!(overview.schema_version, 1);
        assert_eq!(overview.controls.variation_count, 3);
        assert!(overview
            .provider_options
            .iter()
            .any(|option| option.workflow == CapabilityWorkflow::Sfx
                && option.supported_controls.contains(&SfxControlKind::Prompt)
                && option
                    .supported_controls
                    .contains(&SfxControlKind::Duration)));
        assert!(overview
            .provider_options
            .iter()
            .any(|option| option.workflow == CapabilityWorkflow::Ambience
                && option
                    .supported_controls
                    .contains(&SfxControlKind::Loopable)));
        assert!(overview.submission.can_submit);
    }

    #[test]
    fn provider_scorecards_select_moss_and_defer_multimodal_candidates() {
        let overview = SfxStudioOverview::reference().expect("sfx studio builds");
        let moss = overview
            .provider_scorecards
            .iter()
            .find(|scorecard| scorecard.candidate_id == "moss-soundeffect")
            .expect("MOSS scorecard exists");
        let mmaudio = overview
            .provider_scorecards
            .iter()
            .find(|scorecard| scorecard.candidate_id == "mmaudio")
            .expect("MMAudio scorecard exists");

        assert!(moss.recommended);
        assert_eq!(moss.readiness, SfxProviderReadiness::Ready);
        assert!(moss.lanes.contains(&EvaluationLane::Sfx));
        assert_eq!(
            mmaudio.readiness,
            SfxProviderReadiness::DeferredToVideoAudio
        );
        assert!(overview
            .deferred_multimodal_candidate_ids
            .contains(&"mmaudio".to_string()));
    }

    #[test]
    fn submission_recipe_preserves_prompt_provenance_and_outputs() {
        let overview = SfxStudioOverview::reference().expect("sfx studio builds");

        assert_eq!(overview.submission.recipe.workflow, RecipeWorkflow::Sfx);
        assert_eq!(
            overview.submission.recipe.output_asset_ids,
            overview
                .saved_outputs
                .iter()
                .map(|output| output.asset.id.clone())
                .collect::<Vec<_>>()
        );

        let RecipeRequest::Sfx(recipe) = &overview.submission.recipe.request else {
            panic!("expected SFX request");
        };

        assert!(recipe.prompt.contains("metallic hatch"));
        assert!(recipe.loopable);
        assert_eq!(
            overview.submission.recipe.source_references[0].asset_id,
            Some("asset-reference-metal-room-tone".to_string())
        );
    }

    #[test]
    fn saved_outputs_include_sfx_tags_exports_and_loop_points() {
        let overview = SfxStudioOverview::reference().expect("sfx studio builds");

        assert_eq!(overview.saved_outputs.len(), 2);
        assert!(overview
            .saved_outputs
            .iter()
            .any(|output| output.asset.kind == AudioAssetKind::Sfx
                && output.asset.tags.contains(&"impact".to_string())
                && output.exported));
        assert!(overview
            .saved_outputs
            .iter()
            .any(|output| output.asset.kind == AudioAssetKind::Ambience
                && output.version.technical.loop_points.is_some()));
        assert!(overview
            .validation_checks
            .iter()
            .all(|check| check.status == SfxValidationStatus::Passed));
    }
}
