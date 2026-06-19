use crate::domain::{
    AssetCreation, AudioAsset, AudioAssetKind, AudioAssetVersion, AudioFileFormat,
    AudioFileReference, CommercialUseStatus, GenerationJob, GenerationRecipe, JobKind, JobProgress,
    JobStatus, LibraryScope, LicenseStatus, ModelRuntime, RecipeRequest, RecipeWorkflow,
    RightsMetadata, SourceReference, SourceReferenceType, SyncMode, TechnicalAudioMetadata,
    TimeRange, VideoAudioSyncPoint, VideoAudioTargetRange, VideoRegion, VideoToAudioRecipe,
    VoiceConsentStatus, WatermarkStatus,
};
use crate::evaluation::{
    CommercialUseEvaluation, EvaluationLane, EvaluationStatus, ModelEvaluationCandidate,
    ModelEvaluationCatalog, ProductEligibility, ProductRuntimePath,
};
use crate::manifests::{
    CapabilityInput, CapabilityQuery, CapabilityWorkflow, ChannelLayout, ProviderCatalog,
};
use crate::runtime::RuntimeOverview;
use crate::storage::{StoragePathAllocator, StoragePathError, StoragePaths};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const VIDEO_TO_AUDIO_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoToAudioOverview {
    pub schema_version: u32,
    pub project_id: String,
    pub source: VideoSourceInput,
    pub direction: VideoAudioDirection,
    pub target_ranges: Vec<VideoAudioTargetRange>,
    pub detected_events: Vec<DetectedVideoEvent>,
    pub provider_options: Vec<VideoToAudioProviderOption>,
    pub selected_provider: VideoToAudioProviderSelection,
    pub provider_scorecards: Vec<VideoToAudioProviderScorecard>,
    pub sync_preview: SynchronizedAudioPreview,
    pub submission: VideoToAudioSubmission,
    pub saved_output: VideoToAudioSavedOutput,
    pub export_package: VideoToAudioExportPackage,
    pub provenance: MultimodalProvenance,
    pub safety_gates: Vec<VideoAudioSafetyGate>,
    pub validation_checks: Vec<VideoToAudioValidationCheck>,
}

impl VideoToAudioOverview {
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
        let source = source_input();
        let direction = direction();
        let target_ranges = target_ranges();
        let detected_events = detected_events();
        let provider_options = provider_options(catalog, runtime);
        let selected_provider = provider_options
            .iter()
            .find(|option| option.runnable)
            .map(VideoToAudioProviderSelection::from_option)
            .unwrap_or_else(|| VideoToAudioProviderSelection {
                provider_id: "unavailable".to_string(),
                model_id: "unavailable".to_string(),
                model_version: None,
                workflow: CapabilityWorkflow::VideoToAudio,
                runtime: ModelRuntime::ResearchOnly,
                accepted: false,
                blocker: Some(
                    "No runnable video-to-audio provider is registered for prototype submission."
                        .to_string(),
                ),
            });
        let provider_scorecards = provider_scorecards(evaluation);
        let sync_preview = sync_preview(&detected_events);
        let provenance = provenance(&source, &target_ranges);
        let safety_gates = safety_gates();
        let submission = submission(
            &source,
            &direction,
            &target_ranges,
            &sync_preview,
            &selected_provider,
            &safety_gates,
        );
        let saved_output = saved_output(&submission, allocator)?;

