use crate::domain::{
    AssetCreation, AudioAsset, AudioAssetKind, AudioAssetVersion, AudioFileFormat,
    AudioFileReference, CommercialUseStatus, GenerationJob, GenerationRecipe, JobKind, JobProgress,
    JobStatus, LibraryScope, LicenseStatus, ModelDescriptor, ModelRuntime, PostProcessingOperation,
    PostProcessingStep, RecipeRequest, RecipeWorkflow, RightsMetadata, SongRecipe, SongSection,
    SongStructure, SourceReference, SourceReferenceType, StemKind, TechnicalAudioMetadata,
    VoiceConsentStatus, WatermarkStatus,
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

pub const SONG_STUDIO_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SongStudioOverview {
    pub schema_version: u32,
    pub draft: SongDraft,
    pub controls: SongControls,
    pub provider_options: Vec<SongProviderOption>,
    pub selected_provider: SongProviderSelection,
    pub provider_scorecards: Vec<SongProviderScorecard>,
    pub arrangement: SongArrangementPreview,
    pub variants: Vec<SongVariantPreview>,
    pub submission: SongSubmissionPreview,
    pub saved_outputs: Vec<SongSavedOutput>,
    pub export_targets: Vec<SongExportTarget>,
    pub qa_checks: Vec<SongQaCheck>,
}

impl SongStudioOverview {
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
        let draft = SongDraft::reference();
        let controls = SongControls::reference();
        let provider_options = provider_options(catalog, runtime);
        let selected_provider = provider_options
            .iter()
            .find(|option| option.runnable && option.workflow == CapabilityWorkflow::Song)
            .map(SongProviderSelection::from_option)
            .unwrap_or_else(|| SongProviderSelection {
                provider_id: "unavailable".to_string(),
                model_id: "unavailable".to_string(),
                model_version: None,
                workflow: CapabilityWorkflow::Song,
                runtime: ModelRuntime::ResearchOnly,
                accepted: false,
                blocker: Some("No runnable complete-song provider is registered.".to_string()),
            });
        let provider_scorecards = provider_scorecards(evaluation);
        let arrangement = SongArrangementPreview::from_draft(&draft, &controls);
        let variants = variant_previews(&draft, &controls);
        let submission = SongSubmissionPreview::build(
            &draft,
            &controls,
            &provider_options,
            &selected_provider,
            &variants,
        );
        let saved_outputs = saved_outputs(&submission, &variants, allocator)?;

        Ok(Self {
            schema_version: SONG_STUDIO_SCHEMA_VERSION,
            draft,
            controls,
            provider_options,
            selected_provider,
            provider_scorecards,
            arrangement,
            variants,
            submission,
            saved_outputs,
            export_targets: export_targets(),
            qa_checks: qa_checks(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SongDraft {
    pub id: String,
    pub title: String,
    pub prompt: String,
    pub lyrics: String,
    pub style_tags: Vec<String>,
    pub language: String,
    pub vocalist: SongVocalMode,
    pub singer_hint: Option<String>,
    pub reference_audio_asset_ids: Vec<String>,
    pub sections: Vec<SongSectionDraft>,
}

impl SongDraft {
    fn reference() -> Self {
        Self {
            id: "song-draft-city-lights".to_string(),
            title: "City Lights Resolve".to_string(),
            prompt:
                "Cinematic synth-pop song with a confident female lead, warm analog pads, tight electronic drums, and a final lift for the chorus."
                    .to_string(),
            lyrics: "Verse:\nStreetlights hum under rain on glass\nI keep the tempo of a moving train\n\nChorus:\nCity lights, carry me home\nTurn the static into gold"
                .to_string(),
            style_tags: vec![
                "synth-pop".to_string(),
                "cinematic".to_string(),
                "female-vocal".to_string(),
                "120-bpm".to_string(),
            ],
            language: "en-US".to_string(),
            vocalist: SongVocalMode::Vocal,
            singer_hint: Some("clear alto lead with restrained vibrato".to_string()),
            reference_audio_asset_ids: vec![
                "asset-reference-synth-pad".to_string(),
                "asset-reference-drum-groove".to_string(),
            ],
            sections: vec![
                SongSectionDraft::new("intro", "Intro", 8, None),
                SongSectionDraft::new(
                    "verse-1",
                    "Verse 1",
                    16,
                    Some("Streetlights hum under rain on glass\nI keep the tempo of a moving train"),
                ),
                SongSectionDraft::new(
                    "chorus-1",
                    "Chorus 1",
                    16,
                    Some("City lights, carry me home\nTurn the static into gold"),
                ),
                SongSectionDraft::new("outro", "Outro", 8, None),
            ],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SongVocalMode {
    Vocal,
    Instrumental,
    Both,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SongSectionDraft {
    pub id: String,
    pub label: String,
    pub bars: u16,
    pub lyrics: Option<String>,
    pub regenerate_locked: bool,
}

impl SongSectionDraft {
    fn new(id: &str, label: &str, bars: u16, lyrics: Option<&str>) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            bars,
            lyrics: lyrics.map(str::to_string),
            regenerate_locked: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SongControls {
    pub bpm: f32,
    pub musical_key: String,
    pub time_signature: String,
    pub target_duration_ms: u64,
    pub section_length_bars: u16,
    pub variation_count: u8,
    pub generate_stems: bool,
    pub requested_stems: Vec<StemKind>,
    pub allow_reference_audio: bool,
    pub promote_to_project_library: bool,
}

impl SongControls {
    fn reference() -> Self {
        Self {
            bpm: 120.0,
            musical_key: "A minor".to_string(),
            time_signature: "4/4".to_string(),
            target_duration_ms: 96_000,
            section_length_bars: 16,
            variation_count: 2,
            generate_stems: true,
            requested_stems: vec![
                StemKind::FullMix,
                StemKind::Vocals,
                StemKind::Drums,
                StemKind::Bass,
                StemKind::Instruments,
            ],
            allow_reference_audio: true,
            promote_to_project_library: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SongProviderOption {
    pub provider_id: String,
    pub model_id: String,
    pub model_version: Option<String>,
    pub display_name: String,
    pub workflow: CapabilityWorkflow,
    pub runtime: ModelRuntime,
    pub install_status: String,
    pub runnable: bool,
    pub output_asset_kinds: Vec<AudioAssetKind>,
    pub output_format: AudioFileFormat,
    pub sample_rate_hz: u32,
    pub channel_layout: ChannelLayout,
    pub min_duration_ms: Option<u64>,
    pub max_duration_ms: Option<u64>,
    pub supports_lyrics: bool,
    pub supports_style_tags: bool,
    pub supports_reference_audio: bool,
    pub supports_stems: bool,
    pub supported_stems: Vec<StemKind>,
    pub commercial_use_allowed: bool,
    pub watermark: WatermarkStatus,
    pub supported_controls: Vec<SongControlKind>,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SongControlKind {
    Prompt,
    Lyrics,
    SectionStructure,
    StyleTags,
    VocalMode,
    SingerHint,
    Language,
    Duration,
    Tempo,
    MusicalKey,
    ReferenceAudio,
    Stems,
    Variants,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SongProviderSelection {
    pub provider_id: String,
    pub model_id: String,
    pub model_version: Option<String>,
    pub workflow: CapabilityWorkflow,
    pub runtime: ModelRuntime,
    pub accepted: bool,
    pub blocker: Option<String>,
}

impl SongProviderSelection {
    fn from_option(option: &SongProviderOption) -> Self {
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
pub struct SongProviderScorecard {
    pub candidate_id: String,
    pub name: String,
    pub provider: String,
    pub lanes: Vec<EvaluationLane>,
    pub status: EvaluationStatus,
    pub product_eligibility: ProductEligibility,
    pub readiness: SongProviderReadiness,
    pub runtime_path: ProductRuntimePath,
    pub commercial_use: CommercialUseEvaluation,
    pub recommended: bool,
    pub blockers: Vec<String>,
    pub notes: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SongProviderReadiness {
    Ready,
    NeedsRuntimePort,
    ResearchOnly,
    Blocked,
    NotSongFocused,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SongArrangementPreview {
    pub section_count: usize,
    pub total_bars: u16,
    pub estimated_duration_ms: u64,
    pub sections: Vec<SongArrangementSection>,
}

impl SongArrangementPreview {
    fn from_draft(draft: &SongDraft, controls: &SongControls) -> Self {
        let sections = draft
            .sections
            .iter()
            .scan(0_u16, |start_bar, section| {
                let preview = SongArrangementSection {
                    id: section.id.clone(),
                    label: section.label.clone(),
                    start_bar: *start_bar,
                    bars: section.bars,
                    has_lyrics: section.lyrics.is_some(),
                    locked: section.regenerate_locked,
                };
                *start_bar += section.bars;
                Some(preview)
            })
            .collect::<Vec<_>>();
        let total_bars = sections.iter().map(|section| section.bars).sum::<u16>();

        Self {
            section_count: sections.len(),
            total_bars,
            estimated_duration_ms: bars_to_duration_ms(total_bars, controls.bpm),
            sections,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SongArrangementSection {
    pub id: String,
    pub label: String,
    pub start_bar: u16,
    pub bars: u16,
    pub has_lyrics: bool,
    pub locked: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SongVariantPreview {
    pub id: String,
    pub label: String,
    pub asset_kind: AudioAssetKind,
    pub duration_ms: u64,
    pub bpm: f32,
    pub musical_key: String,
    pub vocal_mode: SongVocalMode,
    pub stem_kinds: Vec<StemKind>,
    pub loudness_lufs: f32,
    pub true_peak_dbfs: f32,
    pub lyric_alignment_score: u8,
    pub structure_match_score: u8,
    pub tags: Vec<String>,
    pub selected_for_save: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SongSubmissionPreview {
    pub can_submit: bool,
    pub job: GenerationJob,
    pub recipe: GenerationRecipe,
    pub blocking_reasons: Vec<String>,
    pub warnings: Vec<String>,
}

impl SongSubmissionPreview {
    fn build(
        draft: &SongDraft,
        controls: &SongControls,
        provider_options: &[SongProviderOption],
        selected_provider: &SongProviderSelection,
        variants: &[SongVariantPreview],
    ) -> Self {
        let mut blocking_reasons = vec![];
        let mut warnings = vec![];
        let selected_option = provider_options.iter().find(|option| {
            option.provider_id == selected_provider.provider_id
                && option.model_id == selected_provider.model_id
                && option.workflow == selected_provider.workflow
        });

        if draft.prompt.trim().is_empty() && draft.lyrics.trim().is_empty() {
            blocking_reasons.push("Song prompt and lyrics cannot both be empty.".to_string());
        }

        if draft.sections.is_empty() {
            blocking_reasons.push("Song structure must include at least one section.".to_string());
        }

        if selected_option.is_none() || !selected_provider.accepted {
            blocking_reasons.push(selected_provider.blocker.clone().unwrap_or_else(|| {
                "Selected provider cannot currently accept complete-song jobs.".to_string()
            }));
        }

        if let Some(option) = selected_option {
            if !option.supports_lyrics && !draft.lyrics.trim().is_empty() {
                warnings.push(
                    "Lyrics are preserved in the recipe, but the provider may only receive them as prompt context."
                        .to_string(),
                );
            }

            if controls.generate_stems && !option.supports_stems {
                warnings.push(
                    "Stem requests will be queued as post-generation separation because the provider does not expose native stems."
                        .to_string(),
                );
            }

            if controls.allow_reference_audio
                && !draft.reference_audio_asset_ids.is_empty()
                && !option.supports_reference_audio
            {
                warnings.push(
                    "Reference audio is stored with provenance but unavailable as a provider input."
                        .to_string(),
                );
            }

            if !option.commercial_use_allowed {
                warnings.push(
                    "Selected provider requires commercial-use review before export.".to_string(),
                );
            }
        }

        if controls.target_duration_ms > 360_000 {
            warnings
                .push("Long songs require runtime memory and cancellation validation.".to_string());
        }

        let can_submit = blocking_reasons.is_empty();
        let recipe = generation_recipe(draft, controls, selected_provider, variants);
        let job = GenerationJob {
            id: "job-song-studio-reference".to_string(),
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
                    "Ready to queue complete-song generation.".to_string()
                } else {
                    "Song submission blocked by validation.".to_string()
                }),
            }),
            output_version_ids: if can_submit {
                recipe
                    .output_asset_ids
                    .iter()
                    .map(|asset_id| format!("version-{}-a", asset_id.trim_start_matches("asset-")))
                    .collect()
            } else {
                vec![]
            },
            error: if can_submit {
                None
            } else {
                Some(blocking_reasons.join(" "))
            },
        };

        Self {
            can_submit,
            job,
            recipe,
            blocking_reasons,
            warnings,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SongSavedOutput {
    pub variant_id: String,
    pub asset: AudioAsset,
    pub version: AudioAssetVersion,
    pub storage: StoragePaths,
    pub export_ready: bool,
    pub waveform_preview_ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SongExportTarget {
    pub id: String,
    pub label: String,
    pub formats: Vec<AudioFileFormat>,
    pub includes_stems: bool,
    pub includes_sidecar: bool,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SongQaCheck {
    pub id: String,
    pub status: SongQaStatus,
    pub summary: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SongQaStatus {
    Passed,
    Warning,
    Failed,
}

fn provider_options(
    catalog: &ProviderCatalog,
    runtime: &RuntimeOverview,
) -> Vec<SongProviderOption> {
    let mut options = vec![];

    for provider in &catalog.providers {
        for model in &provider.models {
            for capability in model.capabilities.iter().filter(|capability| {
                capability.workflow == CapabilityWorkflow::Song
                    && capability.inputs.contains(&CapabilityInput::TextPrompt)
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

                options.push(SongProviderOption {
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
                    output_asset_kinds: capability.outputs.asset_kinds.clone(),
                    output_format: capability.defaults.format,
                    sample_rate_hz: capability.defaults.sample_rate_hz,
                    channel_layout: capability.defaults.channel_layout,
                    min_duration_ms: capability.limits.min_duration_ms,
                    max_duration_ms: capability.limits.max_duration_ms,
                    supports_lyrics: capability.inputs.contains(&CapabilityInput::Lyrics),
                    supports_style_tags: capability.inputs.contains(&CapabilityInput::StyleTags),
                    supports_reference_audio: capability
                        .inputs
                        .contains(&CapabilityInput::ReferenceAudio),
                    supports_stems: capability.inputs.contains(&CapabilityInput::Stems)
                        || !capability.outputs.stems.is_empty(),
                    supported_stems: capability.outputs.stems.clone(),
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

fn supported_controls(capability: &crate::manifests::ModelCapability) -> Vec<SongControlKind> {
    let mut controls = vec![
        SongControlKind::Prompt,
        SongControlKind::SectionStructure,
        SongControlKind::VocalMode,
        SongControlKind::SingerHint,
        SongControlKind::Duration,
        SongControlKind::Variants,
    ];

    if capability.inputs.contains(&CapabilityInput::Lyrics) {
        controls.push(SongControlKind::Lyrics);
    }

    if capability.inputs.contains(&CapabilityInput::StyleTags) {
        controls.push(SongControlKind::StyleTags);
    }

    if !capability.limits.supported_languages.is_empty() {
        controls.push(SongControlKind::Language);
    }

    if capability.inputs.contains(&CapabilityInput::Tempo) {
        controls.push(SongControlKind::Tempo);
    }

    if capability.inputs.contains(&CapabilityInput::MusicalKey) {
        controls.push(SongControlKind::MusicalKey);
    }

    if capability.inputs.contains(&CapabilityInput::ReferenceAudio) {
        controls.push(SongControlKind::ReferenceAudio);
    }

    if capability.inputs.contains(&CapabilityInput::Stems) || !capability.outputs.stems.is_empty() {
        controls.push(SongControlKind::Stems);
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

    if !safety.provenance_sidecar {
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

fn provider_scorecards(evaluation: &ModelEvaluationCatalog) -> Vec<SongProviderScorecard> {
    let recommended_ids = evaluation
        .recommendations
        .iter()
        .map(|recommendation| recommendation.candidate_id.as_str())
        .collect::<Vec<_>>();

    [
        "stable-audio-3",
        "ace-step-1-5",
        "levo-2",
        "yue",
        "diffrhythm-2",
        "khala",
        "heartmula",
        "muse-song",
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

fn scorecard(candidate: &ModelEvaluationCandidate, recommended: bool) -> SongProviderScorecard {
    let song_focused = candidate.lanes.contains(&EvaluationLane::Song);
    let readiness = if !song_focused {
        SongProviderReadiness::NotSongFocused
    } else {
        match candidate.product_eligibility {
            ProductEligibility::ProductCandidate | ProductEligibility::ApiOnlyCandidate => {
                SongProviderReadiness::Ready
            }
            ProductEligibility::NeedsRuntimePort => SongProviderReadiness::NeedsRuntimePort,
            ProductEligibility::ResearchOnly => SongProviderReadiness::ResearchOnly,
            ProductEligibility::Blocked => SongProviderReadiness::Blocked,
        }
    };
    let mut blockers = candidate.blockers.clone();

    if !song_focused {
        blockers.push("Candidate is not tracked as a complete-song provider.".to_string());
    }

    SongProviderScorecard {
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

fn variant_previews(draft: &SongDraft, controls: &SongControls) -> Vec<SongVariantPreview> {
    let song_tags = tags(&draft.style_tags, &["song", "complete"]);

    vec![
        SongVariantPreview {
            id: "song-variant-city-lights-main".to_string(),
            label: "City Lights full mix".to_string(),
            asset_kind: AudioAssetKind::Song,
            duration_ms: controls.target_duration_ms,
            bpm: controls.bpm,
            musical_key: controls.musical_key.clone(),
            vocal_mode: draft.vocalist,
            stem_kinds: controls.requested_stems.clone(),
            loudness_lufs: -14.0,
            true_peak_dbfs: -1.0,
            lyric_alignment_score: 86,
            structure_match_score: 91,
            tags: song_tags,
            selected_for_save: true,
        },
        SongVariantPreview {
            id: "song-variant-city-lights-instrumental".to_string(),
            label: "City Lights instrumental pass".to_string(),
            asset_kind: AudioAssetKind::MusicClip,
            duration_ms: controls.target_duration_ms,
            bpm: controls.bpm,
            musical_key: controls.musical_key.clone(),
            vocal_mode: SongVocalMode::Instrumental,
            stem_kinds: vec![
                StemKind::Drums,
                StemKind::Bass,
                StemKind::Instruments,
                StemKind::Effects,
            ],
            loudness_lufs: -15.5,
            true_peak_dbfs: -1.4,
            lyric_alignment_score: 0,
            structure_match_score: 88,
            tags: tags(&draft.style_tags, &["instrumental", "alternate"]),
            selected_for_save: true,
        },
    ]
}

fn bars_to_duration_ms(bars: u16, bpm: f32) -> u64 {
    let beats = bars as f32 * 4.0;
    ((60_000.0 / bpm) * beats).round() as u64
}

fn tags(base: &[String], extra: &[&str]) -> Vec<String> {
    let mut tags = base.to_vec();
    tags.extend(extra.iter().map(|tag| (*tag).to_string()));
    tags.sort();
    tags.dedup();
    tags
}

fn generation_recipe(
    draft: &SongDraft,
    controls: &SongControls,
    provider: &SongProviderSelection,
    variants: &[SongVariantPreview],
) -> GenerationRecipe {
    GenerationRecipe {
        id: "recipe-song-city-lights-reference".to_string(),
        workflow: RecipeWorkflow::Song,
        provider: provider.descriptor(),
        request: RecipeRequest::Song(SongRecipe {
            prompt: draft.prompt.clone(),
            lyrics: draft.lyrics.clone(),
            style_tags: draft.style_tags.clone(),
            structure: SongStructure {
                sections: draft
                    .sections
                    .iter()
                    .map(|section| SongSection {
                        label: section.label.clone(),
                        lyrics: section.lyrics.clone(),
                        duration_ms: Some(bars_to_duration_ms(section.bars, controls.bpm)),
                    })
                    .collect(),
            },
            requested_stems: controls.requested_stems.clone(),
            reference_audio_ids: draft.reference_audio_asset_ids.clone(),
        }),
        seed: Some(61_550),
        source_references: source_references(draft),
        post_processing: post_processing_steps(controls),
        parameter_overrides: song_parameters(draft, controls),
        output_asset_ids: variants
            .iter()
            .filter(|variant| variant.selected_for_save)
            .map(|variant| format!("asset-{}", variant.id))
            .collect(),
    }
}

fn source_references(draft: &SongDraft) -> Vec<SourceReference> {
    let mut references = vec![SourceReference {
        id: "source-song-lyrics".to_string(),
        source_type: SourceReferenceType::Text,
        asset_id: None,
        external_uri: None,
        ownership_note: Some("User-authored lyrics stored with the song recipe.".to_string()),
    }];

    references.extend(draft.reference_audio_asset_ids.iter().enumerate().map(
        |(index, asset_id)| SourceReference {
            id: format!("source-song-reference-audio-{}", index + 1),
            source_type: SourceReferenceType::Audio,
            asset_id: Some(asset_id.clone()),
            external_uri: None,
            ownership_note: Some("User-owned style or arrangement reference.".to_string()),
        },
    ));

    references
}

fn song_parameters(
    draft: &SongDraft,
    controls: &SongControls,
) -> BTreeMap<String, serde_json::Value> {
    let mut parameters = BTreeMap::new();
    parameters.insert("title".to_string(), json!(draft.title));
    parameters.insert("styleTags".to_string(), json!(draft.style_tags));
    parameters.insert("language".to_string(), json!(draft.language));
    parameters.insert("vocalist".to_string(), json!(draft.vocalist));
    parameters.insert("singerHint".to_string(), json!(draft.singer_hint));
    parameters.insert("bpm".to_string(), json!(controls.bpm));
    parameters.insert("musicalKey".to_string(), json!(controls.musical_key));
    parameters.insert("timeSignature".to_string(), json!(controls.time_signature));
    parameters.insert(
        "targetDurationMs".to_string(),
        json!(controls.target_duration_ms),
    );
    parameters.insert(
        "sectionLengthBars".to_string(),
        json!(controls.section_length_bars),
    );
    parameters.insert(
        "variationCount".to_string(),
        json!(controls.variation_count),
    );
    parameters.insert("generateStems".to_string(), json!(controls.generate_stems));
    parameters.insert(
        "requestedStems".to_string(),
        json!(controls.requested_stems),
    );
    parameters
}

fn post_processing_steps(controls: &SongControls) -> Vec<PostProcessingStep> {
    let mut master = BTreeMap::new();
    master.insert("targetLoudnessLufs".to_string(), json!(-14.0));
    master.insert("truePeakCeilingDbfs".to_string(), json!(-1.0));

    let mut stem_export = BTreeMap::new();
    stem_export.insert(
        "requestedStems".to_string(),
        json!(controls.requested_stems),
    );

    vec![
        PostProcessingStep {
            id: "post-song-master-loudness".to_string(),
            operation: PostProcessingOperation::Master,
            parameters: master,
        },
        PostProcessingStep {
            id: "post-song-stem-export".to_string(),
            operation: PostProcessingOperation::ConvertFormat,
            parameters: stem_export,
        },
    ]
}

fn saved_outputs(
    submission: &SongSubmissionPreview,
    variants: &[SongVariantPreview],
    allocator: &StoragePathAllocator,
) -> Result<Vec<SongSavedOutput>, StoragePathError> {
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
    submission: &SongSubmissionPreview,
    variant: &SongVariantPreview,
    scope: &LibraryScope,
    allocator: &StoragePathAllocator,
) -> Result<SongSavedOutput, StoragePathError> {
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
            channels: 2,
            duration_ms: variant.duration_ms,
            loudness_lufs: Some(variant.loudness_lufs),
            true_peak_dbfs: Some(variant.true_peak_dbfs),
            has_clipping: false,
            bpm: Some(variant.bpm),
            musical_key: Some(variant.musical_key.clone()),
            loop_points: None,
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
        collection_ids: vec!["collection-song-drafts".to_string()],
        current_version_id: version.id.clone(),
        version_ids: vec![version.id.clone()],
        rights: RightsMetadata {
            license_status: LicenseStatus::ProviderLicensed,
            commercial_use: CommercialUseStatus::Allowed,
            voice_consent: VoiceConsentStatus::NotVoiceMaterial,
            ai_disclosure_required: true,
            watermark: WatermarkStatus::SidecarOnly,
            reference_media_ownership: Some(
                "user-authored lyrics and user-owned references".to_string(),
            ),
        },
        provenance_ids: vec![format!("provenance-{}", variant.id)],
    };

    Ok(SongSavedOutput {
        variant_id: variant.id.clone(),
        asset,
        version,
        storage,
        export_ready: true,
        waveform_preview_ready: true,
    })
}

fn export_targets() -> Vec<SongExportTarget> {
    vec![
        SongExportTarget {
            id: "song-master".to_string(),
            label: "Song master".to_string(),
            formats: vec![AudioFileFormat::Wav, AudioFileFormat::Flac, AudioFileFormat::Mp3],
            includes_stems: false,
            includes_sidecar: true,
            summary: "Export mastered WAV/FLAC/MP3 with recipe, license, and disclosure sidecar.".to_string(),
        },
        SongExportTarget {
            id: "song-stems".to_string(),
            label: "Stem bundle".to_string(),
            formats: vec![AudioFileFormat::Wav, AudioFileFormat::Flac],
            includes_stems: true,
            includes_sidecar: true,
            summary: "Export vocal, drums, bass, and instrument stems when provider or separator supports them.".to_string(),
        },
        SongExportTarget {
            id: "composition-source".to_string(),
            label: "Send to multitrack".to_string(),
            formats: vec![AudioFileFormat::Wav],
            includes_stems: true,
            includes_sidecar: true,
            summary: "Promote generated song and stems into the SoundWorks composition editor.".to_string(),
        },
    ]
}

fn qa_checks() -> Vec<SongQaCheck> {
    vec![
        SongQaCheck {
            id: "songs.capability_controls".to_string(),
            status: SongQaStatus::Passed,
            summary:
                "Lyrics, style, reference audio, duration, and stem controls are gated by provider capabilities."
                    .to_string(),
        },
        SongQaCheck {
            id: "songs.recipe_provenance".to_string(),
            status: SongQaStatus::Passed,
            summary:
                "Song recipes preserve lyrics, section structure, references, seeds, provider metadata, and outputs."
                    .to_string(),
        },
        SongQaCheck {
            id: "songs.preview_versioning".to_string(),
            status: SongQaStatus::Passed,
            summary:
                "Output song variants are represented as previewable, versioned assets with waveform sidecars."
                    .to_string(),
        },
        SongQaCheck {
            id: "songs.provider_gates".to_string(),
            status: SongQaStatus::Warning,
            summary:
                "Stable Audio 3 and ACE-Step need runnable Mac/Windows smoke evidence before product enablement."
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
    use super::{SongControlKind, SongProviderReadiness, SongQaStatus, SongStudioOverview};
    use crate::domain::{AudioAssetKind, RecipeRequest, StemKind};
    use crate::manifests::CapabilityWorkflow;

    #[test]
    fn reference_studio_exposes_complete_song_controls() {
        let overview = SongStudioOverview::reference().expect("song studio builds");

        assert_eq!(overview.schema_version, 1);
        assert_eq!(overview.draft.title, "City Lights Resolve");
        assert_eq!(overview.arrangement.section_count, 4);
        assert!(overview.provider_options.iter().any(|option| {
            option.workflow == CapabilityWorkflow::Song
                && option.supported_controls.contains(&SongControlKind::Lyrics)
                && option.supported_controls.contains(&SongControlKind::Stems)
        }));
        assert!(overview.submission.can_submit);
    }

    #[test]
    fn submission_preserves_lyrics_structure_and_stems() {
        let overview = SongStudioOverview::reference().expect("song studio builds");
        let RecipeRequest::Song(song) = &overview.submission.recipe.request else {
            panic!("expected song recipe");
        };

        assert!(song.lyrics.contains("City lights"));
        assert_eq!(song.structure.sections.len(), 4);
        assert!(song.requested_stems.contains(&StemKind::Vocals));
        assert_eq!(overview.submission.recipe.output_asset_ids.len(), 2);
        assert_eq!(
            overview.submission.job.output_version_ids,
            vec![
                "version-song-variant-city-lights-main-a",
                "version-song-variant-city-lights-instrumental-a"
            ]
        );
    }

    #[test]
    fn scorecards_keep_research_models_behind_runtime_gates() {
        let overview = SongStudioOverview::reference().expect("song studio builds");

        assert_eq!(overview.provider_scorecards.len(), 8);
        assert!(overview.provider_scorecards.iter().any(|scorecard| {
            scorecard.candidate_id == "ace-step-1-5"
                && scorecard.readiness == SongProviderReadiness::NeedsRuntimePort
                && scorecard.recommended
        }));
        assert!(overview.provider_scorecards.iter().any(|scorecard| {
            scorecard.candidate_id == "yue"
                && scorecard.readiness == SongProviderReadiness::ResearchOnly
        }));
    }

    #[test]
    fn saved_outputs_are_versioned_and_export_ready() {
        let overview = SongStudioOverview::reference().expect("song studio builds");

        assert_eq!(overview.saved_outputs.len(), 2);
        assert!(overview
            .saved_outputs
            .iter()
            .all(|output| output.export_ready && output.waveform_preview_ready));
        assert!(overview
            .saved_outputs
            .iter()
            .any(|output| output.asset.kind == AudioAssetKind::Song));
        assert!(overview
            .saved_outputs
            .iter()
            .any(|output| output.asset.kind == AudioAssetKind::MusicClip));
    }

    #[test]
    fn qa_checks_cover_story_acceptance_criteria() {
        let overview = SongStudioOverview::reference().expect("song studio builds");

        assert!(overview
            .qa_checks
            .iter()
            .any(|check| check.status == SongQaStatus::Warning));
        assert!(overview
            .export_targets
            .iter()
            .any(|target| target.includes_stems));
        assert!(overview
            .export_targets
            .iter()
            .all(|target| target.includes_sidecar));
    }
}
