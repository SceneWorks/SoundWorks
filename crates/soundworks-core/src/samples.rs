use crate::domain::{
    AssetCreation, AudioAsset, AudioAssetKind, AudioAssetVersion, AudioFileFormat,
    AudioFileReference, CommercialUseStatus, GenerationJob, GenerationRecipe,
    InstrumentSampleRecipe, JobKind, JobProgress, JobStatus, LibraryScope, LicenseStatus,
    LoopPoints, LoopRecipe, ModelDescriptor, ModelRuntime, PostProcessingOperation,
    PostProcessingStep, RecipeRequest, RecipeWorkflow, RightsMetadata, SourceReference,
    SourceReferenceType, TechnicalAudioMetadata, VoiceConsentStatus, WatermarkStatus,
};
use crate::evaluation::{
    EvaluationLane, ModelEvaluationCandidate, ModelEvaluationCatalog, ProductEligibility,
};
use crate::manifests::{CapabilityInput, CapabilityWorkflow, ChannelLayout, ProviderCatalog};
use crate::runtime::RuntimeOverview;
use crate::storage::{StoragePathAllocator, StoragePathError, StoragePaths};
use crate::studio_common::{
    kebab_label, limitations_for_license, limitations_for_safety, SafetyLimitationOptions,
    StudioInstallStatus,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;

pub const SAMPLES_STUDIO_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SamplesStudioOverview {
    pub schema_version: u32,
    pub prompt: SamplePrompt,
    pub controls: SampleControls,
    pub provider_options: Vec<SampleProviderOption>,
    pub selected_provider: SampleProviderSelection,
    pub provider_scorecards: Vec<SampleProviderScorecard>,
    pub variants: Vec<SampleVariantPreview>,
    pub pack: SamplePackPreview,
    pub submission: SampleSubmissionPreview,
    pub saved_outputs: Vec<SampleSavedOutput>,
    pub post_processing_actions: Vec<SamplePostProcessingAction>,
    pub qa_checks: Vec<SampleQaCheck>,
}

impl SamplesStudioOverview {
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
        let prompt = SamplePrompt::reference();
        let controls = SampleControls::reference();
        let provider_options = provider_options(catalog, runtime);
        let selected_provider = provider_options
            .iter()
            .find(|option| option.runnable && option.workflow == CapabilityWorkflow::Loop)
            .or_else(|| {
                provider_options
                    .iter()
                    .find(|option| option.workflow == CapabilityWorkflow::InstrumentSample)
            })
            .map(SampleProviderSelection::from_option)
            .unwrap_or_else(|| SampleProviderSelection {
                provider_id: "unavailable".to_string(),
                model_id: "unavailable".to_string(),
                model_version: None,
                workflow: CapabilityWorkflow::Loop,
                runtime: ModelRuntime::ResearchOnly,
                accepted: false,
                blocker: Some(
                    "No runnable instrument sample or loop provider is registered.".to_string(),
                ),
            });
        let provider_scorecards = provider_scorecards(evaluation);
        let variants = variant_previews(&prompt, &controls);
        let pack = SamplePackPreview::from_variants(&variants);
        let submission = SampleSubmissionPreview::build(
            &prompt,
            &controls,
            &provider_options,
            &selected_provider,
            &variants,
        );
        let saved_outputs = saved_outputs(&submission, &variants, allocator)?;

        Ok(Self {
            schema_version: SAMPLES_STUDIO_SCHEMA_VERSION,
            prompt,
            controls,
            provider_options,
            selected_provider,
            provider_scorecards,
            variants,
            pack,
            submission,
            saved_outputs,
            post_processing_actions: post_processing_actions(),
            qa_checks: qa_checks(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SamplePrompt {
    pub id: String,
    pub text: String,
    pub negative_prompt: String,
    pub instrument_family: InstrumentFamily,
    pub articulation: String,
    pub genre_tags: Vec<String>,
    pub reference_audio_asset_id: Option<String>,
}

impl SamplePrompt {
    fn reference() -> Self {
        Self {
            id: "prompt-sample-pack-synth-bass".to_string(),
            text: "Tight analog synth bass one-shots and a four-bar driving loop for a neon chase cue."
                .to_string(),
            negative_prompt: "full song, vocal, crowd, reverb wash".to_string(),
            instrument_family: InstrumentFamily::SynthBass,
            articulation: "pluck and short sustain".to_string(),
            genre_tags: vec![
                "synthwave".to_string(),
                "game-score".to_string(),
                "bass".to_string(),
            ],
            reference_audio_asset_id: Some("asset-reference-neon-bass".to_string()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InstrumentFamily {
    Drums,
    Bass,
    SynthBass,
    Guitar,
    Keys,
    Strings,
    Brass,
    Texture,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SampleControls {
    pub musical_key: String,
    pub scale: String,
    pub bpm: f32,
    pub bars: u16,
    pub beats: u16,
    pub loopable: bool,
    pub dry_wet_ambience: u8,
    pub velocity_energy: u8,
    pub variation_count: u8,
    pub batch_size: u8,
    pub promote_to_project_library: bool,
}

impl SampleControls {
    fn reference() -> Self {
        Self {
            musical_key: "A minor".to_string(),
            scale: "natural minor".to_string(),
            bpm: 120.0,
            bars: 4,
            beats: 4,
            loopable: true,
            dry_wet_ambience: 18,
            velocity_energy: 76,
            variation_count: 4,
            batch_size: 6,
            promote_to_project_library: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SampleProviderOption {
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
    pub supports_tempo: bool,
    pub supports_key: bool,
    pub supports_loop_points: bool,
    pub commercial_use_allowed: bool,
    pub watermark: WatermarkStatus,
    pub supported_controls: Vec<SampleControlKind>,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SampleControlKind {
    Prompt,
    InstrumentFamily,
    Articulation,
    MusicalKey,
    Scale,
    Tempo,
    BarsBeats,
    Loopable,
    DryWetAmbience,
    VelocityEnergy,
    ReferenceAudio,
    BatchGeneration,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SampleProviderSelection {
    pub provider_id: String,
    pub model_id: String,
    pub model_version: Option<String>,
    pub workflow: CapabilityWorkflow,
    pub runtime: ModelRuntime,
    pub accepted: bool,
    pub blocker: Option<String>,
}

impl SampleProviderSelection {
    fn from_option(option: &SampleProviderOption) -> Self {
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

pub type SampleProviderScorecard = crate::studio_common::ProviderScorecard<SampleProviderReadiness>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SampleProviderReadiness {
    Ready,
    NeedsRuntimePort,
    ResearchOnly,
    Blocked,
    NotSampleFocused,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SampleVariantPreview {
    pub id: String,
    pub label: String,
    pub workflow: CapabilityWorkflow,
    pub asset_kind: AudioAssetKind,
    pub instrument_family: InstrumentFamily,
    pub articulation: String,
    pub duration_ms: u64,
    pub bpm: Option<f32>,
    pub musical_key: Option<String>,
    pub time_signature: Option<String>,
    pub loop_points: Option<LoopPoints>,
    pub transient_one_shot: bool,
    pub loudness_lufs: f32,
    pub true_peak_dbfs: f32,
    pub has_clipping: bool,
    pub tags: Vec<String>,
    pub collection_id: String,
    pub selected_for_pack: bool,
    pub favorite: bool,
    pub duplicate_of_variant_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SamplePackPreview {
    pub collection_id: String,
    pub name: String,
    pub variant_count: usize,
    pub selected_variant_ids: Vec<String>,
    pub favorite_variant_ids: Vec<String>,
    pub loop_variant_ids: Vec<String>,
    pub one_shot_variant_ids: Vec<String>,
    pub export_formats: Vec<AudioFileFormat>,
}

impl SamplePackPreview {
    fn from_variants(variants: &[SampleVariantPreview]) -> Self {
        Self {
            collection_id: "collection-neon-bass-pack".to_string(),
            name: "Neon bass starter pack".to_string(),
            variant_count: variants.len(),
            selected_variant_ids: variants
                .iter()
                .filter(|variant| variant.selected_for_pack)
                .map(|variant| variant.id.clone())
                .collect(),
            favorite_variant_ids: variants
                .iter()
                .filter(|variant| variant.favorite)
                .map(|variant| variant.id.clone())
                .collect(),
            loop_variant_ids: variants
                .iter()
                .filter(|variant| variant.asset_kind == AudioAssetKind::Loop)
                .map(|variant| variant.id.clone())
                .collect(),
            one_shot_variant_ids: variants
                .iter()
                .filter(|variant| variant.transient_one_shot)
                .map(|variant| variant.id.clone())
                .collect(),
            export_formats: vec![AudioFileFormat::Wav, AudioFileFormat::Flac],
        }
    }
}

/// UI submission readiness preview (batched: plural jobs/recipes). Like the
/// shared `StudioSubmissionPreview`, `can_submit`/`blocking_reasons` are a
/// display contract computed from reference inputs, NOT the authoritative gate —
/// the real gate is `runtime::RuntimeJobStore::enqueue` (F-021).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SampleSubmissionPreview {
    pub can_submit: bool,
    pub jobs: Vec<GenerationJob>,
    pub recipes: Vec<GenerationRecipe>,
    pub blocking_reasons: Vec<String>,
    pub warnings: Vec<String>,
}

impl SampleSubmissionPreview {
    fn build(
        prompt: &SamplePrompt,
        controls: &SampleControls,
        provider_options: &[SampleProviderOption],
        selected_provider: &SampleProviderSelection,
        variants: &[SampleVariantPreview],
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
                "Selected provider cannot currently accept sample or loop jobs.".to_string()
            }));
        }

        if let Some(option) = selected_option {
            if controls.loopable && !option.supports_loop_points {
                warnings.push(
                    "Loop points will be verified with post-processing because the provider does not expose native loop metadata."
                        .to_string(),
                );
            }

            if !option.supports_tempo {
                warnings.push(
                    "BPM is stored with the recipe and validated after generation.".to_string(),
                );
            }

            if !option.supports_key {
                warnings.push("Key detection will verify the requested musical key.".to_string());
            }

            if prompt.reference_audio_asset_id.is_some() && !option.supports_reference_audio {
                warnings.push(
                    "Reference audio is stored with provenance but unavailable as a provider input."
                        .to_string(),
                );
            }

            if !option.commercial_use_allowed {
                warnings
                    .push("Selected provider requires license review before export.".to_string());
            }
        }

        if controls.dry_wet_ambience > 40 {
            warnings.push("High ambience may reduce reusable sample isolation.".to_string());
        }

        let can_submit = blocking_reasons.is_empty();
        let recipes = generation_recipes(prompt, controls, selected_provider, variants);
        let jobs = recipes
            .iter()
            .map(|recipe| GenerationJob {
                id: format!("job-{}", recipe.id),
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
                        "Ready to queue samples and loops generation.".to_string()
                    } else {
                        "Samples submission blocked by validation.".to_string()
                    }),
                }),
                output_version_ids: if can_submit {
                    recipe
                        .output_asset_ids
                        .iter()
                        .map(|asset_id| {
                            format!("version-{}-a", asset_id.trim_start_matches("asset-"))
                        })
                        .collect()
                } else {
                    vec![]
                },
                error: if can_submit {
                    None
                } else {
                    Some(blocking_reasons.join(" "))
                },
            })
            .collect();

        Self {
            can_submit,
            jobs,
            recipes,
            blocking_reasons,
            warnings,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SampleSavedOutput {
    pub variant_id: String,
    pub asset: AudioAsset,
    pub version: AudioAssetVersion,
    pub storage: StoragePaths,
    pub exported: bool,
    pub waveform_preview_ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SamplePostProcessingAction {
    pub id: String,
    pub operation: PostProcessingOperation,
    pub enabled: bool,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SampleQaCheck {
    pub id: String,
    pub status: SampleQaStatus,
    pub summary: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SampleQaStatus {
    Passed,
    Warning,
    Failed,
}

fn provider_options(
    catalog: &ProviderCatalog,
    runtime: &RuntimeOverview,
) -> Vec<SampleProviderOption> {
    let mut options = vec![];

    for provider in &catalog.providers {
        for model in &provider.models {
            for capability in model.capabilities.iter().filter(|capability| {
                matches!(
                    capability.workflow,
                    CapabilityWorkflow::InstrumentSample | CapabilityWorkflow::Loop
                ) && capability.inputs.contains(&CapabilityInput::TextPrompt)
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
                limitations.extend(limitations_for_safety(
                    &capability.safety,
                    SafetyLimitationOptions {
                        check_voice_consent: false,
                        check_commercial_use: true,
                        check_provenance_sidecar: false,
                    },
                ));

                let output_asset_kind = capability
                    .outputs
                    .asset_kinds
                    .first()
                    .copied()
                    .unwrap_or(AudioAssetKind::InstrumentSample);

                options.push(SampleProviderOption {
                    provider_id: provider.id.clone(),
                    model_id: model.id.clone(),
                    model_version: model.version.clone(),
                    display_name: format!("{} / {}", provider.name, model.name),
                    workflow: capability.workflow,
                    runtime: model.runtime,
                    install_status: kebab_label(&model.install.status),
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
                    supports_tempo: capability.inputs.contains(&CapabilityInput::Tempo),
                    supports_key: capability.inputs.contains(&CapabilityInput::MusicalKey),
                    supports_loop_points: matches!(capability.workflow, CapabilityWorkflow::Loop),
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

fn supported_controls(capability: &crate::manifests::ModelCapability) -> Vec<SampleControlKind> {
    let mut controls = vec![
        SampleControlKind::Prompt,
        SampleControlKind::InstrumentFamily,
        SampleControlKind::Articulation,
        SampleControlKind::DryWetAmbience,
        SampleControlKind::VelocityEnergy,
        SampleControlKind::BatchGeneration,
    ];

    if capability.inputs.contains(&CapabilityInput::MusicalKey) {
        controls.push(SampleControlKind::MusicalKey);
        controls.push(SampleControlKind::Scale);
    }

    if capability.inputs.contains(&CapabilityInput::Tempo) {
        controls.push(SampleControlKind::Tempo);
        controls.push(SampleControlKind::BarsBeats);
    }

    if matches!(capability.workflow, CapabilityWorkflow::Loop) {
        controls.push(SampleControlKind::Loopable);
    }

    if capability.inputs.contains(&CapabilityInput::ReferenceAudio) {
        controls.push(SampleControlKind::ReferenceAudio);
    }

    controls
}

fn provider_scorecards(evaluation: &ModelEvaluationCatalog) -> Vec<SampleProviderScorecard> {
    let recommended_ids = evaluation
        .recommendations
        .iter()
        .map(|recommendation| recommendation.candidate_id.as_str())
        .collect::<Vec<_>>();

    [
        "stable-audio-3",
        "ace-step-1-5",
        "heartmula",
        "muse-song",
        "stable-audio-open-1",
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

fn scorecard(candidate: &ModelEvaluationCandidate, recommended: bool) -> SampleProviderScorecard {
    let sample_focused = candidate.lanes.iter().any(|lane| {
        matches!(
            lane,
            EvaluationLane::InstrumentSample | EvaluationLane::Loop
        )
    });
    let readiness = if !sample_focused {
        SampleProviderReadiness::NotSampleFocused
    } else {
        match candidate.product_eligibility {
            ProductEligibility::ProductCandidate | ProductEligibility::ApiOnlyCandidate => {
                SampleProviderReadiness::Ready
            }
            ProductEligibility::NeedsRuntimePort => SampleProviderReadiness::NeedsRuntimePort,
            ProductEligibility::ResearchOnly => SampleProviderReadiness::ResearchOnly,
            ProductEligibility::Blocked => SampleProviderReadiness::Blocked,
        }
    };
    let mut blockers = candidate.blockers.clone();

    if !sample_focused {
        blockers.push(
            "Candidate is tracked for adjacent audio lanes, not as a primary sample/loop provider."
                .to_string(),
        );
    }

    SampleProviderScorecard {
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

fn variant_previews(prompt: &SamplePrompt, controls: &SampleControls) -> Vec<SampleVariantPreview> {
    let base_tags = prompt.genre_tags.clone();

    vec![
        SampleVariantPreview {
            id: "sample-variant-bass-pluck-a".to_string(),
            label: "Bass pluck A1".to_string(),
            workflow: CapabilityWorkflow::InstrumentSample,
            asset_kind: AudioAssetKind::InstrumentSample,
            instrument_family: prompt.instrument_family,
            articulation: prompt.articulation.clone(),
            duration_ms: 900,
            bpm: None,
            musical_key: Some("A1".to_string()),
            time_signature: None,
            loop_points: None,
            transient_one_shot: true,
            loudness_lufs: -18.0,
            true_peak_dbfs: -1.4,
            has_clipping: false,
            tags: tags(&base_tags, &["one-shot", "pluck"]),
            collection_id: "collection-neon-bass-pack".to_string(),
            selected_for_pack: true,
            favorite: true,
            duplicate_of_variant_id: None,
        },
        SampleVariantPreview {
            id: "sample-variant-bass-stab-c2".to_string(),
            label: "Bass stab C2".to_string(),
            workflow: CapabilityWorkflow::InstrumentSample,
            asset_kind: AudioAssetKind::InstrumentSample,
            instrument_family: prompt.instrument_family,
            articulation: "short stab".to_string(),
            duration_ms: 650,
            bpm: None,
            musical_key: Some("C2".to_string()),
            time_signature: None,
            loop_points: None,
            transient_one_shot: true,
            loudness_lufs: -17.2,
            true_peak_dbfs: -1.1,
            has_clipping: false,
            tags: tags(&base_tags, &["one-shot", "stab"]),
            collection_id: "collection-neon-bass-pack".to_string(),
            selected_for_pack: true,
            favorite: false,
            duplicate_of_variant_id: None,
        },
        SampleVariantPreview {
            id: "loop-variant-bassline-120a".to_string(),
            label: "Four-bar chase bassline".to_string(),
            workflow: CapabilityWorkflow::Loop,
            asset_kind: AudioAssetKind::Loop,
            instrument_family: prompt.instrument_family,
            articulation: "pulsed sequence".to_string(),
            duration_ms: loop_duration_ms(controls),
            bpm: Some(controls.bpm),
            musical_key: Some(controls.musical_key.clone()),
            time_signature: Some(format!("{}/4", controls.beats)),
            loop_points: Some(LoopPoints {
                start_sample: 0,
                end_sample: 352_800,
            }),
            transient_one_shot: false,
            loudness_lufs: -19.0,
            true_peak_dbfs: -2.0,
            has_clipping: false,
            tags: tags(&base_tags, &["loop", "four-bar"]),
            collection_id: "collection-neon-bass-pack".to_string(),
            selected_for_pack: true,
            favorite: true,
            duplicate_of_variant_id: None,
        },
        SampleVariantPreview {
            id: "loop-variant-bassline-drier".to_string(),
            label: "Dry alternate bassline".to_string(),
            workflow: CapabilityWorkflow::Loop,
            asset_kind: AudioAssetKind::Loop,
            instrument_family: prompt.instrument_family,
            articulation: "dry pulsed sequence".to_string(),
            duration_ms: loop_duration_ms(controls),
            bpm: Some(controls.bpm),
            musical_key: Some(controls.musical_key.clone()),
            time_signature: Some(format!("{}/4", controls.beats)),
            loop_points: Some(LoopPoints {
                start_sample: 0,
                end_sample: 352_800,
            }),
            transient_one_shot: false,
            loudness_lufs: -20.0,
            true_peak_dbfs: -2.8,
            has_clipping: false,
            tags: tags(&base_tags, &["loop", "dry"]),
            collection_id: "collection-neon-bass-pack".to_string(),
            selected_for_pack: false,
            favorite: false,
            duplicate_of_variant_id: Some("loop-variant-bassline-120a".to_string()),
        },
    ]
}

fn loop_duration_ms(controls: &SampleControls) -> u64 {
    let beats = controls.bars as f32 * controls.beats as f32;
    ((60_000.0 / controls.bpm) * beats).round() as u64
}

fn tags(base: &[String], extra: &[&str]) -> Vec<String> {
    let mut tags = base.to_vec();
    tags.extend(extra.iter().map(|tag| (*tag).to_string()));
    tags.sort();
    tags.dedup();
    tags
}

fn generation_recipes(
    prompt: &SamplePrompt,
    controls: &SampleControls,
    provider: &SampleProviderSelection,
    variants: &[SampleVariantPreview],
) -> Vec<GenerationRecipe> {
    let sample_output_ids = variants
        .iter()
        .filter(|variant| {
            variant.selected_for_pack && variant.asset_kind == AudioAssetKind::InstrumentSample
        })
        .map(|variant| format!("asset-{}", variant.id))
        .collect::<Vec<_>>();
    let loop_output_ids = variants
        .iter()
        .filter(|variant| variant.selected_for_pack && variant.asset_kind == AudioAssetKind::Loop)
        .map(|variant| format!("asset-{}", variant.id))
        .collect::<Vec<_>>();

    vec![
        GenerationRecipe {
            id: "recipe-samples-one-shots-reference".to_string(),
            workflow: RecipeWorkflow::InstrumentSample,
            provider: provider.descriptor(),
            request: RecipeRequest::InstrumentSample(InstrumentSampleRecipe {
                prompt: prompt.text.clone(),
                instrument: Some(kebab_label(&prompt.instrument_family)),
                pitch: Some(controls.musical_key.clone()),
                velocity: Some(controls.velocity_energy),
                target_duration_ms: Some(900),
            }),
            seed: Some(61_540),
            source_references: source_references(prompt),
            post_processing: post_processing_steps(controls),
            parameter_overrides: sample_parameters(prompt, controls),
            output_asset_ids: sample_output_ids,
        },
        GenerationRecipe {
            id: "recipe-loops-four-bar-reference".to_string(),
            workflow: RecipeWorkflow::Loop,
            provider: provider.descriptor(),
            request: RecipeRequest::Loop(LoopRecipe {
                prompt: prompt.text.clone(),
                bpm: controls.bpm,
                musical_key: Some(controls.musical_key.clone()),
                bars: controls.bars,
                loopable: controls.loopable,
            }),
            seed: Some(61_541),
            source_references: source_references(prompt),
            post_processing: post_processing_steps(controls),
            parameter_overrides: sample_parameters(prompt, controls),
            output_asset_ids: loop_output_ids,
        },
    ]
}

fn source_references(prompt: &SamplePrompt) -> Vec<SourceReference> {
    prompt
        .reference_audio_asset_id
        .iter()
        .map(|asset_id| SourceReference {
            id: "source-sample-reference-audio".to_string(),
            source_type: SourceReferenceType::Audio,
            asset_id: Some(asset_id.clone()),
            external_uri: None,
            ownership_note: Some("User-owned groove/reference texture.".to_string()),
        })
        .collect()
}

fn sample_parameters(
    prompt: &SamplePrompt,
    controls: &SampleControls,
) -> BTreeMap<String, serde_json::Value> {
    let mut parameters = BTreeMap::new();
    parameters.insert(
        "instrumentFamily".to_string(),
        json!(prompt.instrument_family),
    );
    parameters.insert("articulation".to_string(), json!(prompt.articulation));
    parameters.insert("genreTags".to_string(), json!(prompt.genre_tags));
    parameters.insert("musicalKey".to_string(), json!(controls.musical_key));
    parameters.insert("scale".to_string(), json!(controls.scale));
    parameters.insert("bpm".to_string(), json!(controls.bpm));
    parameters.insert("bars".to_string(), json!(controls.bars));
    parameters.insert("beats".to_string(), json!(controls.beats));
    parameters.insert("loopable".to_string(), json!(controls.loopable));
    parameters.insert(
        "dryWetAmbience".to_string(),
        json!(controls.dry_wet_ambience),
    );
    parameters.insert(
        "velocityEnergy".to_string(),
        json!(controls.velocity_energy),
    );
    parameters.insert("batchSize".to_string(), json!(controls.batch_size));
    parameters
}

fn post_processing_steps(controls: &SampleControls) -> Vec<PostProcessingStep> {
    let mut normalize = BTreeMap::new();
    normalize.insert("targetLoudnessLufs".to_string(), json!(-18.0));

    let mut loop_check = BTreeMap::new();
    loop_check.insert("bpm".to_string(), json!(controls.bpm));
    loop_check.insert("bars".to_string(), json!(controls.bars));

    vec![
        PostProcessingStep {
            id: "post-normalize-sample-pack".to_string(),
            operation: PostProcessingOperation::Normalize,
            parameters: normalize,
        },
        PostProcessingStep {
            id: "post-trim-sample-head-tail".to_string(),
            operation: PostProcessingOperation::Trim,
            parameters: BTreeMap::new(),
        },
        PostProcessingStep {
            id: "post-loop-seam-check".to_string(),
            operation: PostProcessingOperation::Fade,
            parameters: loop_check,
        },
    ]
}

fn saved_outputs(
    submission: &SampleSubmissionPreview,
    variants: &[SampleVariantPreview],
    allocator: &StoragePathAllocator,
) -> Result<Vec<SampleSavedOutput>, StoragePathError> {
    let scope = LibraryScope::Project {
        project_id: "project-demo".to_string(),
    };

    variants
        .iter()
        .filter(|variant| variant.selected_for_pack)
        .map(|variant| saved_output(submission, variant, &scope, allocator))
        .collect()
}

fn saved_output(
    submission: &SampleSubmissionPreview,
    variant: &SampleVariantPreview,
    scope: &LibraryScope,
    allocator: &StoragePathAllocator,
) -> Result<SampleSavedOutput, StoragePathError> {
    let asset_id = format!("asset-{}", variant.id);
    let version_id = format!("version-{}-a", variant.id);
    let storage = allocator.allocate_asset_version(
        scope,
        variant.asset_kind,
        &asset_id,
        &version_id,
        AudioFileFormat::Wav,
    )?;
    let recipe_id = submission
        .recipes
        .iter()
        .find(|recipe| recipe.output_asset_ids.contains(&asset_id))
        .map(|recipe| recipe.id.clone())
        .unwrap_or_else(|| "recipe-samples-unassigned".to_string());
    let job_id = submission
        .jobs
        .iter()
        .find(|job| job.recipe_id == recipe_id)
        .map(|job| job.id.clone())
        .unwrap_or_else(|| "job-samples-unassigned".to_string());

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
            channels: if variant.asset_kind == AudioAssetKind::Loop {
                2
            } else {
                1
            },
            duration_ms: variant.duration_ms,
            loudness_lufs: Some(variant.loudness_lufs),
            true_peak_dbfs: Some(variant.true_peak_dbfs),
            has_clipping: variant.has_clipping,
            bpm: variant.bpm,
            musical_key: variant.musical_key.clone(),
            loop_points: variant.loop_points,
        },
        created_by: AssetCreation::Generated { recipe_id, job_id },
        waveform_preview_cache: Some(storage.waveform_preview_path.clone()),
        spectrogram_preview_cache: Some(storage.spectrogram_preview_path.clone()),
    };

    let asset = AudioAsset {
        id: asset_id,
        scope: scope.clone(),
        kind: variant.asset_kind,
        name: variant.label.clone(),
        tags: variant.tags.clone(),
        collection_ids: vec![variant.collection_id.clone()],
        current_version_id: version.id.clone(),
        version_ids: vec![version.id.clone()],
        rights: RightsMetadata {
            license_status: LicenseStatus::ProviderLicensed,
            commercial_use: CommercialUseStatus::Allowed,
            voice_consent: VoiceConsentStatus::NotVoiceMaterial,
            ai_disclosure_required: true,
            watermark: WatermarkStatus::SidecarOnly,
            reference_media_ownership: Some(
                "generated from user-owned reference groove".to_string(),
            ),
        },
        provenance_ids: vec![format!("provenance-{}", variant.id)],
    };

    Ok(SampleSavedOutput {
        variant_id: variant.id.clone(),
        asset,
        version,
        storage,
        exported: true,
        waveform_preview_ready: true,
    })
}

fn post_processing_actions() -> Vec<SamplePostProcessingAction> {
    vec![
        SamplePostProcessingAction {
            id: "trim-silence".to_string(),
            operation: PostProcessingOperation::Trim,
            enabled: true,
            summary: "Trim one-shot heads/tails without damaging transients.".to_string(),
        },
        SamplePostProcessingAction {
            id: "normalize".to_string(),
            operation: PostProcessingOperation::Normalize,
            enabled: true,
            summary: "Normalize samples and loops for audition/export consistency.".to_string(),
        },
        SamplePostProcessingAction {
            id: "loop-seam".to_string(),
            operation: PostProcessingOperation::Fade,
            enabled: true,
            summary: "Check loop seam and apply short crossfade when required.".to_string(),
        },
        SamplePostProcessingAction {
            id: "pack-export".to_string(),
            operation: PostProcessingOperation::ConvertFormat,
            enabled: true,
            summary: "Export sample-pack WAV/FLAC files with BPM/key/provenance sidecars."
                .to_string(),
        },
    ]
}

fn qa_checks() -> Vec<SampleQaCheck> {
    vec![
        SampleQaCheck {
            id: "samples.provider_capabilities".to_string(),
            status: SampleQaStatus::Passed,
            summary:
                "Instrument, tempo, key, loop, and batch controls come from provider capabilities."
                    .to_string(),
        },
        SampleQaCheck {
            id: "samples.isolation".to_string(),
            status: SampleQaStatus::Passed,
            summary: "One-shot variants track transient/sample isolation metadata.".to_string(),
        },
        SampleQaCheck {
            id: "samples.loop_seam".to_string(),
            status: SampleQaStatus::Passed,
            summary: "Loop variants include BPM, key, bar count, and inspectable loop points."
                .to_string(),
        },
        SampleQaCheck {
            id: "samples.audio_quality".to_string(),
            status: SampleQaStatus::Passed,
            summary: "Clipping, silence, loudness, and duration mismatch checks are represented."
                .to_string(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::{
        SampleControlKind, SampleProviderReadiness, SampleQaStatus, SamplesStudioOverview,
    };
    use crate::domain::{AudioAssetKind, RecipeRequest, RecipeWorkflow};
    use crate::evaluation::EvaluationLane;
    use crate::manifests::CapabilityWorkflow;

    #[test]
    fn reference_studio_exposes_sample_and_loop_controls() {
        let overview = SamplesStudioOverview::reference().expect("samples studio builds");

        assert_eq!(overview.schema_version, 1);
        assert_eq!(overview.controls.bpm, 120.0);
        assert!(overview
            .provider_options
            .iter()
            .any(
                |option| option.workflow == CapabilityWorkflow::InstrumentSample
                    && option
                        .supported_controls
                        .contains(&SampleControlKind::MusicalKey)
            ));
        assert!(overview
            .provider_options
            .iter()
            .any(|option| option.workflow == CapabilityWorkflow::Loop
                && option
                    .supported_controls
                    .contains(&SampleControlKind::Tempo)
                && option
                    .supported_controls
                    .contains(&SampleControlKind::Loopable)));
        assert!(!overview.submission.can_submit);
        assert!(overview
            .submission
            .blocking_reasons
            .iter()
            .any(|reason| reason.contains("verified cache/package evidence")));
    }

    #[test]
    fn provider_scorecards_focus_on_sample_loop_candidates() {
        let overview = SamplesStudioOverview::reference().expect("samples studio builds");
        let ace = overview
            .provider_scorecards
            .iter()
            .find(|scorecard| scorecard.candidate_id == "ace-step-1-5")
            .expect("ACE-Step scorecard exists");
        let stable_open = overview
            .provider_scorecards
            .iter()
            .find(|scorecard| scorecard.candidate_id == "stable-audio-open-1")
            .expect("Stable Audio Open scorecard exists");

        assert!(ace.recommended);
        assert!(ace.lanes.contains(&EvaluationLane::Loop));
        assert_eq!(ace.readiness, SampleProviderReadiness::NeedsRuntimePort);
        assert_eq!(
            stable_open.readiness,
            SampleProviderReadiness::NeedsRuntimePort
        );
    }

    #[test]
    fn submission_preserves_separate_sample_and_loop_recipes() {
        let overview = SamplesStudioOverview::reference().expect("samples studio builds");

        assert_eq!(overview.submission.recipes.len(), 2);
        assert!(overview
            .submission
            .recipes
            .iter()
            .any(|recipe| recipe.workflow == RecipeWorkflow::InstrumentSample));
        assert!(overview
            .submission
            .recipes
            .iter()
            .any(|recipe| recipe.workflow == RecipeWorkflow::Loop));

        let RecipeRequest::Loop(loop_recipe) = &overview
            .submission
            .recipes
            .iter()
            .find(|recipe| recipe.workflow == RecipeWorkflow::Loop)
            .expect("loop recipe")
            .request
        else {
            panic!("expected loop request");
        };

        assert_eq!(loop_recipe.bpm, 120.0);
        assert_eq!(loop_recipe.bars, 4);
        assert!(loop_recipe.loopable);
    }

    #[test]
    fn saved_outputs_include_bpm_key_collections_and_loop_points() {
        let overview = SamplesStudioOverview::reference().expect("samples studio builds");

        assert_eq!(overview.saved_outputs.len(), 3);
        assert!(overview.saved_outputs.iter().any(|output| output.asset.kind
            == AudioAssetKind::InstrumentSample
            && output
                .asset
                .collection_ids
                .contains(&overview.pack.collection_id)
            && output.version.technical.musical_key.is_some()
            && output.exported));
        assert!(overview
            .saved_outputs
            .iter()
            .any(|output| output.asset.kind == AudioAssetKind::Loop
                && output.version.technical.bpm == Some(120.0)
                && output.version.technical.loop_points.is_some()));
        assert!(overview
            .qa_checks
            .iter()
            .all(|check| check.status == SampleQaStatus::Passed));
    }
}
