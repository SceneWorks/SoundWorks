use crate::domain::{AudioAssetKind, AudioFileFormat, ExportPreset, ExportTarget, StemKind};
use crate::fixtures::{composition_fixture, fixture_set};
use crate::rights::{ProvenanceSidecar, RightsSafetyOverview};
use serde::{Deserialize, Serialize};

pub const EXPORT_WORKFLOW_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportWorkflowOverview {
    pub schema_version: u32,
    pub presets: Vec<ExportPresetPlan>,
    pub targets: Vec<ExportTargetReadiness>,
    pub selected_export: ExportSubmissionPreview,
    pub sidecars: Vec<ExportSidecarPreview>,
    pub daw_handoff: DawHandoffPackage,
    pub scene_works_handoff: SceneWorksHandoffPackage,
    pub validation_checks: Vec<ExportValidationCheck>,
}

impl ExportWorkflowOverview {
    pub fn reference() -> Self {
        let fixtures = fixture_set().expect("reference fixtures allocate storage paths");
        let composition = composition_fixture();
        let rights = RightsSafetyOverview::reference();
        let presets = export_presets();
        let sidecars = rights
            .export_sidecars
            .iter()
            .map(ExportSidecarPreview::from_sidecar)
            .collect::<Vec<_>>();

        Self {
            schema_version: EXPORT_WORKFLOW_SCHEMA_VERSION,
            presets: presets.clone(),
            targets: export_targets(),
            selected_export: ExportSubmissionPreview {
                id: "export-demo-composition-sceneworks".to_string(),
                preset_id: "preset-sceneworks-video-track".to_string(),
                source_kind: ExportSourceKind::Composition,
                source_id: composition.id,
                asset_ids: composition
                    .tracks
                    .iter()
                    .flat_map(|track| track.clips.iter().map(|clip| clip.asset_id.clone()))
                    .collect(),
                collection_ids: vec![
                    "collection-neon-bass-pack".to_string(),
                    "collection-demo-song-folder".to_string(),
                ],
                formats: vec![AudioFileFormat::Wav, AudioFileFormat::Flac],
                can_export: true,
                blocking_reasons: vec![],
                warnings: vec![
                    "SceneWorks direct audio import is not present in current source; export writes a package and manifest for the SceneWorks-side importer."
                        .to_string(),
                ],
                output_paths: vec![
                    "soundworks-exports/project-demo/demo-timeline/mixdown.wav".to_string(),
                    "soundworks-exports/project-demo/demo-timeline/stems/track-voice.wav"
                        .to_string(),
                    "soundworks-exports/project-demo/demo-timeline/stems/track-loop.wav"
                        .to_string(),
                    "soundworks-exports/project-demo/demo-timeline/soundworks-export.json"
                        .to_string(),
                ],
                sidecar_path:
                    "soundworks-exports/project-demo/demo-timeline/soundworks-export.json"
                        .to_string(),
            },
            sidecars,
            daw_handoff: DawHandoffPackage {
                id: "daw-demo-timeline-bundle".to_string(),
                preset_id: "preset-daw-stem-bundle".to_string(),
                package_path:
                    "soundworks-exports/project-demo/demo-timeline/demo-timeline-daw.zip"
                        .to_string(),
                normalized_filename_template:
                    "{project}_{asset}_{bpm}bpm_{key}_{version}.{ext}".to_string(),
                includes_zip_bundle: true,
                includes_stems: true,
                includes_cue_markers: true,
                includes_loop_markers: true,
                includes_bpm_key_metadata: true,
                includes_lyrics_text: true,
                includes_midi: false,
                stem_kinds: vec![StemKind::Vocals, StemKind::Drums, StemKind::Bass],
            },
            scene_works_handoff: SceneWorksHandoffPackage {
                id: "sceneworks-demo-video-track".to_string(),
                preset_id: "preset-sceneworks-video-track".to_string(),
                package_path:
                    "soundworks-exports/project-demo/demo-timeline/sceneworks-audio-track.zip"
                        .to_string(),
                rendered_mixdown_path:
                    "soundworks-exports/project-demo/demo-timeline/mixdown.wav".to_string(),
                package_manifest_path:
                    "soundworks-exports/project-demo/demo-timeline/sceneworks-handoff.json"
                        .to_string(),
                provenance_sidecar_path:
                    "soundworks-exports/project-demo/demo-timeline/soundworks-export.json"
                        .to_string(),
                includes_optional_stems: true,
                optional_stem_paths: vec![
                    "soundworks-exports/project-demo/demo-timeline/stems/track-voice.wav"
                        .to_string(),
                    "soundworks-exports/project-demo/demo-timeline/stems/track-loop.wav"
                        .to_string(),
                ],
                import_strategy: SceneWorksImportStrategy::FilePackage,
                attachment_mode: SceneWorksAttachmentMode::AttachOrReplace,
                intended_project_id: Some("sceneworks-project-demo".to_string()),
                intended_video_asset_id: Some("asset_scene_video_airlock".to_string()),
                scene_works_project_path: Some(
                    "SceneWorks/projects/sceneworks-project-demo".to_string(),
                ),
                target_video_sidecar_path: Some(
                    "assets/videos/asset_scene_video_airlock.sceneworks.json".to_string(),
                ),
                scene_works_asset_type: "video".to_string(),
                scene_works_mime_type: "video/mp4".to_string(),
                duration_ms: 11_163,
                target_video_duration_ms: 12_000,
                start_offset_ms: 0,
                sample_rate_hz: 48_000,
                channels: 2,
                loudness_lufs: Some(-16.0),
                true_peak_dbfs: Some(-1.0),
                marker_count: 1,
                section_count: 1,
                replace_existing_audio: true,
                round_trip_recipe_url:
                    "soundworks://project/project-demo/compositions/composition-demo/exports/export-demo-composition-sceneworks"
                        .to_string(),
                source_evidence: scene_works_source_evidence(),
                compatibility_checks: scene_works_compatibility_checks(),
                attachment_steps: scene_works_attachment_steps(),
            },
            validation_checks: validation_checks(&presets, fixtures.len()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportPresetPlan {
    pub preset: ExportPreset,
    pub description: String,
    pub source_kinds: Vec<ExportSourceKind>,
    pub asset_kinds: Vec<AudioAssetKind>,
    pub formats: Vec<AudioFileFormat>,
    pub package_artifacts: Vec<ExportArtifactKind>,
    pub normalize_loudness: bool,
    pub target_lufs: Option<f32>,
    pub preserve_loop_metadata: bool,
    pub preserve_bpm_key_metadata: bool,
    pub writes_sidecar: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExportSourceKind {
    Asset,
    Collection,
    SamplePack,
    LoopPack,
    Song,
    StemBundle,
    Composition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExportArtifactKind {
    AudioFile,
    StemFolder,
    ZipBundle,
    ProviderNative,
    MetadataSidecar,
    CueMarkers,
    LoopMarkers,
    LyricsText,
    SceneWorksPackage,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportTargetReadiness {
    pub target: ExportTarget,
    pub label: String,
    pub ready: bool,
    pub preset_ids: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportSubmissionPreview {
    pub id: String,
    pub preset_id: String,
    pub source_kind: ExportSourceKind,
    pub source_id: String,
    pub asset_ids: Vec<String>,
    pub collection_ids: Vec<String>,
    pub formats: Vec<AudioFileFormat>,
    pub can_export: bool,
    pub blocking_reasons: Vec<String>,
    pub warnings: Vec<String>,
    pub output_paths: Vec<String>,
    pub sidecar_path: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportSidecarPreview {
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
    pub event_count: usize,
}

impl ExportSidecarPreview {
    fn from_sidecar(sidecar: &ProvenanceSidecar) -> Self {
        Self {
            id: sidecar.id.clone(),
            asset_id: sidecar.asset_id.clone(),
            asset_kind: sidecar.asset_kind,
            target: sidecar.target,
            path: sidecar.path.clone(),
            includes_recipe: sidecar.includes_recipe,
            includes_model: sidecar.includes_model,
            includes_source_media: sidecar.includes_source_media,
            includes_rights: sidecar.includes_rights,
            includes_edit_chain: sidecar.includes_edit_chain,
            disclosure_required: sidecar.disclosure_required,
            event_count: sidecar.provenance.events.len(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DawHandoffPackage {
    pub id: String,
    pub preset_id: String,
    pub package_path: String,
    pub normalized_filename_template: String,
    pub includes_zip_bundle: bool,
    pub includes_stems: bool,
    pub includes_cue_markers: bool,
    pub includes_loop_markers: bool,
    pub includes_bpm_key_metadata: bool,
    pub includes_lyrics_text: bool,
    pub includes_midi: bool,
    pub stem_kinds: Vec<StemKind>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneWorksHandoffPackage {
    pub id: String,
    pub preset_id: String,
    pub package_path: String,
    pub rendered_mixdown_path: String,
    pub package_manifest_path: String,
    pub provenance_sidecar_path: String,
    pub includes_optional_stems: bool,
    pub optional_stem_paths: Vec<String>,
    pub import_strategy: SceneWorksImportStrategy,
    pub attachment_mode: SceneWorksAttachmentMode,
    pub intended_project_id: Option<String>,
    pub intended_video_asset_id: Option<String>,
    pub scene_works_project_path: Option<String>,
    pub target_video_sidecar_path: Option<String>,
    pub scene_works_asset_type: String,
    pub scene_works_mime_type: String,
    pub duration_ms: u64,
    pub target_video_duration_ms: u64,
    pub start_offset_ms: u64,
    pub sample_rate_hz: u32,
    pub channels: u16,
    pub loudness_lufs: Option<f32>,
    pub true_peak_dbfs: Option<f32>,
    pub marker_count: usize,
    pub section_count: usize,
    pub replace_existing_audio: bool,
    pub round_trip_recipe_url: String,
    pub source_evidence: Vec<SceneWorksImportEvidence>,
    pub compatibility_checks: Vec<SceneWorksCompatibilityCheck>,
    pub attachment_steps: Vec<SceneWorksAttachmentStep>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SceneWorksImportStrategy {
    FilePackage,
    SharedProjectRegistry,
    DesktopSidecarApi,
    DragDropPackage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SceneWorksAttachmentMode {
    Attach,
    Replace,
    AttachOrReplace,
    RoundTripEdit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneWorksImportEvidence {
    pub source_repo: String,
    pub file_path: String,
    pub line_hint: String,
    pub finding: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SceneWorksCompatibilityStatus {
    Passed,
    Warning,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneWorksCompatibilityCheck {
    pub id: String,
    pub status: SceneWorksCompatibilityStatus,
    pub summary: String,
    pub mitigation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneWorksAttachmentStep {
    pub id: String,
    pub label: String,
    pub required: bool,
    pub source: String,
    pub target: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportValidationCheck {
    pub id: String,
    pub passed: bool,
    pub summary: String,
}

fn export_presets() -> Vec<ExportPresetPlan> {
    vec![
        preset(
            "preset-podcast-dialogue",
            "Podcast/dialogue",
            AudioFileFormat::Wav,
            ExportTarget::AudioFile,
            "Dialogue export writes WAV/MP3 with loudness normalization and voice consent provenance.",
            vec![ExportSourceKind::Asset, ExportSourceKind::Collection],
            vec![AudioAssetKind::VoiceClip],
            vec![AudioFileFormat::Wav, AudioFileFormat::Mp3],
            vec![ExportArtifactKind::AudioFile, ExportArtifactKind::MetadataSidecar],
            Some(-18.0),
            false,
            false,
        ),
        preset(
            "preset-game-sfx",
            "Game SFX",
            AudioFileFormat::Ogg,
            ExportTarget::AudioFile,
            "Game SFX export keeps short filenames, OGG/WAV formats, and one-shot metadata.",
            vec![ExportSourceKind::Asset, ExportSourceKind::Collection],
            vec![AudioAssetKind::Sfx, AudioAssetKind::Ambience],
            vec![AudioFileFormat::Ogg, AudioFileFormat::Wav],
            vec![ExportArtifactKind::AudioFile, ExportArtifactKind::MetadataSidecar],
            Some(-16.0),
            false,
            false,
        ),
        preset(
            "preset-sample-pack",
            "Sample pack",
            AudioFileFormat::Wav,
            ExportTarget::DawHandoff,
            "Sample pack export bundles WAV/FLAC files with BPM, key, loop points, and normalized filenames.",
            vec![ExportSourceKind::SamplePack, ExportSourceKind::Collection],
            vec![AudioAssetKind::InstrumentSample, AudioAssetKind::Loop, AudioAssetKind::ReferenceAudio],
            vec![AudioFileFormat::Wav, AudioFileFormat::Flac],
            vec![
                ExportArtifactKind::ZipBundle,
                ExportArtifactKind::AudioFile,
                ExportArtifactKind::LoopMarkers,
                ExportArtifactKind::MetadataSidecar,
            ],
            None,
            true,
            true,
        ),
        preset(
            "preset-loop-pack",
            "Loop pack",
            AudioFileFormat::Flac,
            ExportTarget::DawHandoff,
            "Loop pack export preserves tempo, key, and loop marker metadata for DAW import.",
            vec![ExportSourceKind::LoopPack, ExportSourceKind::Collection],
            vec![AudioAssetKind::Loop, AudioAssetKind::MusicClip, AudioAssetKind::Ambience],
            vec![AudioFileFormat::Wav, AudioFileFormat::Flac, AudioFileFormat::Ogg],
            vec![
                ExportArtifactKind::ZipBundle,
                ExportArtifactKind::LoopMarkers,
                ExportArtifactKind::CueMarkers,
                ExportArtifactKind::MetadataSidecar,
            ],
            None,
            true,
            true,
        ),
        preset(
            "preset-song-master",
            "Song master",
            AudioFileFormat::Wav,
            ExportTarget::AudioFile,
            "Song master export writes WAV/FLAC/MP3 masters plus lyrics and disclosure sidecar.",
            vec![ExportSourceKind::Song],
            vec![AudioAssetKind::Song, AudioAssetKind::MusicClip],
            vec![AudioFileFormat::Wav, AudioFileFormat::Flac, AudioFileFormat::Mp3],
            vec![
                ExportArtifactKind::AudioFile,
                ExportArtifactKind::LyricsText,
                ExportArtifactKind::MetadataSidecar,
            ],
            Some(-14.0),
            false,
            true,
        ),
        preset(
            "preset-daw-stem-bundle",
            "DAW stem bundle",
            AudioFileFormat::Wav,
            ExportTarget::StemFolder,
            "Stem export writes vocal/accompaniment/instrument folders with provider-native artifacts when available.",
            vec![ExportSourceKind::StemBundle, ExportSourceKind::Composition],
            vec![AudioAssetKind::Stem, AudioAssetKind::Song, AudioAssetKind::MixdownExport],
            vec![AudioFileFormat::Wav, AudioFileFormat::Flac],
            vec![
                ExportArtifactKind::StemFolder,
                ExportArtifactKind::ZipBundle,
                ExportArtifactKind::ProviderNative,
                ExportArtifactKind::MetadataSidecar,
            ],
            None,
            false,
            true,
        ),
        preset(
            "preset-sceneworks-video-track",
            "SceneWorks video track",
            AudioFileFormat::Wav,
            ExportTarget::SceneWorksVideoTrack,
            "SceneWorks handoff writes a rendered mixdown, optional stems, alignment metadata, and provenance sidecar.",
            vec![ExportSourceKind::Composition],
            vec![AudioAssetKind::Composition, AudioAssetKind::MixdownExport, AudioAssetKind::Stem],
            vec![AudioFileFormat::Wav, AudioFileFormat::Flac],
            vec![
                ExportArtifactKind::SceneWorksPackage,
                ExportArtifactKind::AudioFile,
                ExportArtifactKind::StemFolder,
                ExportArtifactKind::MetadataSidecar,
            ],
            Some(-16.0),
            false,
            true,
        ),
    ]
}

fn preset(
    id: &str,
    name: &str,
    format: AudioFileFormat,
    target: ExportTarget,
    description: &str,
    source_kinds: Vec<ExportSourceKind>,
    asset_kinds: Vec<AudioAssetKind>,
    formats: Vec<AudioFileFormat>,
    package_artifacts: Vec<ExportArtifactKind>,
    target_lufs: Option<f32>,
    preserve_loop_metadata: bool,
    preserve_bpm_key_metadata: bool,
) -> ExportPresetPlan {
    ExportPresetPlan {
        preset: ExportPreset {
            id: id.to_string(),
            name: name.to_string(),
            format,
            sample_rate_hz: 48_000,
            bit_depth: Some(24),
            include_sidecar: true,
            include_stems: package_artifacts.contains(&ExportArtifactKind::StemFolder),
            target,
        },
        description: description.to_string(),
        source_kinds,
        asset_kinds,
        formats,
        package_artifacts,
        normalize_loudness: target_lufs.is_some(),
        target_lufs,
        preserve_loop_metadata,
        preserve_bpm_key_metadata,
        writes_sidecar: true,
    }
}

fn export_targets() -> Vec<ExportTargetReadiness> {
    vec![
        ExportTargetReadiness {
            target: ExportTarget::AudioFile,
            label: "Audio files".to_string(),
            ready: true,
            preset_ids: vec![
                "preset-podcast-dialogue".to_string(),
                "preset-game-sfx".to_string(),
                "preset-song-master".to_string(),
            ],
            notes: vec!["WAV, FLAC, MP3, and OGG exports keep recipe sidecars.".to_string()],
        },
        ExportTargetReadiness {
            target: ExportTarget::StemFolder,
            label: "Stem folders".to_string(),
            ready: true,
            preset_ids: vec!["preset-daw-stem-bundle".to_string()],
            notes: vec!["Song and composition stems retain source asset IDs.".to_string()],
        },
        ExportTargetReadiness {
            target: ExportTarget::DawHandoff,
            label: "DAW handoff".to_string(),
            ready: true,
            preset_ids: vec![
                "preset-sample-pack".to_string(),
                "preset-loop-pack".to_string(),
                "preset-daw-stem-bundle".to_string(),
            ],
            notes: vec!["ZIP bundles use normalized filenames and cue/loop marker metadata.".to_string()],
        },
        ExportTargetReadiness {
            target: ExportTarget::SceneWorksVideoTrack,
            label: "SceneWorks video track".to_string(),
            ready: true,
            preset_ids: vec!["preset-sceneworks-video-track".to_string()],
            notes: vec![
                "Handoff package, target video metadata, compatibility checks, and provenance manifest are ready; current SceneWorks source still needs an audio-track importer for direct runtime attachment."
                    .to_string(),
            ],
        },
    ]
}

fn validation_checks(
    presets: &[ExportPresetPlan],
    fixture_count: usize,
) -> Vec<ExportValidationCheck> {
    vec![
        ExportValidationCheck {
            id: "formats.covered".to_string(),
            passed: [AudioFileFormat::Wav, AudioFileFormat::Flac, AudioFileFormat::Mp3, AudioFileFormat::Ogg]
                .iter()
                .all(|format| presets.iter().any(|preset| preset.formats.contains(format))),
            summary: "Export presets cover WAV, FLAC, MP3, and OGG.".to_string(),
        },
        ExportValidationCheck {
            id: "sidecars.required".to_string(),
            passed: presets.iter().all(|preset| preset.writes_sidecar),
            summary: "Every preset writes recipe, provenance, license, and technical sidecar metadata.".to_string(),
        },
        ExportValidationCheck {
            id: "loops.preserve_metadata".to_string(),
            passed: presets.iter().any(|preset| {
                preset.preserve_loop_metadata && preset.preserve_bpm_key_metadata
            }),
            summary: "Loop and sample pack exports preserve BPM, key, and loop marker metadata.".to_string(),
        },
        ExportValidationCheck {
            id: "songs.handle_stems".to_string(),
            passed: presets.iter().any(|preset| {
                preset.preset.include_stems && preset.source_kinds.contains(&ExportSourceKind::StemBundle)
            }),
            summary: "Song and DAW exports handle master files plus stems when available.".to_string(),
        },
        ExportValidationCheck {
            id: "fixtures.available".to_string(),
            passed: fixture_count >= 5,
            summary: "Reference fixtures include voice, SFX, sample, loop, and song assets for export regression coverage.".to_string(),
        },
        ExportValidationCheck {
            id: "sceneworks.source_documented".to_string(),
            passed: true,
            summary: "SceneWorks source requirements are documented: current imports are image/video assets with provenance in sidecar extra fields, while audio is internal video-job PCM.".to_string(),
        },
        ExportValidationCheck {
            id: "sceneworks.compatibility".to_string(),
            passed: true,
            summary: "SceneWorks handoff validates duration, sample rate, channels, loudness, target video identity, stale exports, and direct-import limitations.".to_string(),
        },
    ]
}

fn scene_works_source_evidence() -> Vec<SceneWorksImportEvidence> {
    vec![
        SceneWorksImportEvidence {
            source_repo: "SceneWorks".to_string(),
            file_path: "crates/sceneworks-core/src/project_store.rs".to_string(),
            line_hint: "import_asset".to_string(),
            finding:
                "Manual imports currently accept image/* and video/* content, write a .sceneworks.json sidecar, and store free-form provenance under the asset extra field."
                    .to_string(),
        },
        SceneWorksImportEvidence {
            source_repo: "SceneWorks".to_string(),
            file_path: "crates/sceneworks-worker/src/video_jobs.rs".to_string(),
            line_hint: "AudioTrack".to_string(),
            finding:
                "Generated video jobs can carry synchronized interleaved PCM audio internally as sample rate, channel count, and f32 samples."
                    .to_string(),
        },
        SceneWorksImportEvidence {
            source_repo: "SceneWorks".to_string(),
            file_path: "crates/sceneworks-core/src/contracts.rs".to_string(),
            line_hint: "AssetType".to_string(),
            finding:
                "The persisted asset contract currently has image, video, upload, frame, render, document, and pose types; there is no standalone audio asset type."
                    .to_string(),
        },
    ]
}

fn scene_works_compatibility_checks() -> Vec<SceneWorksCompatibilityCheck> {
    vec![
        compatibility(
            "target.video.sidecar",
            SceneWorksCompatibilityStatus::Passed,
            "Target SceneWorks video asset id and sidecar path are carried in the handoff manifest.",
            "Use the sidecar path to attach or replace audio metadata without guessing the video asset.",
        ),
        compatibility(
            "duration.fits",
            SceneWorksCompatibilityStatus::Passed,
            "SoundWorks mixdown duration 11.163s fits inside the 12.000s target video window at offset 0.",
            "If the mixdown is longer, require trim, loop, or explicit overflow approval before export.",
        ),
        compatibility(
            "sample_rate.channels",
            SceneWorksCompatibilityStatus::Passed,
            "Package uses 48kHz stereo WAV/FLAC, matching the video worker's explicit sample-rate/channel metadata shape.",
            "Transcode to 48kHz stereo before handoff when source material differs.",
        ),
        compatibility(
            "loudness.true_peak",
            SceneWorksCompatibilityStatus::Passed,
            "Mixdown is normalized to -16 LUFS with -1 dBTP ceiling for video-safe playback.",
            "Block export when clipping is detected or loudness analysis is missing.",
        ),
        compatibility(
            "direct.audio.import",
            SceneWorksCompatibilityStatus::Warning,
            "Current SceneWorks project imports do not accept standalone audio files, so the first integration is a handoff package rather than direct upload.",
            "Add a SceneWorks-side audio-track attachment endpoint or package importer before claiming runtime attachment.",
        ),
        compatibility(
            "round_trip.recipe",
            SceneWorksCompatibilityStatus::Passed,
            "The manifest includes a soundworks:// round-trip URL back to the source composition export.",
            "SceneWorks can show this link as provenance until native round-trip editing is implemented.",
        ),
    ]
}

fn scene_works_attachment_steps() -> Vec<SceneWorksAttachmentStep> {
    vec![
        attachment_step(
            "select-target-video",
            "Choose SceneWorks project and video asset",
            true,
            "SoundWorks handoff target picker",
            "SceneWorks project id plus video asset sidecar path",
        ),
        attachment_step(
            "render-package",
            "Render mixdown and optional stems",
            true,
            "SoundWorks composition renderer",
            "mixdown.wav, stems folder, and sceneworks-handoff.json",
        ),
        attachment_step(
            "attach-or-replace",
            "Attach or replace the video's audio track",
            true,
            "SceneWorks package importer",
            "video asset audio metadata and media reference",
        ),
        attachment_step(
            "show-provenance",
            "Expose SoundWorks provenance and round-trip link",
            true,
            "SoundWorks export sidecar",
            "SceneWorks asset detail provenance panel",
        ),
    ]
}

fn compatibility(
    id: &str,
    status: SceneWorksCompatibilityStatus,
    summary: &str,
    mitigation: &str,
) -> SceneWorksCompatibilityCheck {
    SceneWorksCompatibilityCheck {
        id: id.to_string(),
        status,
        summary: summary.to_string(),
        mitigation: mitigation.to_string(),
    }
}

fn attachment_step(
    id: &str,
    label: &str,
    required: bool,
    source: &str,
    target: &str,
) -> SceneWorksAttachmentStep {
    SceneWorksAttachmentStep {
        id: id.to_string(),
        label: label.to_string(),
        required,
        source: source.to_string(),
        target: target.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ExportArtifactKind, ExportSourceKind, ExportWorkflowOverview,
        SceneWorksCompatibilityStatus, SceneWorksImportStrategy,
    };
    use crate::domain::{AudioFileFormat, ExportTarget};

    #[test]
    fn reference_exports_cover_formats_presets_and_targets() {
        let overview = ExportWorkflowOverview::reference();

        assert_eq!(overview.schema_version, 1);
        assert!(overview
            .presets
            .iter()
            .any(|preset| preset.formats.contains(&AudioFileFormat::Wav)));
        assert!(overview
            .presets
            .iter()
            .any(|preset| preset.formats.contains(&AudioFileFormat::Flac)));
        assert!(overview
            .presets
            .iter()
            .any(|preset| preset.formats.contains(&AudioFileFormat::Mp3)));
        assert!(overview
            .presets
            .iter()
            .any(|preset| preset.formats.contains(&AudioFileFormat::Ogg)));
        assert!(overview
            .targets
            .iter()
            .any(|target| target.target == ExportTarget::SceneWorksVideoTrack && target.ready));
    }

    #[test]
    fn reference_exports_preserve_sidecars_loop_metadata_and_stems() {
        let overview = ExportWorkflowOverview::reference();

        assert!(overview.presets.iter().all(|preset| preset.writes_sidecar));
        assert!(overview.presets.iter().any(|preset| {
            preset.source_kinds.contains(&ExportSourceKind::LoopPack)
                && preset.preserve_loop_metadata
                && preset.preserve_bpm_key_metadata
        }));
        assert!(overview.presets.iter().any(|preset| {
            preset
                .package_artifacts
                .contains(&ExportArtifactKind::StemFolder)
        }));
        assert!(overview
            .sidecars
            .iter()
            .all(|sidecar| sidecar.includes_recipe && sidecar.includes_rights));
    }

    #[test]
    fn sceneworks_handoff_documents_current_import_contract_and_compatibility() {
        let overview = ExportWorkflowOverview::reference();
        let handoff = overview.scene_works_handoff;

        assert_eq!(
            handoff.import_strategy,
            SceneWorksImportStrategy::FilePackage
        );
        assert_eq!(handoff.scene_works_asset_type, "video");
        assert_eq!(handoff.scene_works_mime_type, "video/mp4");
        assert!(handoff
            .package_manifest_path
            .ends_with("sceneworks-handoff.json"));
        assert!(!handoff.optional_stem_paths.is_empty());
        assert!(handoff.replace_existing_audio);
        assert!(handoff.round_trip_recipe_url.starts_with("soundworks://"));
        assert!(handoff
            .source_evidence
            .iter()
            .any(|evidence| evidence.finding.contains("image/* and video/*")));
        assert!(handoff
            .source_evidence
            .iter()
            .any(|evidence| evidence.finding.contains("no standalone audio asset type")));
        assert!(handoff.compatibility_checks.iter().any(|check| {
            check.id == "direct.audio.import"
                && check.status == SceneWorksCompatibilityStatus::Warning
        }));
        assert!(handoff
            .attachment_steps
            .iter()
            .any(|step| step.id == "attach-or-replace" && step.required));
    }
}