        Ok(Self {
            schema_version: VIDEO_TO_AUDIO_SCHEMA_VERSION,
            project_id: "project-demo".to_string(),
            source,
            direction,
            target_ranges,
            detected_events,
            provider_options,
            selected_provider,
            provider_scorecards,
            sync_preview,
            submission,
            saved_output,
            export_package: export_package(),
            provenance,
            safety_gates,
            validation_checks: validation_checks(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoSourceInput {
    pub video_reference_id: String,
    pub video_asset_id: String,
    pub filename: String,
    pub duration_ms: u64,
    pub frame_rate: String,
    pub resolution: String,
    pub has_source_audio: bool,
    pub image_reference_ids: Vec<String>,
    pub reference_audio_asset_ids: Vec<String>,
    pub ownership_attestation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoAudioDirection {
    pub prompt: String,
    pub negative_prompt: String,
    pub sync_mode: SyncMode,
    pub requested_outputs: Vec<AudioAssetKind>,
    pub duration_ms: u64,
    pub regenerate_policy: RegeneratePolicy,
    pub export_target: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RegeneratePolicy {
    WholeTrack,
    SelectedRanges,
    ObjectRegions,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectedVideoEvent {
    pub id: String,
    pub label: String,
    pub at_ms: u64,
    pub confidence: f32,
    pub object_label: Option<String>,
    pub region: Option<VideoRegion>,
    pub requested_sound: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoToAudioProviderOption {
    pub provider_id: String,
    pub model_id: String,
    pub model_version: Option<String>,
    pub display_name: String,
    pub workflow: CapabilityWorkflow,
    pub runtime: ModelRuntime,
    pub runnable: bool,
    pub install_status: String,
    pub output_asset_kinds: Vec<AudioAssetKind>,
    pub output_format: AudioFileFormat,
    pub sample_rate_hz: u32,
    pub channel_layout: ChannelLayout,
    pub supports_video: bool,
    pub supports_text: bool,
    pub supports_reference_audio: bool,
    pub supports_range_refinement: bool,
    pub supports_object_regions: bool,
    pub commercial_use_allowed: bool,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoToAudioProviderSelection {
    pub provider_id: String,
    pub model_id: String,
    pub model_version: Option<String>,
    pub workflow: CapabilityWorkflow,
    pub runtime: ModelRuntime,
    pub accepted: bool,
    pub blocker: Option<String>,
}

impl VideoToAudioProviderSelection {
    fn from_option(option: &VideoToAudioProviderOption) -> Self {
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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoToAudioProviderScorecard {
    pub candidate_id: String,
    pub name: String,
    pub provider: String,
    pub lanes: Vec<EvaluationLane>,
    pub status: EvaluationStatus,
    pub product_eligibility: ProductEligibility,
    pub readiness: VideoToAudioReadiness,
    pub runtime_path: ProductRuntimePath,
    pub commercial_use: CommercialUseEvaluation,
    pub recommended: bool,
    pub supports: Vec<VideoToAudioCapability>,
    pub blockers: Vec<String>,
    pub notes: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VideoToAudioReadiness {
    PrototypeReady,
    NeedsRuntimePort,
    ResearchOnly,
    TextOnlySfx,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VideoToAudioCapability {
    TextToSfx,
    VideoConditioning,
    ImageConditioning,
    ReferenceAudioConditioning,
    FrameSync,
    ObjectRegionRefinement,
    NaturalLanguageEditing,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SynchronizedAudioPreview {
    pub id: String,
    pub duration_ms: u64,
    pub sample_rate_hz: u32,
    pub channel_layout: ChannelLayout,
    pub waveform_preview_path: String,
    pub sync_points: Vec<VideoAudioSyncPoint>,
    pub segments: Vec<VideoAudioSegment>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoAudioSegment {
    pub id: String,
    pub target_range_id: String,
    pub label: String,
    pub range: TimeRange,
    pub asset_kind: AudioAssetKind,
    pub sync_confidence: f32,
    pub editable: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoToAudioSubmission {
    pub can_submit: bool,
    pub job: GenerationJob,
    pub recipe: GenerationRecipe,
    pub blocking_reasons: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoToAudioSavedOutput {
    pub asset: AudioAsset,
    pub version: AudioAssetVersion,
    pub storage: StoragePaths,
    pub waveform_preview_ready: bool,
    pub synchronized_to_video: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoToAudioExportPackage {
    pub id: String,
    pub mixdown_path: String,
    pub sidecar_path: String,
    pub includes_sync_points: bool,
    pub includes_source_media_refs: bool,
    pub includes_detected_events: bool,
    pub includes_rights: bool,
    pub destination_targets: Vec<String>,
    pub required_fields: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MultimodalProvenance {
    pub recipe_id: String,
    pub source_reference_ids: Vec<String>,
    pub sidecar_path: String,
    pub captured_fields: Vec<String>,
    pub round_trip_notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoAudioSafetyGate {
    pub id: String,
    pub status: VideoAudioGateStatus,
    pub summary: String,
    pub enforcement: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VideoAudioGateStatus {
    Passed,
    Warning,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoToAudioValidationCheck {
    pub id: String,
    pub status: VideoAudioGateStatus,
    pub summary: String,
}

fn source_input() -> VideoSourceInput {
    VideoSourceInput {
        video_reference_id: "ref-video-airlock-approach".to_string(),
        video_asset_id: "asset-reference-video-airlock".to_string(),
        filename: "airlock-approach-silent.mp4".to_string(),
        duration_ms: 14_400,
        frame_rate: "24 fps".to_string(),
        resolution: "1920x1080".to_string(),
        has_source_audio: false,
        image_reference_ids: vec!["ref-keyframe-door-panel".to_string()],
        reference_audio_asset_ids: vec!["asset-reference-metal-room-tone".to_string()],
        ownership_attestation: "User-owned previs clip cleared for generated Foley.".to_string(),
    }
}

fn direction() -> VideoAudioDirection {
    VideoAudioDirection {
        prompt:
            "Generate synchronized sci-fi airlock Foley: servo whirr, boot steps, metal hatch hit, pressure seal, and low room tone."
                .to_string(),
        negative_prompt: "music, dialogue, crowd, melody, alarm loop".to_string(),
        sync_mode: SyncMode::FrameSynchronized,
        requested_outputs: vec![AudioAssetKind::Sfx, AudioAssetKind::Ambience],
        duration_ms: 14_400,
        regenerate_policy: RegeneratePolicy::SelectedRanges,
        export_target: "SceneWorks video audio track package".to_string(),
    }
}

fn target_ranges() -> Vec<VideoAudioTargetRange> {
    vec![
        target(
            "range-door-servo",
            "Door servo",
            1_000,
            3_300,
            Some("airlock hatch"),
            Some(VideoRegion {
                x: 0.58,
                y: 0.18,
                width: 0.28,
                height: 0.55,
            }),
            "mechanical motor ramp with subtle metal resonance",
        ),
        target(
            "range-footsteps",
            "Boot steps",
            3_650,
            6_900,
            Some("crew boots"),
            Some(VideoRegion {
                x: 0.28,
                y: 0.58,
                width: 0.22,
                height: 0.3,
            }),
            "three dampened footsteps aligned to contact frames",
        ),
        target(
            "range-pressure-seal",
            "Pressure seal",
            8_200,
            11_600,
            Some("door gasket"),
            None,
            "hatch impact, air hiss, and pressure release tail",
        ),
    ]
}

fn target(
    id: &str,
    label: &str,
    start_ms: u64,
    end_ms: u64,
    object_label: Option<&str>,
    region: Option<VideoRegion>,
    requested_action: &str,
) -> VideoAudioTargetRange {
    VideoAudioTargetRange {
        id: id.to_string(),
        label: label.to_string(),
        range: TimeRange { start_ms, end_ms },
        object_label: object_label.map(str::to_string),
        region,
        requested_action: requested_action.to_string(),
    }
}

fn detected_events() -> Vec<DetectedVideoEvent> {
    vec![
        event(
            "event-servo-start",
            "Servo start",
            1_120,
            0.86,
            Some("airlock hatch"),
            "servo ramp",
        ),
        event(
            "event-step-1",
            "Footstep contact",
            3_920,
            0.81,
            Some("crew boots"),
            "boot step",
        ),
        event(
            "event-step-2",
            "Footstep contact",
            5_180,
            0.79,
            Some("crew boots"),
            "boot step",
        ),
        event(
            "event-hatch-hit",
            "Hatch lock",
            8_640,
            0.88,
            Some("door gasket"),
            "metal impact",
        ),
        event(
            "event-pressure-tail",
            "Pressure tail",
            10_900,
            0.74,
            None,
            "air hiss",
        ),
    ]
}

fn event(
    id: &str,
    label: &str,
    at_ms: u64,
    confidence: f32,
    object_label: Option<&str>,
    requested_sound: &str,
) -> DetectedVideoEvent {
    DetectedVideoEvent {
        id: id.to_string(),
        label: label.to_string(),
        at_ms,
        confidence,
        object_label: object_label.map(str::to_string),
        region: None,
        requested_sound: requested_sound.to_string(),
    }
}

fn provider_options(
    catalog: &ProviderCatalog,
    runtime: &RuntimeOverview,
) -> Vec<VideoToAudioProviderOption> {
    catalog
        .find_matches(&CapabilityQuery {
            workflow: CapabilityWorkflow::VideoToAudio,
            required_inputs: vec![CapabilityInput::SourceVideo, CapabilityInput::TextPrompt],
            output_asset_kind: Some(AudioAssetKind::Sfx),
            channel_layout: Some(ChannelLayout::Stereo),
            duration_ms: Some(14_400),
            require_runnable: true,
            ..CapabilityQuery::default()
        })
        .into_iter()
        .map(|matched| {
            let state = runtime.model_states.iter().find(|state| {
                state.provider_id == matched.provider_id && state.model_id == matched.model_id
            });
            let mut limitations = state
                .map(|state| state.reasons.clone())
                .unwrap_or_else(|| {
                    vec!["Runtime has not reported this provider/model pair.".to_string()]
                });

            limitations.push(
                "Reference provider proves workflow contract only; real video-to-audio adapters remain model-integration work."
                    .to_string(),
            );

            VideoToAudioProviderOption {
                provider_id: matched.provider_id,
                model_id: matched.model_id,
                model_version: matched.model_version,
                display_name: "SoundWorks Reference Capability Registry / Reference Audio Generation Suite"
                    .to_string(),
                workflow: matched.workflow,
                runtime: matched.descriptor.runtime,
                runnable: true,
                install_status: "packaged".to_string(),
                output_asset_kinds: vec![AudioAssetKind::Sfx, AudioAssetKind::Ambience],
                output_format: matched.defaults.format,
                sample_rate_hz: matched.defaults.sample_rate_hz,
                channel_layout: matched.defaults.channel_layout,
                supports_video: true,
                supports_text: true,
                supports_reference_audio: false,
                supports_range_refinement: true,
                supports_object_regions: true,
                commercial_use_allowed: true,
                limitations,
            }
        })
        .collect()
}

fn provider_scorecards(evaluation: &ModelEvaluationCatalog) -> Vec<VideoToAudioProviderScorecard> {
    let recommended_ids = evaluation
        .recommendations
        .iter()
        .map(|recommendation| recommendation.candidate_id.as_str())
        .collect::<Vec<_>>();

    ["mmaudio", "audiox", "thinksound", "moss-soundeffect"]
        .into_iter()
        .filter_map(|candidate_id| {
            evaluation
                .candidates
                .iter()
                .find(|candidate| candidate.id == candidate_id)
                .map(|candidate| {
                    scorecard(candidate, recommended_ids.contains(&candidate.id.as_str()))
                })
        })
        .collect()
}

fn scorecard(
    candidate: &ModelEvaluationCandidate,
    recommended: bool,
) -> VideoToAudioProviderScorecard {
    let readiness = if candidate.id == "moss-soundeffect" {
        VideoToAudioReadiness::TextOnlySfx
    } else {
        match candidate.product_eligibility {
            ProductEligibility::ProductCandidate | ProductEligibility::ApiOnlyCandidate => {
                VideoToAudioReadiness::PrototypeReady
            }
            ProductEligibility::NeedsRuntimePort => VideoToAudioReadiness::NeedsRuntimePort,
            ProductEligibility::ResearchOnly => VideoToAudioReadiness::ResearchOnly,
            ProductEligibility::Blocked => VideoToAudioReadiness::Blocked,
        }
    };
    let supports = match candidate.id.as_str() {
        "mmaudio" => vec![
            VideoToAudioCapability::VideoConditioning,
            VideoToAudioCapability::FrameSync,
            VideoToAudioCapability::TextToSfx,
        ],
        "audiox" => vec![
            VideoToAudioCapability::TextToSfx,
            VideoToAudioCapability::VideoConditioning,
            VideoToAudioCapability::ImageConditioning,
            VideoToAudioCapability::ReferenceAudioConditioning,
        ],
        "thinksound" => vec![
            VideoToAudioCapability::VideoConditioning,
            VideoToAudioCapability::ObjectRegionRefinement,
            VideoToAudioCapability::NaturalLanguageEditing,
            VideoToAudioCapability::FrameSync,
        ],
        "moss-soundeffect" => vec![VideoToAudioCapability::TextToSfx],
        _ => vec![],
    };
    let mut blockers = candidate.blockers.clone();

    if candidate.id == "moss-soundeffect" {
        blockers.push(
            "Text-to-SFX candidate only; use for Foley bed comparison, not video-conditioned sync."
                .to_string(),
        );
    }

    VideoToAudioProviderScorecard {
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
        supports,
        blockers,
        notes: candidate.notes.clone(),
    }
}

fn sync_preview(events: &[DetectedVideoEvent]) -> SynchronizedAudioPreview {
    SynchronizedAudioPreview {
        id: "preview-airlock-foley-sync".to_string(),
        duration_ms: 14_400,
        sample_rate_hz: 48_000,
        channel_layout: ChannelLayout::Stereo,
        waveform_preview_path:
            "soundworks-library/projects/project-demo/sfx/asset-video-airlock-foley/version-video-airlock-foley-a/previews/waveform.json"
                .to_string(),
        sync_points: events
            .iter()
            .map(|event| VideoAudioSyncPoint {
                id: format!("sync-{}", event.id),
                at_ms: event.at_ms,
                label: event.label.clone(),
                confidence: event.confidence,
            })
            .collect(),
        segments: vec![
            segment("segment-servo", "range-door-servo", "Servo ramp", 1_000, 3_300, AudioAssetKind::Sfx, 0.84),
            segment("segment-steps", "range-footsteps", "Boot steps", 3_650, 6_900, AudioAssetKind::Sfx, 0.78),
            segment("segment-pressure", "range-pressure-seal", "Pressure seal", 8_200, 11_600, AudioAssetKind::Sfx, 0.86),
            segment("segment-room-tone", "range-full-bed", "Room tone bed", 0, 14_400, AudioAssetKind::Ambience, 0.72),
        ],
        warnings: vec![
            "Real model sync confidence must be replaced with generated output analysis before release."
                .to_string(),
            "Reference audio is captured as provenance; current provider option does not condition on it."
                .to_string(),
        ],
    }
}

fn segment(
    id: &str,
    target_range_id: &str,
    label: &str,
    start_ms: u64,
    end_ms: u64,
    asset_kind: AudioAssetKind,
    sync_confidence: f32,
) -> VideoAudioSegment {
    VideoAudioSegment {
        id: id.to_string(),
        target_range_id: target_range_id.to_string(),
        label: label.to_string(),
        range: TimeRange { start_ms, end_ms },
        asset_kind,
        sync_confidence,
        editable: true,
    }
}

fn submission(
    source: &VideoSourceInput,
    direction: &VideoAudioDirection,
    target_ranges: &[VideoAudioTargetRange],
    sync_preview: &SynchronizedAudioPreview,
    selected_provider: &VideoToAudioProviderSelection,
    safety_gates: &[VideoAudioSafetyGate],
) -> VideoToAudioSubmission {
    let mut blocking_reasons = vec![];
    let mut warnings = sync_preview.warnings.clone();

    if direction.prompt.trim().is_empty() {
        blocking_reasons.push("Director prompt is empty.".to_string());
    }

    if source.video_reference_id.trim().is_empty() {
        blocking_reasons.push("Source video is required.".to_string());
    }

    if !selected_provider.accepted {
        blocking_reasons.push(selected_provider.blocker.clone().unwrap_or_else(|| {
            "Selected provider cannot currently accept video-to-audio jobs.".to_string()
        }));
    }

    for gate in safety_gates {
        match gate.status {
            VideoAudioGateStatus::Blocked => blocking_reasons.push(gate.summary.clone()),
            VideoAudioGateStatus::Warning => warnings.push(gate.summary.clone()),
            VideoAudioGateStatus::Passed => {}
        }
    }

    let can_submit = blocking_reasons.is_empty();
    let recipe = generation_recipe(
        source,
        direction,
        target_ranges,
        sync_preview,
        selected_provider,
    );

    VideoToAudioSubmission {
        can_submit,
        job: GenerationJob {
            id: "job-video-airlock-foley".to_string(),
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
                    "Ready to queue prototype synchronized Foley generation.".to_string()
                } else {
                    "Video-to-audio submission blocked by validation.".to_string()
                }),
            }),
            output_version_ids: if can_submit {
                vec!["version-video-airlock-foley-a".to_string()]
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

fn generation_recipe(
    source: &VideoSourceInput,
    direction: &VideoAudioDirection,
    target_ranges: &[VideoAudioTargetRange],
    sync_preview: &SynchronizedAudioPreview,
    selected_provider: &VideoToAudioProviderSelection,
) -> GenerationRecipe {
    let mut parameter_overrides = BTreeMap::new();
    parameter_overrides.insert(
        "negativePrompt".to_string(),
        serde_json::json!(direction.negative_prompt),
    );
    parameter_overrides.insert(
        "regeneratePolicy".to_string(),
        serde_json::json!(direction.regenerate_policy),
    );
    parameter_overrides.insert(
        "syncPointCount".to_string(),
        serde_json::json!(sync_preview.sync_points.len()),
    );
    parameter_overrides.insert(
        "exportTarget".to_string(),
        serde_json::json!(direction.export_target),
    );

    GenerationRecipe {
        id: "recipe-video-airlock-foley".to_string(),
        workflow: RecipeWorkflow::VideoToAudio,
        provider: crate::domain::ModelDescriptor {
            provider_id: selected_provider.provider_id.clone(),
            model_id: selected_provider.model_id.clone(),
            model_version: selected_provider.model_version.clone(),
            model_hash: None,
            runtime: selected_provider.runtime,
        },
        request: RecipeRequest::VideoToAudio(VideoToAudioRecipe {
            prompt: direction.prompt.clone(),
            source_video_reference_id: source.video_reference_id.clone(),
            source_image_reference_ids: source.image_reference_ids.clone(),
            source_audio_reference_ids: source.reference_audio_asset_ids.clone(),
            target_ranges: target_ranges.to_vec(),
            detected_event_ids: sync_preview
                .sync_points
                .iter()
                .map(|point| point.id.clone())
                .collect(),
            sync_points: sync_preview.sync_points.clone(),
            sync_mode: direction.sync_mode,
        }),
        seed: Some(6183),
        source_references: source_references(source),
        post_processing: vec![],
        parameter_overrides,
        output_asset_ids: vec!["asset-video-airlock-foley".to_string()],
    }
}

fn source_references(source: &VideoSourceInput) -> Vec<SourceReference> {
    let mut references = vec![SourceReference {
        id: source.video_reference_id.clone(),
        source_type: SourceReferenceType::Video,
        asset_id: Some(source.video_asset_id.clone()),
        external_uri: None,
        ownership_note: Some(source.ownership_attestation.clone()),
    }];

    references.extend(source.image_reference_ids.iter().map(|id| SourceReference {
        id: id.clone(),
        source_type: SourceReferenceType::Image,
        asset_id: None,
        external_uri: None,
        ownership_note: Some("User-provided keyframe reference.".to_string()),
    }));
    references.extend(
        source
            .reference_audio_asset_ids
            .iter()
            .map(|id| SourceReference {
                id: format!("ref-{id}"),
                source_type: SourceReferenceType::Audio,
                asset_id: Some(id.clone()),
                external_uri: None,
                ownership_note: Some(
                    "Reference room tone cleared for conditioning or matching.".to_string(),
                ),
            }),
    );

    references
}

fn saved_output(
    submission: &VideoToAudioSubmission,
    allocator: &StoragePathAllocator,
) -> Result<VideoToAudioSavedOutput, StoragePathError> {
    let asset = AudioAsset {
        id: "asset-video-airlock-foley".to_string(),
        scope: LibraryScope::Project {
            project_id: "project-demo".to_string(),
        },
        kind: AudioAssetKind::Sfx,
        name: "Airlock synchronized Foley".to_string(),
        tags: vec![
            "video-to-audio".to_string(),
            "foley".to_string(),
            "airlock".to_string(),
            "sync".to_string(),
        ],
        collection_ids: vec!["collection-project-sfx".to_string()],
        current_version_id: "version-video-airlock-foley-a".to_string(),
        version_ids: vec!["version-video-airlock-foley-a".to_string()],
        rights: RightsMetadata {
            license_status: LicenseStatus::ProviderLicensed,
            commercial_use: CommercialUseStatus::Allowed,
            voice_consent: VoiceConsentStatus::NotVoiceMaterial,
            ai_disclosure_required: true,
            watermark: WatermarkStatus::SidecarOnly,
            reference_media_ownership: Some("user-owned source video".to_string()),
        },
        provenance_ids: vec!["provenance-video-airlock-foley".to_string()],
    };
    let storage = allocator.allocate_asset_version(
        &asset.scope,
        asset.kind,
        &asset.id,
        &asset.current_version_id,
        AudioFileFormat::Wav,
    )?;
    let version = AudioAssetVersion {
        id: asset.current_version_id.clone(),
        asset_id: asset.id.clone(),
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
            channels: 2,
            duration_ms: 14_400,
            loudness_lufs: Some(-18.0),
            true_peak_dbfs: Some(-1.2),
            has_clipping: false,
            bpm: None,
            musical_key: None,
            loop_points: None,
        },
        created_by: AssetCreation::Generated {
            recipe_id: submission.recipe.id.clone(),
            job_id: submission.job.id.clone(),
        },
        waveform_preview_cache: Some(storage.waveform_preview_path.clone()),
        spectrogram_preview_cache: Some(storage.spectrogram_preview_path.clone()),
    };

    Ok(VideoToAudioSavedOutput {
        asset,
        version,
        storage,
        waveform_preview_ready: true,
        synchronized_to_video: true,
    })
}

fn export_package() -> VideoToAudioExportPackage {
    VideoToAudioExportPackage {
        id: "export-video-airlock-foley".to_string(),
        mixdown_path: "soundworks-exports/project-demo/airlock-approach/foley-mixdown.wav"
            .to_string(),
        sidecar_path:
            "soundworks-exports/project-demo/airlock-approach/video-to-audio-provenance.json"
                .to_string(),
        includes_sync_points: true,
        includes_source_media_refs: true,
        includes_detected_events: true,
        includes_rights: true,
        destination_targets: vec![
            "SoundWorks composition timeline".to_string(),
            "SceneWorks video audio-track package".to_string(),
        ],
        required_fields: vec![
            "sourceVideoReferenceId".to_string(),
            "sourceProjectId".to_string(),
            "timeRanges".to_string(),
            "syncPoints".to_string(),
            "modelProvider".to_string(),
            "sourceMediaRights".to_string(),
            "aiDisclosureRequired".to_string(),
        ],
    }
}

fn provenance(
    source: &VideoSourceInput,
    target_ranges: &[VideoAudioTargetRange],
) -> MultimodalProvenance {
    MultimodalProvenance {
        recipe_id: "recipe-video-airlock-foley".to_string(),
        source_reference_ids: source_references(source)
            .into_iter()
            .map(|reference| reference.id)
            .collect(),
        sidecar_path:
            "soundworks-library/projects/project-demo/sfx/asset-video-airlock-foley/version-video-airlock-foley-a/metadata/recipe-provenance.json"
                .to_string(),
        captured_fields: vec![
            "source video asset and ownership attestation".to_string(),
            "image keyframe references".to_string(),
            "reference audio asset IDs".to_string(),
            "time ranges and object labels".to_string(),
            format!("{} targeted Foley ranges", target_ranges.len()),
            "sync points and confidence scores".to_string(),
            "provider/model/runtime and license gate state".to_string(),
        ],
        round_trip_notes: vec![
            "Saved output can be dragged into the multitrack editor as synchronized SFX."
                .to_string(),
            "SceneWorks handoff package can reuse the SC-6202 manifest shape once target import code is implemented in SceneWorks."
                .to_string(),
        ],
    }
}

fn safety_gates() -> Vec<VideoAudioSafetyGate> {
    vec![
        gate(
            "source-media-rights",
            VideoAudioGateStatus::Passed,
            "Source video is user-owned and cleared for generated Foley.",
            "Allow generation and preserve ownership note in the sidecar.",
        ),
        gate(
            "protected-media-imitation",
            VideoAudioGateStatus::Passed,
            "Prompt avoids requests to imitate protected film, game, or library sounds.",
            "Block export if protected-media imitation language is introduced.",
        ),
        gate(
            "real-provider-audio",
            VideoAudioGateStatus::Warning,
            "Reference contract is queueable, but real generated audio quality is not proven.",
            "Keep provider scorecards research-only until runnable smoke output is attached.",
        ),
    ]
}

fn gate(
    id: &str,
    status: VideoAudioGateStatus,
    summary: &str,
    enforcement: &str,
) -> VideoAudioSafetyGate {
    VideoAudioSafetyGate {
        id: id.to_string(),
        status,
        summary: summary.to_string(),
        enforcement: enforcement.to_string(),
    }
}

fn validation_checks() -> Vec<VideoToAudioValidationCheck> {
    vec![
        check(
            "video_audio.source_inputs",
            VideoAudioGateStatus::Passed,
            "Workflow captures video, image keyframe, reference audio, and natural-language direction inputs.",
        ),
        check(
            "video_audio.range_refinement",
            VideoAudioGateStatus::Passed,
            "Target ranges preserve time spans, object labels, optional regions, and requested sounds.",
        ),
        check(
            "video_audio.capability_boundary",
            VideoAudioGateStatus::Passed,
            "Provider scorecards distinguish video-conditioned candidates from text-only SFX candidates.",
        ),
        check(
            "video_audio.export_sidecar",
            VideoAudioGateStatus::Passed,
            "Export package includes source media, sync points, detected events, rights, and disclosure fields.",
        ),
        check(
            "video_audio.real_provider_evidence",
            VideoAudioGateStatus::Warning,
            "Real provider adapters and generated audio bytes still require later runnable model integration.",
        ),
    ]
}

fn check(id: &str, status: VideoAudioGateStatus, summary: &str) -> VideoToAudioValidationCheck {
    VideoToAudioValidationCheck {
        id: id.to_string(),
        status,
        summary: summary.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::{VideoAudioGateStatus, VideoToAudioOverview, VideoToAudioReadiness};
    use crate::domain::{AudioAssetKind, RecipeRequest, RecipeWorkflow, SourceReferenceType};

    #[test]
    fn reference_workflow_captures_multimodal_inputs_and_sync() {
        let overview = VideoToAudioOverview::reference().expect("reference video workflow");

        assert_eq!(overview.schema_version, 1);
        assert_eq!(
            overview.source.video_reference_id,
            "ref-video-airlock-approach"
        );
        assert_eq!(overview.source.image_reference_ids.len(), 1);
        assert_eq!(overview.source.reference_audio_asset_ids.len(), 1);
        assert_eq!(overview.target_ranges.len(), 3);
        assert_eq!(overview.detected_events.len(), 5);
        assert_eq!(overview.sync_preview.sync_points.len(), 5);
        assert_eq!(overview.sync_preview.segments.len(), 4);
        assert!(overview.submission.can_submit);
    }

    #[test]
    fn recipe_preserves_ranges_sync_points_and_source_references() {
        let overview = VideoToAudioOverview::reference().expect("reference video workflow");
        let recipe = &overview.submission.recipe;

        assert_eq!(recipe.workflow, RecipeWorkflow::VideoToAudio);
        assert_eq!(recipe.output_asset_ids, vec!["asset-video-airlock-foley"]);
        assert!(recipe
            .source_references
            .iter()
            .any(|reference| reference.source_type == SourceReferenceType::Video));
        assert!(recipe
            .source_references
            .iter()
            .any(|reference| reference.source_type == SourceReferenceType::Image));
        assert!(recipe
            .source_references
            .iter()
            .any(|reference| reference.source_type == SourceReferenceType::Audio));

        let RecipeRequest::VideoToAudio(request) = &recipe.request else {
            panic!("expected video-to-audio request");
        };

        assert_eq!(request.target_ranges.len(), 3);
        assert_eq!(request.sync_points.len(), 5);
        assert_eq!(request.source_audio_reference_ids.len(), 1);
    }

    #[test]
    fn scorecards_distinguish_video_conditioning_from_text_sfx() {
        let overview = VideoToAudioOverview::reference().expect("reference video workflow");
        let mmaudio = overview
            .provider_scorecards
            .iter()
            .find(|scorecard| scorecard.candidate_id == "mmaudio")
            .expect("MMAudio scorecard");
        let moss = overview
            .provider_scorecards
            .iter()
            .find(|scorecard| scorecard.candidate_id == "moss-soundeffect")
            .expect("MOSS scorecard");

        assert!(mmaudio.recommended);
        assert_eq!(mmaudio.readiness, VideoToAudioReadiness::ResearchOnly);
        assert_eq!(moss.readiness, VideoToAudioReadiness::TextOnlySfx);
        assert!(moss
            .blockers
            .iter()
            .any(|blocker| blocker.contains("Text-to-SFX candidate only")));
    }

    #[test]
    fn saved_output_and_export_sidecar_capture_auditable_handoff() {
        let overview = VideoToAudioOverview::reference().expect("reference video workflow");

        assert_eq!(overview.saved_output.asset.kind, AudioAssetKind::Sfx);
        assert!(overview.saved_output.synchronized_to_video);
        assert!(overview
            .saved_output
            .storage
            .sidecar_path
            .contains("recipe-provenance.json"));
        assert!(overview.export_package.includes_sync_points);
        assert!(overview.export_package.includes_detected_events);
        assert!(overview.export_package.includes_rights);
        assert!(overview
            .export_package
            .destination_targets
            .contains(&"SceneWorks video audio-track package".to_string()));
        assert!(overview
            .safety_gates
            .iter()
            .any(|gate| gate.status == VideoAudioGateStatus::Warning));
    }
}
