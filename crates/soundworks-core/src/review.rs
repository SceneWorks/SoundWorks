use crate::domain::{
    AssetCreation, AudioAsset, AudioAssetKind, AudioAssetVersion, AudioFileFormat,
    AudioFileReference, EditRecipe, GenerationJob, GenerationRecipe, JobKind, JobStatus,
    ModelDescriptor, ModelRuntime, PostProcessingOperation, PostProcessingStep, RecipeRequest,
    RecipeSummary, RecipeWorkflow, SourceReference, SourceReferenceType, TechnicalAudioMetadata,
    TimeRange,
};
use crate::fixtures::fixture_set;
use crate::storage::{StoragePathAllocator, StoragePathError};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;

pub const REVIEW_WORKSPACE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewWorkspaceOverview {
    pub schema_version: u32,
    pub assets: Vec<ReviewAssetPreview>,
    pub selected_asset: ReviewAssetPreview,
    pub transport: WaveformTransportState,
    pub waveform: WaveformPreview,
    pub spectrogram: SpectrogramPreview,
    pub edit_actions: Vec<LightweightEditAction>,
    pub edit_submission: EditSubmissionPreview,
    pub version_comparison: VersionComparison,
    pub provenance: ReviewProvenance,
    pub validation_checks: Vec<ReviewValidationCheck>,
}

