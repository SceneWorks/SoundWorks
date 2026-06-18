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
                    "SceneWorks target is stored as handoff metadata until sc-6202 defines the import contract."
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
                provenance_sidecar_path:
                    "soundworks-exports/project-demo/demo-timeline/soundworks-export.json"
                        .to_string(),
                includes_optional_stems: true,
                intended_project_id: Some("sceneworks-project-placeholder".to_string()),
                intended_video_asset_id: Some("video-target-placeholder".to_string()),
                duration_ms: 11_163,
                sample_rate_hz: 48_000,
                channels: 2,
                loudness_lufs: Some(-16.0),
                true_peak_dbfs: Some(-1.0),
                marker_count: 1,
                section_count: 1,
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
    pub provenance_sidecar_path: String,
    pub includes_optional_stems: bool,
    pub intended_project_id: Option<String>,
    pub intended_video_asset_id: Option<String>,
    pub duration_ms: u64,
    pub sample_rate_hz: u32,
    pub channels: u16,
    pub loudness_lufs: Option<f32>,
    pub true_peak_dbfs: Option<f32>,
    pub marker_count: usize,
    pub section_count: usize,
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
                "First-class handoff package is ready; actual SceneWorks import is tracked by sc-6202."
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
    ]
}

#[cfg(test)]
mod tests {
    use super::{ExportArtifactKind, ExportSourceKind, ExportWorkflowOverview};
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
}
