use serde::{Deserialize, Serialize};

pub mod domain;
pub mod evaluation;
pub mod fixtures;
pub mod manifests;
pub mod runtime;
pub mod sfx;
pub mod storage;
pub mod tts;
pub mod voice_lab;

pub use domain::*;
pub use evaluation::*;
pub use fixtures::*;
pub use manifests::*;
pub use runtime::*;
pub use sfx::*;
pub use storage::*;
pub use tts::*;
pub use voice_lab::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppOverview {
    pub product_name: String,
    pub architecture: ArchitectureOverview,
    pub studios: Vec<StudioSurface>,
    pub commands: Vec<CommandBoundary>,
    pub provider_catalog: ProviderCatalogOverview,
    pub model_evaluation: ModelEvaluationOverview,
    pub tts_studio: TtsStudioSummary,
    pub voice_lab: VoiceLabSummary,
    pub sfx_studio: SfxStudioSummary,
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
                StudioSurface::planned("loops", "Samples + Loops", "/studios/loops"),
                StudioSurface::planned("songs", "Song Studio", "/studios/songs"),
                StudioSurface::planned(
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
            ],
            provider_catalog: ProviderCatalogOverview::from_catalog(&ProviderCatalog::reference()),
            model_evaluation: ModelEvaluationCatalog::reference().overview(),
            tts_studio: TtsStudioSummary::from_overview(
                &TtsStudioOverview::reference().expect("reference TTS studio is valid"),
            ),
            voice_lab: VoiceLabSummary::from_overview(
                &VoiceLabOverview::reference().expect("reference Voice Lab is valid"),
            ),
            sfx_studio: SfxStudioSummary::from_overview(
                &SfxStudioOverview::reference().expect("reference SFX Studio is valid"),
            ),
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
        Self {
            schema_version: overview.schema_version,
            variant_count: overview.variants.len(),
            saved_output_count: overview.saved_outputs.len(),
            provider_count: overview.provider_options.len(),
            scorecard_count: overview.provider_scorecards.len(),
            can_submit: overview.submission.can_submit,
            selected_provider_id: overview.selected_provider.provider_id.clone(),
            selected_model_id: overview.selected_provider.model_id.clone(),
            saved_asset_kinds: overview
                .saved_outputs
                .iter()
                .map(|output| output.asset.kind)
                .collect(),
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
    fn planned(id: &str, name: &str, route: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            route: route.to_string(),
            status: ScaffoldStatus::Planned,
        }
    }

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
    use crate::storage::{StoragePathAllocator, SCHEMA_MIGRATIONS};

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
        assert_eq!(payload["commands"][2]["name"], "get_runtime_overview");
        assert_eq!(
            payload["commands"][3]["name"],
            "get_model_evaluation_catalog"
        );
        assert_eq!(payload["providerCatalog"]["capabilityCount"], 12);
        assert_eq!(payload["modelEvaluation"]["candidateCount"], 28);
        assert_eq!(payload["ttsStudio"]["segmentCount"], 3);
        assert_eq!(payload["commands"][4]["name"], "get_tts_studio_overview");
        assert_eq!(payload["commands"][5]["name"], "get_voice_lab_overview");
        assert_eq!(payload["commands"][6]["name"], "get_sfx_studio_overview");
        assert_eq!(payload["voiceLab"]["modeCount"], 3);
        assert_eq!(payload["sfxStudio"]["variantCount"], 3);
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
    fn schema_migrations_cover_required_domain_tables() {
        let sql = SCHEMA_MIGRATIONS
            .iter()
            .map(|migration| migration.sql)
            .collect::<Vec<_>>()
            .join("\n");

        for table in [
            "projects",
            "audio_assets",
            "audio_asset_versions",
            "generation_recipes",
            "generation_jobs",
            "voice_profiles",
            "compositions",
            "storage_paths",
            "provider_manifests",
            "model_manifests",
            "model_evaluation_candidates",
            "model_evaluation_fixtures",
            "model_evaluation_recommendations",
            "voice_lab_profiles",
            "voice_lab_reference_clips",
            "voice_lab_provider_scorecards",
            "voice_lab_safety_gates",
            "voice_lab_conversion_submissions",
            "sfx_studio_prompts",
            "sfx_studio_variants",
            "sfx_studio_submissions",
            "sfx_studio_saved_outputs",
            "sfx_studio_provider_scorecards",
            "sfx_studio_post_processing_actions",
        ] {
            assert!(
                sql.contains(table),
                "expected schema migrations to include {table}"
            );
        }
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

        assert_eq!(overview.provider_catalog.provider_count, 1);
        assert_eq!(overview.provider_catalog.model_count, 3);
        assert_eq!(overview.provider_catalog.capability_count, 12);
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
        assert!(overview
            .model_evaluation
            .recommended_candidate_ids
            .contains(&"moss-soundeffect".to_string()));
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
        assert!(overview.tts_studio.can_submit);
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
            vec![AudioAssetKind::Sfx, AudioAssetKind::Ambience]
        );
        assert!(overview.sfx_studio.can_submit);
    }
}
