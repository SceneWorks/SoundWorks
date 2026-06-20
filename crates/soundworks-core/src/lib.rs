use serde::{Deserialize, Serialize};

pub mod asset_library;
pub mod composition_editor;
pub mod domain;
pub mod evaluation;
pub mod export_workflow;
pub mod fixtures;
pub mod loudness;
pub mod manifests;
pub mod model_manager;
pub mod mvp_validation;
pub mod project_library;
pub mod review;
pub mod rights;
pub mod runtime;
pub mod samples;
pub mod sfx;
pub mod songs;
pub mod storage;
pub mod studio_common;
pub mod tts;
pub mod video_to_audio;
pub mod voice_lab;
pub mod workspace;

pub use asset_library::*;
pub use composition_editor::*;
pub use domain::*;
pub use evaluation::*;
pub use export_workflow::*;
pub use fixtures::*;
pub use manifests::*;
pub use model_manager::*;
pub use mvp_validation::*;
pub use project_library::*;
pub use review::*;
pub use rights::*;
pub use runtime::*;
pub use samples::*;
pub use sfx::*;
pub use songs::*;
pub use storage::*;
pub use studio_common::*;
pub use tts::*;
pub use video_to_audio::*;
pub use voice_lab::*;
pub use workspace::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppOverview {
    pub product_name: String,
    pub architecture: ArchitectureOverview,
    pub studios: Vec<StudioSurface>,
    pub commands: Vec<CommandBoundary>,
    pub workspace: WorkspaceSummary,
    pub provider_catalog: ProviderCatalogOverview,
    pub asset_library: AssetLibrarySummary,
    pub export_workflow: ExportWorkflowSummary,
    pub composition_editor: CompositionEditorSummary,
    pub mvp_validation: MvpValidationSummary,
    pub model_evaluation: ModelEvaluationOverview,
    pub model_manager: ModelManagerSummary,
    pub tts_studio: TtsStudioSummary,
    pub voice_lab: VoiceLabSummary,
    pub sfx_studio: SfxStudioSummary,
    pub samples_studio: SamplesStudioSummary,
    pub song_studio: SongStudioSummary,
    pub review_workspace: ReviewWorkspaceSummary,
    pub rights_safety: RightsSafetySummary,
    pub video_to_audio: VideoToAudioSummary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchitectureOverview {
    pub layers: Vec<ArchitectureLayer>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchitectureLayer {
    pub id: String,
    pub name: String,
    pub responsibility: String,
    pub status: ScaffoldStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StudioSurface {
    pub id: String,
    pub name: String,
    pub route: String,
    pub status: ScaffoldStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandBoundary {
    pub name: String,
    pub direction: CommandDirection,
    pub purpose: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderCatalogOverview {
    pub schema_version: u32,
    pub provider_count: usize,
    pub model_count: usize,
    pub capability_count: usize,
    pub workflows: Vec<CapabilityWorkflowSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetLibrarySummary {
    pub schema_version: u32,
    pub item_count: usize,
    pub previewable_item_count: usize,
    pub collection_count: usize,
    pub scope_count: usize,
    pub filter_count: usize,
    pub supported_type_count: usize,
    pub favorite_count: usize,
    pub rejected_count: usize,
    pub archived_count: usize,
    pub selected_item_id: Option<String>,
    pub selected_item_type: Option<LibraryItemType>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceSummary {
    pub schema_version: u32,
    pub project_count: usize,
    pub project_asset_count: usize,
    pub global_asset_count: usize,
    pub linked_global_asset_count: usize,
    pub transfer_action_count: usize,
    pub source_picker_target_count: usize,
    pub parity_note_count: usize,
    pub active_project_id: String,
    pub global_library_id: String,
    pub can_create_project: bool,
    pub can_open_project: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportWorkflowSummary {
    pub schema_version: u32,
    pub preset_count: usize,
    pub target_count: usize,
    pub sidecar_count: usize,
    pub ready_target_count: usize,
    pub selected_preset_id: String,
    pub selected_source_kind: ExportSourceKind,
    pub selected_format_count: usize,
    pub can_export_selected: bool,
    pub writes_daw_bundle: bool,
    pub writes_scene_works_package: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompositionEditorSummary {
    pub schema_version: u32,
    pub track_count: usize,
    pub clip_count: usize,
    pub asset_bin_count: usize,
    pub enabled_tool_count: usize,
    pub marker_count: usize,
    pub section_count: usize,
    pub selected_clip_id: String,
    pub can_render_mixdown: bool,
    pub editable_asset_kinds: Vec<AudioAssetKind>,
    pub recommended_component_id: String,
    pub component_candidate_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilityWorkflowSummary {
    pub workflow: CapabilityWorkflow,
    pub default_provider_id: String,
    pub default_model_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TtsStudioSummary {
    pub schema_version: u32,
    pub segment_count: usize,
    pub speaker_count: usize,
    pub provider_count: usize,
    pub can_submit: bool,
    pub selected_provider_id: String,
    pub selected_model_id: String,
    pub saved_asset_kind: AudioAssetKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceLabSummary {
    pub schema_version: u32,
    pub mode_count: usize,
    pub profile_count: usize,
    pub provider_count: usize,
    pub safety_gate_count: usize,
    pub can_submit_conversion: bool,
    pub selected_conversion_candidate_id: String,
    pub saved_asset_kind: AudioAssetKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SfxStudioSummary {
    pub schema_version: u32,
    pub variant_count: usize,
    pub saved_output_count: usize,
    pub provider_count: usize,
    pub scorecard_count: usize,
    pub can_submit: bool,
    pub selected_provider_id: String,
    pub selected_model_id: String,
    pub saved_asset_kinds: Vec<AudioAssetKind>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SamplesStudioSummary {
    pub schema_version: u32,
    pub variant_count: usize,
    pub saved_output_count: usize,
    pub provider_count: usize,
    pub scorecard_count: usize,
    pub can_submit: bool,
    pub selected_provider_id: String,
    pub selected_model_id: String,
    pub pack_collection_id: String,
    pub saved_asset_kinds: Vec<AudioAssetKind>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SongStudioSummary {
    pub schema_version: u32,
    pub section_count: usize,
    pub variant_count: usize,
    pub saved_output_count: usize,
    pub provider_count: usize,
    pub scorecard_count: usize,
    pub can_submit: bool,
    pub selected_provider_id: String,
    pub selected_model_id: String,
    pub requested_stems: Vec<StemKind>,
    pub saved_asset_kinds: Vec<AudioAssetKind>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewWorkspaceSummary {
    pub schema_version: u32,
    pub asset_count: usize,
    pub previewable_asset_count: usize,
    pub edit_action_count: usize,
    pub comparison_count: usize,
    pub can_save_edit: bool,
    pub active_asset_id: String,
    pub edited_version_id: String,
    pub source_asset_kinds: Vec<AudioAssetKind>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RightsSafetySummary {
    pub schema_version: u32,
    pub consent_check_count: usize,
    pub blocked_consent_count: usize,
    pub model_decision_count: usize,
    pub blocked_model_decision_count: usize,
    pub policy_gate_count: usize,
    pub blocked_gate_count: usize,
    pub sidecar_count: usize,
    pub disclosure_count: usize,
    pub can_export: bool,
    pub watermark_policy: WatermarkPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoToAudioSummary {
    pub schema_version: u32,
    pub source_duration_ms: u64,
    pub target_range_count: usize,
    pub detected_event_count: usize,
    pub sync_point_count: usize,
    pub provider_count: usize,
    pub scorecard_count: usize,
    pub can_submit: bool,
    pub selected_provider_id: String,
    pub selected_model_id: String,
    pub saved_asset_kind: AudioAssetKind,
    pub export_target_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CommandDirection {
    UiToBackend,
    BackendToUi,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ScaffoldStatus {
    Planned,
    Scaffolded,
}

impl AppOverview {
    pub fn baseline() -> Self {
        Self {
            product_name: "SoundWorks".to_string(),
            architecture: ArchitectureOverview {
                layers: vec![
                    ArchitectureLayer {
                        id: "react-ui".to_string(),
                        name: "React UI".to_string(),
                        responsibility:
                            "Workflow surfaces, library navigation, waveform review, and composition controls."
                                .to_string(),
                        status: ScaffoldStatus::Scaffolded,
                    },
                    ArchitectureLayer {
                        id: "tauri-commands".to_string(),
                        name: "Tauri Commands".to_string(),
                        responsibility:
                            "Narrow command bridge between the UI and local Rust services.".to_string(),
                        status: ScaffoldStatus::Scaffolded,
                    },
                    ArchitectureLayer {
                        id: "soundworks-core".to_string(),
                        name: "Rust Core".to_string(),
                        responsibility:
                            "Shared domain contracts for assets, recipes, jobs, providers, and exports."
                                .to_string(),
                        status: ScaffoldStatus::Scaffolded,
                    },
                    ArchitectureLayer {
                        id: "worker-runtime".to_string(),
                        name: "Worker Runtime".to_string(),
                        responsibility:
                            "Model execution, installation, device capabilities, progress, and cancellation."
                                .to_string(),
                        status: ScaffoldStatus::Scaffolded,
                    },
                ],
            },
            studios: vec![
                StudioSurface::scaffolded("tts", "TTS Studio", "/studios/tts"),
                StudioSurface::scaffolded("voice-lab", "Voice Lab", "/studios/voice-lab"),
                StudioSurface::scaffolded("sfx", "SFX + Ambience", "/studios/sfx"),
                StudioSurface::scaffolded("loops", "Samples + Loops", "/studios/loops"),
                StudioSurface::scaffolded("songs", "Song Studio", "/studios/songs"),
                StudioSurface::scaffolded("review", "Waveform Review", "/review"),
                StudioSurface::scaffolded("rights-safety", "Rights + Safety", "/rights"),
                StudioSurface::scaffolded(
                    "composition-editor",
                    "Multitrack Editor",
                    "/composition",
                ),
                StudioSurface::scaffolded(
                    "video-to-audio",
                    "Video to Audio",
                    "/studios/video-to-audio",
                ),
            ],
            commands: vec![
                CommandBoundary {
                    name: "get_app_overview".to_string(),
                    direction: CommandDirection::UiToBackend,
                    purpose:
                        "Load scaffolded architecture and workflow metadata from the Rust backend."
                            .to_string(),
                },
                CommandBoundary {
                    name: "get_provider_catalog".to_string(),
                    direction: CommandDirection::UiToBackend,
                    purpose:
                        "Load provider/model manifests, capability defaults, and matching inputs."
                            .to_string(),
                },
                CommandBoundary {
                    name: "get_workspace_overview".to_string(),
                    direction: CommandDirection::UiToBackend,
                    purpose:
                        "Load active project workspace, global library, source picker, reuse actions, and SceneWorks-style scope conventions."
                            .to_string(),
                },
                CommandBoundary {
                    name: "get_asset_library_overview".to_string(),
                    direction: CommandDirection::UiToBackend,
                    purpose:
                        "Load searchable asset library facets, project/global scope, lifecycle state, collections, previews, and provenance detail."
                            .to_string(),
                },
                CommandBoundary {
                    name: "get_export_workflow_overview".to_string(),
                    direction: CommandDirection::UiToBackend,
                    purpose:
                        "Load export presets, formats, stem bundles, DAW handoff, SceneWorks handoff, and metadata sidecar readiness."
                            .to_string(),
                },
                CommandBoundary {
                    name: "get_composition_editor_overview".to_string(),
                    direction: CommandDirection::UiToBackend,
                    purpose:
                        "Load multitrack timeline state, asset placement readiness, clip edit tools, mixer state, render plan, and editor component decision evidence."
                            .to_string(),
                },
                CommandBoundary {
                    name: "get_runtime_overview".to_string(),
                    direction: CommandDirection::UiToBackend,
                    purpose:
                        "Report worker runtime policy, device/model state, job progress, and cancellation readiness."
                            .to_string(),
                },
                CommandBoundary {
                    name: "get_model_evaluation_catalog".to_string(),
                    direction: CommandDirection::UiToBackend,
                    purpose:
                        "Load source-backed model scorecards, fixtures, recommendation status, and product eligibility gates."
                            .to_string(),
                },
                CommandBoundary {
                    name: "get_mvp_validation_overview".to_string(),
                    direction: CommandDirection::UiToBackend,
                    purpose:
                        "Load MVP validation matrix, demo workflows, fixtures, scorecards, stress cases, and release gate."
                            .to_string(),
                },
                CommandBoundary {
                    name: "get_tts_studio_overview".to_string(),
                    direction: CommandDirection::UiToBackend,
                    purpose:
                        "Load TTS script segmentation, voice consent gates, provider limits, submission preview, and saved voice-clip output."
                            .to_string(),
                },
                CommandBoundary {
                    name: "get_voice_lab_overview".to_string(),
                    direction: CommandDirection::UiToBackend,
                    purpose:
                        "Load voice profile consent state, clone/fine-tune/conversion modes, provider scorecards, safety gates, and saved conversion output."
                            .to_string(),
                },
                CommandBoundary {
                    name: "get_sfx_studio_overview".to_string(),
                    direction: CommandDirection::UiToBackend,
                    purpose:
                        "Load SFX and ambience prompts, capability-driven controls, variant previews, provider scorecards, loop checks, post-processing, and saved outputs."
                            .to_string(),
                },
                CommandBoundary {
                    name: "get_samples_studio_overview".to_string(),
                    direction: CommandDirection::UiToBackend,
                    purpose:
                        "Load instrument sample and loop controls, provider scorecards, sample-pack variants, QA checks, recipes, and saved outputs."
                            .to_string(),
                },
                CommandBoundary {
                    name: "get_song_studio_overview".to_string(),
                    direction: CommandDirection::UiToBackend,
                    purpose:
                        "Load complete-song lyrics, structure, style controls, provider scorecards, variants, recipes, stems, export targets, and saved outputs."
                            .to_string(),
                },
                CommandBoundary {
                    name: "get_review_workspace_overview".to_string(),
                    direction: CommandDirection::UiToBackend,
                    purpose:
                        "Load waveform review transport, preview caches, lightweight edit actions, non-destructive edited versions, comparison state, and recipe provenance."
                            .to_string(),
                },
                CommandBoundary {
                    name: "get_rights_safety_overview".to_string(),
                    direction: CommandDirection::UiToBackend,
                    purpose:
                        "Load rights, consent, model-license, disclosure, watermark, and export provenance policy gates."
                            .to_string(),
                },
                CommandBoundary {
                    name: "get_video_to_audio_overview".to_string(),
                    direction: CommandDirection::UiToBackend,
                    purpose:
                        "Load multimodal video/image/audio-conditioned Foley workflow state, provider readiness, sync preview, provenance, safety gates, and export package metadata."
                            .to_string(),
                },
            ],
            workspace: WorkspaceSummary::from_overview(
                &WorkspaceOverview::reference().expect("reference workspace is valid"),
            ),
            provider_catalog: ProviderCatalogOverview::from_catalog(&ProviderCatalog::reference()),
            asset_library: AssetLibrarySummary::from_overview(
                &AssetLibraryOverview::reference().expect("reference Asset Library is valid"),
            ),
            export_workflow: ExportWorkflowSummary::from_overview(
                &ExportWorkflowOverview::reference(),
            ),
            composition_editor: CompositionEditorSummary::from_overview(
                &CompositionEditorOverview::reference(),
            ),
            mvp_validation: MvpValidationSummary::from_overview(&MvpValidationOverview::reference()),
            model_evaluation: ModelEvaluationCatalog::reference().overview(),
            model_manager: ModelManagerOverview::reference().summary,
            tts_studio: TtsStudioSummary::from_overview(
                &TtsStudioOverview::reference().expect("reference TTS studio is valid"),
            ),
            voice_lab: VoiceLabSummary::from_overview(
                &VoiceLabOverview::reference().expect("reference Voice Lab is valid"),
            ),
            sfx_studio: SfxStudioSummary::from_overview(
                &SfxStudioOverview::reference().expect("reference SFX Studio is valid"),
            ),
            samples_studio: SamplesStudioSummary::from_overview(
                &SamplesStudioOverview::reference().expect("reference Samples Studio is valid"),
            ),
            song_studio: SongStudioSummary::from_overview(
                &SongStudioOverview::reference().expect("reference Song Studio is valid"),
            ),
            review_workspace: ReviewWorkspaceSummary::from_overview(
                &ReviewWorkspaceOverview::reference().expect("reference Review workspace is valid"),
            ),
            rights_safety: RightsSafetySummary::from_overview(&RightsSafetyOverview::reference()),
            video_to_audio: VideoToAudioSummary::from_overview(
                &VideoToAudioOverview::reference().expect("reference Video to Audio is valid"),
            ),
        }
    }
}

impl WorkspaceSummary {
    pub fn from_overview(overview: &WorkspaceOverview) -> Self {
        Self {
            schema_version: overview.schema_version,
            project_count: overview.recent_projects.len(),
            project_asset_count: overview.project_assets.len(),
            global_asset_count: overview.global_assets.len(),
            linked_global_asset_count: overview.active_project.linked_global_asset_count,
            transfer_action_count: overview.transfer_actions.len(),
            source_picker_target_count: overview.source_picker.target_surfaces.len(),
            parity_note_count: overview.parity_notes.len(),
            active_project_id: overview.active_project.project.id.clone(),
            global_library_id: overview.workspace.global_library_id.clone(),
            can_create_project: overview.active_project.can_create_from_template,
            can_open_project: overview.active_project.can_open,
        }
    }
}

impl VideoToAudioSummary {
    pub fn from_overview(overview: &VideoToAudioOverview) -> Self {
        Self {
            schema_version: overview.schema_version,
            source_duration_ms: overview.source.duration_ms,
            target_range_count: overview.target_ranges.len(),
            detected_event_count: overview.detected_events.len(),
            sync_point_count: overview.sync_preview.sync_points.len(),
            provider_count: overview.provider_options.len(),
            scorecard_count: overview.provider_scorecards.len(),
            can_submit: overview.submission.can_submit,
            selected_provider_id: overview.selected_provider.provider_id.clone(),
            selected_model_id: overview.selected_provider.model_id.clone(),
            saved_asset_kind: overview.saved_output.asset.kind,
            export_target_count: overview.export_package.destination_targets.len(),
        }
    }
}

impl TtsStudioSummary {
    pub fn from_overview(overview: &TtsStudioOverview) -> Self {
        Self {
            schema_version: overview.schema_version,
            segment_count: overview.script.segments.len(),
            speaker_count: overview.speakers.len(),
            provider_count: overview.provider_options.len(),
            can_submit: overview.submission.can_submit,
            selected_provider_id: overview.selected_provider.provider_id.clone(),
            selected_model_id: overview.selected_provider.model_id.clone(),
            saved_asset_kind: overview.saved_output.asset.kind,
        }
    }
}

impl SfxStudioSummary {
    pub fn from_overview(overview: &SfxStudioOverview) -> Self {
        let mut saved_asset_kinds = overview
            .saved_outputs
            .iter()
            .map(|output| output.asset.kind)
            .collect::<Vec<_>>();
        saved_asset_kinds.sort_by_key(|kind| format!("{kind:?}"));
        saved_asset_kinds.dedup();

        Self {
            schema_version: overview.schema_version,
            variant_count: overview.variants.len(),
            saved_output_count: overview.saved_outputs.len(),
            provider_count: overview.provider_options.len(),
            scorecard_count: overview.provider_scorecards.len(),
            can_submit: overview.submission.can_submit,
            selected_provider_id: overview.selected_provider.provider_id.clone(),
            selected_model_id: overview.selected_provider.model_id.clone(),
            saved_asset_kinds,
        }
    }
}

impl SamplesStudioSummary {
    pub fn from_overview(overview: &SamplesStudioOverview) -> Self {
        let mut saved_asset_kinds = overview
            .saved_outputs
            .iter()
            .map(|output| output.asset.kind)
            .collect::<Vec<_>>();
        saved_asset_kinds.sort_by_key(|kind| format!("{kind:?}"));
        saved_asset_kinds.dedup();

        Self {
            schema_version: overview.schema_version,
            variant_count: overview.variants.len(),
            saved_output_count: overview.saved_outputs.len(),
            provider_count: overview.provider_options.len(),
            scorecard_count: overview.provider_scorecards.len(),
            can_submit: overview.submission.can_submit,
            selected_provider_id: overview.selected_provider.provider_id.clone(),
            selected_model_id: overview.selected_provider.model_id.clone(),
            pack_collection_id: overview.pack.collection_id.clone(),
            saved_asset_kinds,
        }
    }
}

impl SongStudioSummary {
    pub fn from_overview(overview: &SongStudioOverview) -> Self {
        let mut saved_asset_kinds = overview
            .saved_outputs
            .iter()
            .map(|output| output.asset.kind)
            .collect::<Vec<_>>();
        saved_asset_kinds.sort_by_key(|kind| format!("{kind:?}"));
        saved_asset_kinds.dedup();

        Self {
            schema_version: overview.schema_version,
            section_count: overview.arrangement.section_count,
            variant_count: overview.variants.len(),
            saved_output_count: overview.saved_outputs.len(),
            provider_count: overview.provider_options.len(),
            scorecard_count: overview.provider_scorecards.len(),
            can_submit: overview.submission.can_submit,
            selected_provider_id: overview.selected_provider.provider_id.clone(),
            selected_model_id: overview.selected_provider.model_id.clone(),
            requested_stems: overview.controls.requested_stems.clone(),
            saved_asset_kinds,
        }
    }
}

impl VoiceLabSummary {
    pub fn from_overview(overview: &VoiceLabOverview) -> Self {
        Self {
            schema_version: overview.schema_version,
            mode_count: overview.modes.len(),
            profile_count: overview.voice_profiles.len(),
            provider_count: overview.provider_scorecards.len(),
            safety_gate_count: overview.safety_gates.len(),
            can_submit_conversion: overview.selected_conversion.can_submit,
            selected_conversion_candidate_id: overview
                .selected_conversion
                .recipe
                .provider
                .model_id
                .clone(),
            saved_asset_kind: overview.saved_output.asset.kind,
        }
    }
}

impl ReviewWorkspaceSummary {
    pub fn from_overview(overview: &ReviewWorkspaceOverview) -> Self {
        let mut source_asset_kinds = overview
            .assets
            .iter()
            .map(|asset| asset.asset.kind)
            .collect::<Vec<_>>();
        source_asset_kinds.sort_by_key(|kind| format!("{kind:?}"));
        source_asset_kinds.dedup();

        Self {
            schema_version: overview.schema_version,
            asset_count: overview.assets.len(),
            previewable_asset_count: overview
                .assets
                .iter()
                .filter(|asset| asset.can_preview)
                .count(),
            edit_action_count: overview.edit_actions.len(),
            comparison_count: 1,
            can_save_edit: overview.edit_submission.can_save,
            active_asset_id: overview.selected_asset.asset.id.clone(),
            edited_version_id: overview.edit_submission.saved_version.id.clone(),
            source_asset_kinds,
        }
    }
}

impl RightsSafetySummary {
    pub fn from_overview(overview: &RightsSafetyOverview) -> Self {
        Self {
            schema_version: overview.schema_version,
            consent_check_count: overview.consent_checks.len(),
            blocked_consent_count: overview
                .consent_checks
                .iter()
                .filter(|check| check.decision == PolicyDecision::Blocked)
                .count(),
            model_decision_count: overview.model_use_decisions.len(),
            blocked_model_decision_count: overview
                .model_use_decisions
                .iter()
                .filter(|decision| decision.decision == PolicyDecision::Blocked)
                .count(),
            policy_gate_count: overview.content_policy_gates.len(),
            blocked_gate_count: overview
                .content_policy_gates
                .iter()
                .filter(|gate| gate.status == PolicyGateStatus::Blocked)
                .count(),
            sidecar_count: overview.export_sidecars.len(),
            disclosure_count: overview
                .disclosure_checks
                .iter()
                .filter(|check| check.required)
                .count(),
            can_export: overview.can_export(),
            watermark_policy: overview.policy.watermark_policy,
        }
    }
}

impl AssetLibrarySummary {
    pub fn from_overview(overview: &AssetLibraryOverview) -> Self {
        Self {
            schema_version: overview.schema_version,
            item_count: overview.items.len(),
            previewable_item_count: overview
                .items
                .iter()
                .filter(|item| item.quick_audition.previewable)
                .count(),
            collection_count: overview.collections.len(),
            scope_count: overview.scopes.len(),
            filter_count: overview.filters.facets.len(),
            supported_type_count: overview.filters.supported_item_types.len(),
            favorite_count: overview.items.iter().filter(|item| item.favorite).count(),
            rejected_count: overview.items.iter().filter(|item| item.rejected).count(),
            archived_count: overview.items.iter().filter(|item| item.archived).count(),
            selected_item_id: overview
                .selected_item
                .as_ref()
                .map(|detail| detail.item.id.clone()),
            selected_item_type: overview
                .selected_item
                .as_ref()
                .map(|detail| detail.item.item_type),
        }
    }
}

impl ExportWorkflowSummary {
    pub fn from_overview(overview: &ExportWorkflowOverview) -> Self {
        Self {
            schema_version: overview.schema_version,
            preset_count: overview.presets.len(),
            target_count: overview.targets.len(),
            sidecar_count: overview.sidecars.len(),
            ready_target_count: overview
                .targets
                .iter()
                .filter(|target| target.ready)
                .count(),
            selected_preset_id: overview.selected_export.preset_id.clone(),
            selected_source_kind: overview.selected_export.source_kind,
            selected_format_count: overview.selected_export.formats.len(),
            can_export_selected: overview.selected_export.can_export,
            writes_daw_bundle: overview.daw_handoff.includes_zip_bundle,
            writes_scene_works_package: overview.scene_works_handoff.includes_optional_stems,
        }
    }
}

impl CompositionEditorSummary {
    pub fn from_overview(overview: &CompositionEditorOverview) -> Self {
        let mut editable_asset_kinds = overview
            .tracks
            .iter()
            .flat_map(|track| track.clips.iter().map(|clip| clip.asset_kind))
            .collect::<Vec<_>>();
        editable_asset_kinds.sort_by_key(|kind| format!("{kind:?}"));
        editable_asset_kinds.dedup();

        let recommended_component_id = overview
            .component_decisions
            .iter()
            .find(|decision| decision.fit == ComponentFit::StrongPrototypeCandidate)
            .map(|decision| decision.id.clone())
            .unwrap_or_else(|| "needs-spike".to_string());

        Self {
            schema_version: overview.schema_version,
            track_count: overview.tracks.len(),
            clip_count: overview.tracks.iter().map(|track| track.clip_count).sum(),
            asset_bin_count: overview.asset_bin.len(),
            enabled_tool_count: overview.tools.iter().filter(|tool| tool.enabled).count(),
            marker_count: overview.composition.markers.len(),
            section_count: overview.composition.sections.len(),
            selected_clip_id: overview.timeline.selected_clip_id.clone(),
            can_render_mixdown: overview.export_plan.can_render_mixdown,
            editable_asset_kinds,
            recommended_component_id,
            component_candidate_count: overview.component_decisions.len(),
        }
    }
}

impl ProviderCatalogOverview {
    pub fn from_catalog(catalog: &ProviderCatalog) -> Self {
        Self {
            schema_version: catalog.schema_version,
            provider_count: catalog.providers.len(),
            model_count: catalog.model_count(),
            capability_count: catalog.capability_count(),
            workflows: catalog
                .workflow_coverage()
                .into_iter()
                .filter_map(|workflow| {
                    catalog
                        .default_for(workflow)
                        .map(|default| CapabilityWorkflowSummary {
                            workflow,
                            default_provider_id: default.provider_id,
                            default_model_id: default.model_id,
                        })
                })
                .collect(),
        }
    }
}

impl StudioSurface {
    fn scaffolded(id: &str, name: &str, route: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            route: route.to_string(),
            status: ScaffoldStatus::Scaffolded,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AppOverview, ScaffoldStatus};
    use crate::domain::{AudioAssetKind, RecipeWorkflow};
    use crate::fixtures::{composition_fixture, fixture_set, project_fixture};
    use crate::storage::StoragePathAllocator;

    #[test]
    fn baseline_contains_all_initial_studio_surfaces() {
        let overview = AppOverview::baseline();
        let studio_ids: Vec<&str> = overview
            .studios
            .iter()
            .map(|studio| studio.id.as_str())
            .collect();

        assert_eq!(
            studio_ids,
            vec![
                "tts",
                "voice-lab",
                "sfx",
                "loops",
                "songs",
                "review",
                "rights-safety",
                "composition-editor",
                "video-to-audio"
            ]
        );
    }

    #[test]
    fn baseline_marks_current_architecture_layers() {
        let overview = AppOverview::baseline();

        assert!(overview
            .architecture
            .layers
            .iter()
            .any(|layer| layer.id == "react-ui" && layer.status == ScaffoldStatus::Scaffolded));
        assert!(overview.architecture.layers.iter().any(|layer| {
            layer.id == "worker-runtime" && layer.status == ScaffoldStatus::Scaffolded
        }));
    }

    #[test]
    fn baseline_serializes_for_tauri_boundary() {
        let payload = serde_json::to_value(AppOverview::baseline()).expect("baseline serializes");

        assert_eq!(payload["productName"], "SoundWorks");
        assert_eq!(payload["commands"][0]["name"], "get_app_overview");
        assert_eq!(payload["commands"][2]["name"], "get_workspace_overview");
        assert_eq!(payload["commands"][3]["name"], "get_asset_library_overview");
        assert_eq!(
            payload["commands"][4]["name"],
            "get_export_workflow_overview"
        );
        assert_eq!(
            payload["commands"][5]["name"],
            "get_composition_editor_overview"
        );
        assert_eq!(payload["commands"][6]["name"], "get_runtime_overview");
        assert_eq!(
            payload["commands"][7]["name"],
            "get_model_evaluation_catalog"
        );
        assert_eq!(
            payload["commands"][8]["name"],
            "get_mvp_validation_overview"
        );
        assert_eq!(payload["workspace"]["projectAssetCount"], 10);
        assert_eq!(payload["workspace"]["globalAssetCount"], 3);
        assert_eq!(payload["providerCatalog"]["capabilityCount"], 14);
        assert_eq!(payload["assetLibrary"]["supportedTypeCount"], 13);
        assert_eq!(payload["exportWorkflow"]["presetCount"], 7);
        assert_eq!(payload["compositionEditor"]["trackCount"], 4);
        assert_eq!(
            payload["compositionEditor"]["recommendedComponentId"],
            "waveform-playlist"
        );
        assert_eq!(payload["mvpValidation"]["demoWorkflowCount"], 12);
        assert_eq!(payload["mvpValidation"]["blockingItemCount"], 6);
        assert_eq!(payload["mvpValidation"]["readyForMvp"], false);
        assert_eq!(payload["modelEvaluation"]["candidateCount"], 28);
        assert_eq!(payload["ttsStudio"]["segmentCount"], 3);
        assert_eq!(payload["commands"][9]["name"], "get_tts_studio_overview");
        assert_eq!(payload["commands"][10]["name"], "get_voice_lab_overview");
        assert_eq!(payload["commands"][11]["name"], "get_sfx_studio_overview");
        assert_eq!(payload["commands"][13]["name"], "get_song_studio_overview");
        assert_eq!(
            payload["commands"][14]["name"],
            "get_review_workspace_overview"
        );
        assert_eq!(
            payload["commands"][15]["name"],
            "get_rights_safety_overview"
        );
        assert_eq!(
            payload["commands"][16]["name"],
            "get_video_to_audio_overview"
        );
        assert_eq!(payload["voiceLab"]["modeCount"], 3);
        assert_eq!(payload["sfxStudio"]["variantCount"], 3);
        assert_eq!(payload["songStudio"]["variantCount"], 2);
        assert_eq!(payload["reviewWorkspace"]["editActionCount"], 8);
        assert_eq!(payload["videoToAudio"]["targetRangeCount"], 3);
        assert_eq!(payload["videoToAudio"]["syncPointCount"], 5);
    }

    #[test]
    fn fixtures_cover_major_generated_asset_types() {
        let fixtures = fixture_set().expect("fixtures allocate storage paths");
        let asset_kinds: Vec<AudioAssetKind> =
            fixtures.iter().map(|fixture| fixture.asset.kind).collect();
        let workflows: Vec<RecipeWorkflow> = fixtures
            .iter()
            .map(|fixture| fixture.recipe.workflow)
            .collect();

        assert_eq!(
            asset_kinds,
            vec![
                AudioAssetKind::VoiceClip,
                AudioAssetKind::Sfx,
                AudioAssetKind::InstrumentSample,
                AudioAssetKind::Loop,
                AudioAssetKind::Song,
            ]
        );
        assert_eq!(
            workflows,
            vec![
                RecipeWorkflow::Tts,
                RecipeWorkflow::Sfx,
                RecipeWorkflow::InstrumentSample,
                RecipeWorkflow::Loop,
                RecipeWorkflow::Song,
            ]
        );
    }

    #[test]
    fn fixture_recipes_are_replayable_and_serializable() {
        for fixture in fixture_set().expect("fixtures allocate storage paths") {
            let summary = fixture.recipe.inspectable_summary();
            let payload = serde_json::to_value(&fixture.recipe).expect("recipe serializes");

            assert!(summary.replayable);
            assert_eq!(summary.output_asset_count, 1);
            assert!(payload["provider"]["modelVersion"].is_string());
            assert_eq!(fixture.job.output_version_ids, vec![fixture.version.id]);
        }
    }

    #[test]
    fn storage_paths_are_versioned_and_collision_resistant() {
        let fixtures = fixture_set().expect("fixtures allocate storage paths");
        let first_path = fixtures[0].version.file.storage_path.clone();
        let second_path = fixtures[1].version.file.storage_path.clone();

        assert_ne!(first_path, second_path);
        assert!(first_path.contains("/asset-voice-001/version-voice-001-a/media.wav"));
        assert!(fixtures[0]
            .version
            .waveform_preview_cache
            .as_deref()
            .expect("waveform preview")
            .contains("/previews/waveform.json"));
    }

    #[test]
    fn storage_allocator_rejects_unsafe_segments() {
        let allocator = StoragePathAllocator::new("soundworks-library");
        let error = allocator
            .allocate_asset_version(
                &crate::domain::LibraryScope::GlobalLibrary,
                AudioAssetKind::Sfx,
                "../asset",
                "version-1",
                crate::domain::AudioFileFormat::Wav,
            )
            .expect_err("unsafe path rejected");

        assert_eq!(
            error,
            crate::storage::StoragePathError::UnsafeSegment("../asset".to_string())
        );
    }

    #[test]
    fn project_and_composition_fixtures_capture_timeline_state() {
        let project = project_fixture();
        let composition = composition_fixture();
        let timeline_payload = serde_json::to_value(&composition).expect("composition serializes");

        assert_eq!(project.composition_ids, vec!["composition-demo"]);
        assert_eq!(composition.tracks.len(), 2);
        assert_eq!(composition.tracks[0].clips[0].source_range.start_ms, 250);
        assert_eq!(composition.tracks[0].clips[0].fade_out_ms, 80);
        assert_eq!(
            timeline_payload["tracks"][0]["automation"][0]["target"],
            "gain"
        );
        assert_eq!(
            timeline_payload["exportHistory"][0]["presetId"],
            "preset-sceneworks-video-track"
        );
    }

    #[test]
    fn app_overview_summarizes_provider_capabilities() {
        let overview = AppOverview::baseline();

        assert_eq!(overview.provider_catalog.provider_count, 2);
        assert_eq!(overview.provider_catalog.model_count, 4);
        assert_eq!(overview.provider_catalog.capability_count, 14);
        assert_eq!(
            overview.provider_catalog.workflows.len(),
            crate::manifests::CapabilityWorkflow::all().len()
        );
        assert!(overview
            .provider_catalog
            .workflows
            .iter()
            .any(|workflow| workflow.default_model_id == "reference-generation-suite"));
    }

    #[test]
    fn app_overview_summarizes_model_evaluation() {
        let overview = AppOverview::baseline();

        assert_eq!(overview.model_evaluation.schema_version, 1);
        assert_eq!(overview.model_evaluation.candidate_count, 28);
        assert_eq!(overview.model_manager.candidate_count, 28);
        assert!(overview.model_manager.verified_installed_count <= 28);
        assert!(overview
            .model_evaluation
            .recommended_candidate_ids
            .contains(&"moss-soundeffect".to_string()));
    }

    #[test]
    fn app_overview_summarizes_mvp_validation_gate() {
        let overview = AppOverview::baseline();

        assert_eq!(overview.mvp_validation.workflow_count, 12);
        assert_eq!(overview.mvp_validation.demo_workflow_count, 12);
        assert_eq!(overview.mvp_validation.regression_fixture_count, 12);
        assert_eq!(overview.mvp_validation.stress_case_count, 8);
        assert_eq!(overview.mvp_validation.known_limitation_count, 4);
        assert!(!overview.mvp_validation.ready_for_mvp);
    }

    #[test]
    fn app_overview_summarizes_tts_studio() {
        let overview = AppOverview::baseline();

        assert_eq!(overview.studios[0].status, ScaffoldStatus::Scaffolded);
        assert_eq!(overview.tts_studio.segment_count, 3);
        assert_eq!(overview.tts_studio.speaker_count, 2);
        assert_eq!(
            overview.tts_studio.saved_asset_kind,
            AudioAssetKind::VoiceClip
        );
        assert!(!overview.tts_studio.can_submit);
    }

    #[test]
    fn app_overview_summarizes_voice_lab() {
        let overview = AppOverview::baseline();

        assert_eq!(overview.studios[1].status, ScaffoldStatus::Scaffolded);
        assert_eq!(overview.voice_lab.mode_count, 3);
        assert_eq!(overview.voice_lab.profile_count, 2);
        assert_eq!(overview.voice_lab.provider_count, 8);
        assert_eq!(overview.voice_lab.selected_conversion_candidate_id, "rvc");
        assert_eq!(
            overview.voice_lab.saved_asset_kind,
            AudioAssetKind::VoiceClip
        );
        assert!(overview.voice_lab.can_submit_conversion);
    }

    #[test]
    fn app_overview_summarizes_sfx_studio() {
        let overview = AppOverview::baseline();

        assert_eq!(overview.studios[2].status, ScaffoldStatus::Scaffolded);
        assert_eq!(overview.sfx_studio.variant_count, 3);
        assert_eq!(overview.sfx_studio.saved_output_count, 2);
        assert_eq!(overview.sfx_studio.provider_count, 2);
        assert_eq!(overview.sfx_studio.scorecard_count, 9);
        assert_eq!(
            overview.sfx_studio.saved_asset_kinds,
            vec![AudioAssetKind::Ambience, AudioAssetKind::Sfx]
        );
        assert!(!overview.sfx_studio.can_submit);
    }

    #[test]
    fn app_overview_summarizes_video_to_audio() {
        let overview = AppOverview::baseline();

        assert_eq!(overview.studios[8].status, ScaffoldStatus::Scaffolded);
        assert_eq!(overview.video_to_audio.target_range_count, 3);
        assert_eq!(overview.video_to_audio.detected_event_count, 5);
        assert_eq!(overview.video_to_audio.sync_point_count, 5);
        assert_eq!(overview.video_to_audio.provider_count, 1);
        assert_eq!(overview.video_to_audio.scorecard_count, 4);
        assert_eq!(
            overview.video_to_audio.saved_asset_kind,
            AudioAssetKind::Sfx
        );
        assert!(overview.video_to_audio.can_submit);
    }

    #[test]
    fn app_overview_summarizes_project_workspace() {
        let overview = AppOverview::baseline();

        assert_eq!(overview.workspace.schema_version, 1);
        assert_eq!(overview.workspace.active_project_id, "project-demo");
        assert_eq!(overview.workspace.global_library_id, "global-library");
        assert_eq!(overview.workspace.project_count, 2);
        assert_eq!(overview.workspace.project_asset_count, 10);
        assert_eq!(overview.workspace.global_asset_count, 3);
        assert_eq!(overview.workspace.linked_global_asset_count, 1);
        assert_eq!(overview.workspace.transfer_action_count, 3);
        assert_eq!(overview.workspace.source_picker_target_count, 5);
        assert!(overview.workspace.can_create_project);
        assert!(overview.workspace.can_open_project);
    }

    #[test]
    fn app_overview_summarizes_samples_studio() {
        let overview = AppOverview::baseline();

        assert_eq!(overview.studios[3].status, ScaffoldStatus::Scaffolded);
        assert_eq!(overview.samples_studio.variant_count, 4);
        assert_eq!(overview.samples_studio.saved_output_count, 3);
        assert_eq!(overview.samples_studio.provider_count, 4);
        assert_eq!(overview.samples_studio.scorecard_count, 5);
        assert_eq!(
            overview.samples_studio.saved_asset_kinds,
            vec![AudioAssetKind::InstrumentSample, AudioAssetKind::Loop]
        );
        assert!(!overview.samples_studio.can_submit);
    }

    #[test]
    fn app_overview_summarizes_song_studio() {
        let overview = AppOverview::baseline();

        assert_eq!(overview.studios[4].status, ScaffoldStatus::Scaffolded);
        assert_eq!(overview.song_studio.section_count, 4);
        assert_eq!(overview.song_studio.variant_count, 2);
        assert_eq!(overview.song_studio.saved_output_count, 2);
        assert_eq!(overview.song_studio.provider_count, 1);
        assert_eq!(overview.song_studio.scorecard_count, 8);
        assert_eq!(
            overview.song_studio.saved_asset_kinds,
            vec![AudioAssetKind::MusicClip, AudioAssetKind::Song]
        );
        assert!(!overview.song_studio.can_submit);
    }

    #[test]
    fn app_overview_summarizes_review_workspace() {
        let overview = AppOverview::baseline();

        assert_eq!(overview.studios[5].status, ScaffoldStatus::Scaffolded);
        assert_eq!(overview.review_workspace.asset_count, 5);
        assert_eq!(overview.review_workspace.previewable_asset_count, 5);
        assert_eq!(overview.review_workspace.edit_action_count, 8);
        assert_eq!(overview.review_workspace.comparison_count, 1);
        assert_eq!(
            overview.review_workspace.edited_version_id,
            "version-loop-001-b-review-edit"
        );
        assert!(overview.review_workspace.can_save_edit);
    }

    #[test]
    fn app_overview_summarizes_composition_editor() {
        let overview = AppOverview::baseline();

        assert_eq!(overview.studios[7].status, ScaffoldStatus::Scaffolded);
        assert_eq!(overview.composition_editor.track_count, 4);
        assert_eq!(overview.composition_editor.clip_count, 7);
        assert_eq!(overview.composition_editor.asset_bin_count, 5);
        assert_eq!(overview.composition_editor.enabled_tool_count, 9);
        assert_eq!(
            overview.composition_editor.recommended_component_id,
            "waveform-playlist"
        );
        assert!(overview.composition_editor.can_render_mixdown);
        assert!(overview
            .composition_editor
            .editable_asset_kinds
            .contains(&AudioAssetKind::VoiceClip));
        assert!(overview
            .composition_editor
            .editable_asset_kinds
            .contains(&AudioAssetKind::Stem));
    }
}