impl ReviewWorkspaceOverview {
    pub fn reference() -> Result<Self, StoragePathError> {
        let fixtures = fixture_set()?;
        let allocator = StoragePathAllocator::new("soundworks-library");
        let source_fixture = fixtures
            .iter()
            .find(|fixture| fixture.asset.kind == AudioAssetKind::Loop)
            .expect("fixtures include loop asset");
        let edited_paths = allocator.allocate_asset_version(
            &source_fixture.asset.scope,
            source_fixture.asset.kind,
            &source_fixture.asset.id,
            "version-loop-001-b-review-edit",
            AudioFileFormat::Wav,
        )?;
        let edit_actions = edit_actions();
        let edit_recipe = edit_recipe(source_fixture, &edit_actions);
        let edit_job = GenerationJob {
            id: "job-review-edit-loop-001".to_string(),
            recipe_id: edit_recipe.id.clone(),
            kind: JobKind::GenerateAudio,
            status: JobStatus::Succeeded,
            progress: None,
            output_version_ids: vec!["version-loop-001-b-review-edit".to_string()],
            error: None,
        };
        let edited_version = edited_version(source_fixture, edited_paths);
        let mut selected_asset = source_fixture.asset.clone();
        selected_asset.current_version_id = edited_version.id.clone();
        selected_asset.version_ids.push(edited_version.id.clone());
        selected_asset
            .provenance_ids
            .push("provenance-review-edit-loop-001".to_string());

        let assets = fixtures
            .iter()
            .map(|fixture| ReviewAssetPreview {
                asset: fixture.asset.clone(),
                versions: vec![fixture.version.clone()],
                source_workflow: fixture.recipe.workflow,
                can_preview: fixture.version.waveform_preview_cache.is_some()
                    && fixture.version.spectrogram_preview_cache.is_some(),
                preview_status: PreviewCacheStatus::Ready,
            })
            .collect::<Vec<_>>();
        let selected_preview = ReviewAssetPreview {
            asset: selected_asset.clone(),
            versions: vec![source_fixture.version.clone(), edited_version.clone()],
            source_workflow: source_fixture.recipe.workflow,
            can_preview: true,
            preview_status: PreviewCacheStatus::Ready,
        };
        let sidecar_path = edited_version
            .spectrogram_preview_cache
            .as_deref()
            .unwrap_or_default()
            .replace(
                "/previews/spectrogram.bin",
                "/metadata/recipe-provenance.json",
            );

        Ok(Self {
            schema_version: REVIEW_WORKSPACE_SCHEMA_VERSION,
            assets,
            selected_asset: selected_preview,
            transport: WaveformTransportState {
                playing: false,
                position_ms: 3_200,
                duration_ms: source_fixture.version.technical.duration_ms,
                zoom_pixels_per_second: 92,
                selection: Some(TimeRange {
                    start_ms: 640,
                    end_ms: 10_480,
                }),
                loop_region: Some(TimeRange {
                    start_ms: 0,
                    end_ms: source_fixture.version.technical.duration_ms,
                }),
                keyboard_shortcuts: vec![
                    KeyboardShortcut {
                        id: "transport.play_pause".to_string(),
                        keys: "Space".to_string(),
                        action: "Play or pause preview".to_string(),
                    },
                    KeyboardShortcut {
                        id: "transport.seek_backward".to_string(),
                        keys: "ArrowLeft".to_string(),
                        action: "Seek backward".to_string(),
                    },
                    KeyboardShortcut {
                        id: "transport.scrub".to_string(),
                        keys: "Shift+Drag".to_string(),
                        action: "Scrub waveform selection".to_string(),
                    },
                    KeyboardShortcut {
                        id: "transport.zoom".to_string(),
                        keys: "Command+Plus / Command+Minus".to_string(),
                        action: "Zoom waveform".to_string(),
                    },
                ],
                accessible_labels: vec![
                    "Play or pause waveform preview".to_string(),
                    "Seek through selected audio asset".to_string(),
                    "Adjust loop region start and end".to_string(),
                    "Zoom waveform timeline".to_string(),
                ],
            },
            waveform: WaveformPreview {
                asset_version_id: source_fixture.version.id.clone(),
                channel_count: source_fixture.version.technical.channels,
                sample_rate_hz: source_fixture.version.technical.sample_rate_hz,
                duration_ms: source_fixture.version.technical.duration_ms,
                cache_path: source_fixture
                    .version
                    .waveform_preview_cache
                    .clone()
                    .expect("fixture has waveform cache"),
                status: PreviewCacheStatus::Ready,
                peaks: waveform_peaks(),
            },
            spectrogram: SpectrogramPreview {
                asset_version_id: source_fixture.version.id.clone(),
                cache_path: source_fixture
                    .version
                    .spectrogram_preview_cache
                    .clone()
                    .expect("fixture has spectrogram cache"),
                status: PreviewCacheStatus::Ready,
                frequency_bins: 256,
                time_slices: 128,
            },
            edit_actions,
            edit_submission: EditSubmissionPreview {
                id: "review-edit-loop-001".to_string(),
                can_save: true,
                recipe: edit_recipe.clone(),
                job: edit_job,
                source_asset: source_fixture.asset.clone(),
                source_version: source_fixture.version.clone(),
                saved_asset: selected_asset.clone(),
                saved_version: edited_version.clone(),
                warnings: vec![
                    "Normalize target is stored as recipe metadata before media mutation."
                        .to_string(),
                ],
                blocking_reasons: vec![],
            },
            version_comparison: VersionComparison {
                id: "compare-loop-001-a-b".to_string(),
                mode: CompareMode::VersionAb,
                left: ComparableVersion {
                    label: "Original loop".to_string(),
                    asset_id: source_fixture.asset.id.clone(),
                    version_id: source_fixture.version.id.clone(),
                    recipe_id: source_fixture.recipe.id.clone(),
                    duration_ms: source_fixture.version.technical.duration_ms,
                    loudness_lufs: source_fixture.version.technical.loudness_lufs,
                    true_peak_dbfs: source_fixture.version.technical.true_peak_dbfs,
                },
                right: ComparableVersion {
                    label: "Edited loop".to_string(),
                    asset_id: selected_asset.id.clone(),
                    version_id: edited_version.id.clone(),
                    recipe_id: edit_recipe.id.clone(),
                    duration_ms: edited_version.technical.duration_ms,
                    loudness_lufs: edited_version.technical.loudness_lufs,
                    true_peak_dbfs: edited_version.technical.true_peak_dbfs,
                },
                metrics: ComparisonMetrics {
                    duration_delta_ms: -683,
                    loudness_delta_lufs: Some(-2.0),
                    true_peak_delta_db: Some(-0.3),
                    waveform_difference_score: 18,
                },
                notes: vec![
                    "A/B compare can target two versions of one asset.".to_string(),
                    "The same contract accepts generated variants from different assets.".to_string(),
                ],
            },
            provenance: ReviewProvenance {
                inspectable: true,
                original_recipe: source_fixture.recipe.inspectable_summary(),
                edit_recipe: edit_recipe.inspectable_summary(),
                source_version_id: source_fixture.version.id.clone(),
                edited_version_id: edited_version.id.clone(),
                provenance_ids: selected_asset.provenance_ids.clone(),
                sidecar_path,
            },
            validation_checks: vec![
                ReviewValidationCheck::passed(
                    "review.preview_all_generated_assets",
                    "Every generated fixture asset kind exposes waveform and spectrogram preview cache paths.",
                ),
                ReviewValidationCheck::passed(
                    "review.transport_accessibility",
                    "Transport includes play, pause, seek, scrub, zoom, loop region, time display, keyboard shortcuts, and accessible labels.",
                ),
                ReviewValidationCheck::passed(
                    "review.non_destructive_edit",
                    "Save creates a new edited version and preserves the original generated version.",
                ),
                ReviewValidationCheck::passed(
                    "review.provenance_recipe",
                    "Edit recipe, source version, generated source recipe, and provenance sidecar remain inspectable.",
                ),
            ],
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewAssetPreview {
    pub asset: AudioAsset,
    pub versions: Vec<AudioAssetVersion>,
    pub source_workflow: RecipeWorkflow,
    pub can_preview: bool,
    pub preview_status: PreviewCacheStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WaveformTransportState {
    pub playing: bool,
    pub position_ms: u64,
    pub duration_ms: u64,
    pub zoom_pixels_per_second: u32,
    pub selection: Option<TimeRange>,
    pub loop_region: Option<TimeRange>,
    pub keyboard_shortcuts: Vec<KeyboardShortcut>,
    pub accessible_labels: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyboardShortcut {
    pub id: String,
    pub keys: String,
    pub action: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WaveformPreview {
    pub asset_version_id: String,
    pub channel_count: u16,
    pub sample_rate_hz: u32,
    pub duration_ms: u64,
    pub cache_path: String,
    pub status: PreviewCacheStatus,
    pub peaks: Vec<WaveformPeak>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WaveformPeak {
    pub min: f32,
    pub max: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpectrogramPreview {
    pub asset_version_id: String,
    pub cache_path: String,
    pub status: PreviewCacheStatus,
    pub frequency_bins: u16,
    pub time_slices: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PreviewCacheStatus {
    Ready,
    Pending,
    Missing,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LightweightEditAction {
    pub id: String,
    pub kind: EditActionKind,
    pub label: String,
    pub operation: Option<PostProcessingOperation>,
    pub destructive: bool,
    pub non_destructive_save: bool,
    pub enabled: bool,
    pub parameters: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EditActionKind {
    Trim,
    FadeIn,
    FadeOut,
    Normalize,
    RemoveSilence,
    LoopCrossfade,
    ConvertFormat,
    EditMetadata,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditSubmissionPreview {
    pub id: String,
    pub can_save: bool,
    pub recipe: GenerationRecipe,
    pub job: GenerationJob,
    pub source_asset: AudioAsset,
    pub source_version: AudioAssetVersion,
    pub saved_asset: AudioAsset,
    pub saved_version: AudioAssetVersion,
    pub warnings: Vec<String>,
    pub blocking_reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionComparison {
    pub id: String,
    pub mode: CompareMode,
    pub left: ComparableVersion,
    pub right: ComparableVersion,
    pub metrics: ComparisonMetrics,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CompareMode {
    VersionAb,
    VariantAb,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComparableVersion {
    pub label: String,
    pub asset_id: String,
    pub version_id: String,
    pub recipe_id: String,
    pub duration_ms: u64,
    pub loudness_lufs: Option<f32>,
    pub true_peak_dbfs: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComparisonMetrics {
    pub duration_delta_ms: i64,
    pub loudness_delta_lufs: Option<f32>,
    pub true_peak_delta_db: Option<f32>,
    pub waveform_difference_score: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewProvenance {
    pub inspectable: bool,
    pub original_recipe: RecipeSummary,
    pub edit_recipe: RecipeSummary,
    pub source_version_id: String,
    pub edited_version_id: String,
    pub provenance_ids: Vec<String>,
    pub sidecar_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewValidationCheck {
    pub id: String,
    pub status: ReviewValidationStatus,
    pub summary: String,
}

impl ReviewValidationCheck {
    fn passed(id: &str, summary: &str) -> Self {
        Self {
            id: id.to_string(),
            status: ReviewValidationStatus::Passed,
            summary: summary.to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReviewValidationStatus {
    Passed,
    Warning,
    Failed,
}

fn edit_recipe(
    fixture: &crate::fixtures::AssetFixture,
    edit_actions: &[LightweightEditAction],
) -> GenerationRecipe {
    GenerationRecipe {
        id: "recipe-review-edit-loop-001".to_string(),
        workflow: RecipeWorkflow::Edit,
        provider: ModelDescriptor {
            provider_id: "soundworks-reference".to_string(),
            model_id: "reference-editor".to_string(),
            model_version: Some("0.1.0".to_string()),
            model_hash: Some("sha256:reference-editor".to_string()),
            runtime: ModelRuntime::Local,
        },
        request: RecipeRequest::Edit(EditRecipe {
            source_asset_version_id: fixture.version.id.clone(),
            edit_instructions:
                "Trim selection, fade edges, remove silence, crossfade loop, normalize loudness, convert format, and update metadata."
                    .to_string(),
            trim: Some(TimeRange {
                start_ms: 640,
                end_ms: 10_480,
            }),
            normalize_loudness_lufs: Some(-16.0),
        }),
        seed: None,
        source_references: vec![SourceReference {
            id: "source-loop-version-001-a".to_string(),
            source_type: SourceReferenceType::Audio,
            asset_id: Some(fixture.asset.id.clone()),
            external_uri: None,
            ownership_note: Some("Original generated loop remains immutable.".to_string()),
        }],
        post_processing: edit_actions
            .iter()
            .filter_map(|action| {
                action.operation.map(|operation| PostProcessingStep {
                    id: format!("step-{}", action.id),
                    operation,
                    parameters: action.parameters.clone(),
                })
            })
            .collect(),
        parameter_overrides: BTreeMap::from([
            ("saveMode".to_string(), json!("new-version")),
            ("preserveSourceVersion".to_string(), json!(true)),
        ]),
        output_asset_ids: vec![fixture.asset.id.clone()],
    }
}

fn edited_version(
    fixture: &crate::fixtures::AssetFixture,
    paths: crate::storage::StoragePaths,
) -> AudioAssetVersion {
    let mut technical: TechnicalAudioMetadata = fixture.version.technical.clone();
    technical.duration_ms = 10_480;
    technical.loudness_lufs = Some(-16.0);
    technical.true_peak_dbfs = Some(-1.8);

    AudioAssetVersion {
        id: "version-loop-001-b-review-edit".to_string(),
        asset_id: fixture.asset.id.clone(),
        version_index: 2,
        file: AudioFileReference {
            storage_path: paths.media_path,
            format: AudioFileFormat::Wav,
            codec: Some("pcm_s16le".to_string()),
            byte_size: None,
            content_hash: Some("sha256:review-edit-loop-001".to_string()),
        },
        technical,
        created_by: AssetCreation::Edited {
            source_version_id: fixture.version.id.clone(),
            edit_recipe_id: "recipe-review-edit-loop-001".to_string(),
        },
        waveform_preview_cache: Some(paths.waveform_preview_path),
        spectrogram_preview_cache: Some(paths.spectrogram_preview_path),
    }
}

fn edit_actions() -> Vec<LightweightEditAction> {
    vec![
        edit_action(
            "trim-selection",
            EditActionKind::Trim,
            "Trim selection",
            Some(PostProcessingOperation::Trim),
            BTreeMap::from([
                ("startMs".to_string(), json!(640)),
                ("endMs".to_string(), json!(10_480)),
            ]),
        ),
        edit_action(
            "fade-in",
            EditActionKind::FadeIn,
            "Fade in",
            Some(PostProcessingOperation::Fade),
            BTreeMap::from([("durationMs".to_string(), json!(80))]),
        ),
        edit_action(
            "fade-out",
            EditActionKind::FadeOut,
            "Fade out",
            Some(PostProcessingOperation::Fade),
            BTreeMap::from([("durationMs".to_string(), json!(180))]),
        ),
        edit_action(
            "normalize",
            EditActionKind::Normalize,
            "Normalize loudness",
            Some(PostProcessingOperation::Normalize),
            BTreeMap::from([("targetLufs".to_string(), json!(-16.0))]),
        ),
        edit_action(
            "remove-silence",
            EditActionKind::RemoveSilence,
            "Remove silence",
            Some(PostProcessingOperation::RemoveSilence),
            BTreeMap::from([("thresholdDb".to_string(), json!(-48.0))]),
        ),
        edit_action(
            "loop-crossfade",
            EditActionKind::LoopCrossfade,
            "Loop crossfade",
            Some(PostProcessingOperation::LoopCrossfade),
            BTreeMap::from([("durationMs".to_string(), json!(120))]),
        ),
        edit_action(
            "convert-format",
            EditActionKind::ConvertFormat,
            "Convert format",
            Some(PostProcessingOperation::ConvertFormat),
            BTreeMap::from([("format".to_string(), json!("wav"))]),
        ),
        edit_action(
            "metadata-edit",
            EditActionKind::EditMetadata,
            "Edit metadata",
            Some(PostProcessingOperation::EditMetadata),
            BTreeMap::from([
                ("tag".to_string(), json!("reviewed")),
                ("collection".to_string(), json!("Demo SoundWorks Project")),
            ]),
        ),
    ]
}

fn edit_action(
    id: &str,
    kind: EditActionKind,
    label: &str,
    operation: Option<PostProcessingOperation>,
    parameters: BTreeMap<String, serde_json::Value>,
) -> LightweightEditAction {
    LightweightEditAction {
        id: id.to_string(),
        kind,
        label: label.to_string(),
        operation,
        destructive: true,
        non_destructive_save: true,
        enabled: true,
        parameters,
    }
}

fn waveform_peaks() -> Vec<WaveformPeak> {
    [
        (0.08, 0.32),
        (0.14, 0.56),
        (0.2, 0.64),
        (0.11, 0.48),
        (0.22, 0.78),
        (0.17, 0.72),
        (0.28, 0.86),
        (0.19, 0.62),
        (0.12, 0.44),
        (0.24, 0.8),
        (0.18, 0.7),
        (0.3, 0.88),
        (0.2, 0.68),
        (0.1, 0.4),
        (0.16, 0.58),
        (0.25, 0.82),
    ]
    .into_iter()
    .map(|(min, max)| WaveformPeak { min: -min, max })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::{EditActionKind, PreviewCacheStatus, ReviewWorkspaceOverview};
    use crate::domain::{AssetCreation, AudioAssetKind, RecipeWorkflow};
    use std::collections::BTreeSet;

    #[test]
    fn reference_workspace_covers_generated_audio_asset_kinds() {
        let overview = ReviewWorkspaceOverview::reference().expect("review workspace builds");
        let kinds = overview
            .assets
            .iter()
            .map(|asset| asset.asset.kind)
            .collect::<Vec<_>>();

        assert_eq!(
            kinds,
            vec![
                AudioAssetKind::VoiceClip,
                AudioAssetKind::Sfx,
                AudioAssetKind::InstrumentSample,
                AudioAssetKind::Loop,
                AudioAssetKind::Song,
            ]
        );
        assert!(overview.assets.iter().all(|asset| asset.can_preview));
        assert!(overview
            .assets
            .iter()
            .all(|asset| asset.preview_status == PreviewCacheStatus::Ready));
    }

    #[test]
    fn transport_covers_review_controls_and_accessibility() {
        let overview = ReviewWorkspaceOverview::reference().expect("review workspace builds");
        let shortcut_ids = overview
            .transport
            .keyboard_shortcuts
            .iter()
            .map(|shortcut| shortcut.id.as_str())
            .collect::<BTreeSet<_>>();

        for shortcut in [
            "transport.play_pause",
            "transport.seek_backward",
            "transport.scrub",
            "transport.zoom",
        ] {
            assert!(shortcut_ids.contains(shortcut));
        }
        assert!(overview.transport.loop_region.is_some());
        assert!(overview.transport.selection.is_some());
        assert!(!overview.transport.accessible_labels.is_empty());
        assert_eq!(
            overview.transport.duration_ms,
            overview.waveform.duration_ms
        );
    }

    #[test]
    fn edit_actions_cover_required_lightweight_surface() {
        let overview = ReviewWorkspaceOverview::reference().expect("review workspace builds");
        let actions = overview
            .edit_actions
            .iter()
            .map(|action| action.kind)
            .collect::<BTreeSet<_>>();

        for action in [
            EditActionKind::Trim,
            EditActionKind::FadeIn,
            EditActionKind::FadeOut,
            EditActionKind::Normalize,
            EditActionKind::RemoveSilence,
            EditActionKind::LoopCrossfade,
            EditActionKind::ConvertFormat,
            EditActionKind::EditMetadata,
        ] {
            assert!(actions.contains(&action));
        }
        assert!(overview
            .edit_actions
            .iter()
            .all(|action| action.enabled && action.non_destructive_save));
    }

    #[test]
    fn saving_edit_appends_version_without_destroying_source() {
        let overview = ReviewWorkspaceOverview::reference().expect("review workspace builds");
        let submission = overview.edit_submission;

        assert!(submission.can_save);
        assert_eq!(
            submission.recipe.workflow,
            RecipeWorkflow::Edit,
            "edit save is represented as a recipe"
        );
        assert_eq!(submission.source_version.version_index, 1);
        assert_eq!(submission.saved_version.version_index, 2);
        assert_eq!(
            submission.saved_asset.version_ids,
            vec![
                submission.source_version.id.clone(),
                submission.saved_version.id.clone()
            ]
        );
        assert!(matches!(
            submission.saved_version.created_by,
            AssetCreation::Edited { .. }
        ));
    }

    #[test]
    fn comparison_and_provenance_remain_inspectable() {
        let overview = ReviewWorkspaceOverview::reference().expect("review workspace builds");

        assert_ne!(
            overview.version_comparison.left.version_id,
            overview.version_comparison.right.version_id
        );
        assert!(
            overview
                .version_comparison
                .metrics
                .waveform_difference_score
                > 0
        );
        assert!(overview.provenance.inspectable);
        assert_eq!(
            overview.provenance.original_recipe.workflow,
            RecipeWorkflow::Loop
        );
        assert_eq!(
            overview.provenance.edit_recipe.workflow,
            RecipeWorkflow::Edit
        );
        assert_eq!(
            overview.provenance.edited_version_id,
            overview.edit_submission.saved_version.id
        );
        assert!(overview
            .provenance
            .sidecar_path
            .ends_with("recipe-provenance.json"));
    }
}
