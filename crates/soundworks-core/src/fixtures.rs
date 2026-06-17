use crate::domain::{
    AssetCreation, AudioAsset, AudioAssetKind, AudioAssetVersion, AudioFileFormat,
    AudioFileReference, AutomationLane, AutomationPoint, AutomationTarget, Composition,
    CompositionClip, CompositionExport, CompositionTrack, GenerationJob, GenerationRecipe,
    InstrumentSampleRecipe, JobKind, JobStatus, LibraryScope, LoopPoints, LoopRecipe,
    ModelDescriptor, ModelRuntime, Project, RecipeRequest, RecipeWorkflow, RightsMetadata,
    SfxRecipe, SongRecipe, SongSection, SongStructure, StemKind, TechnicalAudioMetadata, TimeRange,
    TimelineMarker, TimelineSection, TrackRole, TtsRecipe,
};
use crate::storage::{StoragePathAllocator, StoragePathError};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub struct AssetFixture {
    pub asset: AudioAsset,
    pub version: AudioAssetVersion,
    pub recipe: GenerationRecipe,
    pub job: GenerationJob,
}

pub fn fixture_set() -> Result<Vec<AssetFixture>, StoragePathError> {
    let allocator = StoragePathAllocator::new("soundworks-library");

    Ok(vec![
        tts_fixture(&allocator)?,
        sfx_fixture(&allocator)?,
        instrument_sample_fixture(&allocator)?,
        loop_fixture(&allocator)?,
        song_fixture(&allocator)?,
    ])
}

pub fn project_fixture() -> Project {
    Project {
        id: "project-demo".to_string(),
        name: "Demo SoundWorks Project".to_string(),
        storage_root: "soundworks-library/projects/project-demo".to_string(),
        asset_ids: vec![
            "asset-voice-001".to_string(),
            "asset-loop-001".to_string(),
            "asset-song-001".to_string(),
        ],
        composition_ids: vec!["composition-demo".to_string()],
        recipe_ids: vec!["recipe-tts-001".to_string(), "recipe-loop-001".to_string()],
        job_ids: vec!["job-asset-voice-001".to_string()],
    }
}

pub fn composition_fixture() -> Composition {
    Composition {
        id: "composition-demo".to_string(),
        scope: LibraryScope::Project {
            project_id: "project-demo".to_string(),
        },
        name: "Demo timeline".to_string(),
        tempo_bpm: Some(86.0),
        musical_key: Some("C minor".to_string()),
        tracks: vec![
            CompositionTrack {
                id: "track-voice".to_string(),
                name: "Narration".to_string(),
                role: TrackRole::Voice,
                clips: vec![CompositionClip {
                    id: "clip-voice-intro".to_string(),
                    asset_id: "asset-voice-001".to_string(),
                    version_id: "version-voice-001-a".to_string(),
                    timeline_start_ms: 0,
                    source_range: TimeRange {
                        start_ms: 250,
                        end_ms: 3900,
                    },
                    fade_in_ms: 25,
                    fade_out_ms: 80,
                    gain_db: -1.5,
                    pan: 0.0,
                }],
                gain_db: 0.0,
                pan: 0.0,
                muted: false,
                soloed: false,
                automation: vec![AutomationLane {
                    target: AutomationTarget::Gain,
                    points: vec![
                        AutomationPoint {
                            at_ms: 0,
                            value: -3.0,
                        },
                        AutomationPoint {
                            at_ms: 500,
                            value: 0.0,
                        },
                    ],
                }],
            },
            CompositionTrack {
                id: "track-loop".to_string(),
                name: "Loop bed".to_string(),
                role: TrackRole::Music,
                clips: vec![CompositionClip {
                    id: "clip-loop-bed".to_string(),
                    asset_id: "asset-loop-001".to_string(),
                    version_id: "version-loop-001-a".to_string(),
                    timeline_start_ms: 0,
                    source_range: TimeRange {
                        start_ms: 0,
                        end_ms: 11_163,
                    },
                    fade_in_ms: 0,
                    fade_out_ms: 250,
                    gain_db: -6.0,
                    pan: 0.0,
                }],
                gain_db: -2.0,
                pan: 0.0,
                muted: false,
                soloed: false,
                automation: vec![],
            },
        ],
        markers: vec![TimelineMarker {
            id: "marker-intro".to_string(),
            at_ms: 0,
            label: "Intro".to_string(),
        }],
        sections: vec![TimelineSection {
            id: "section-intro".to_string(),
            range: TimeRange {
                start_ms: 0,
                end_ms: 11_163,
            },
            label: "Intro bed".to_string(),
        }],
        export_history: vec![CompositionExport {
            id: "export-demo-mix".to_string(),
            job_id: "job-export-demo-mix".to_string(),
            output_asset_id: "asset-mixdown-001".to_string(),
            preset_id: "preset-sceneworks-video-track".to_string(),
        }],
        provenance_ids: vec!["provenance-composition-demo".to_string()],
    }
}

fn tts_fixture(allocator: &StoragePathAllocator) -> Result<AssetFixture, StoragePathError> {
    let recipe = base_recipe(
        "recipe-tts-001",
        RecipeWorkflow::Tts,
        RecipeRequest::Tts(TtsRecipe {
            script: "Welcome to the SoundWorks scratch narration.".to_string(),
            language: Some("en-US".to_string()),
            speaker_labels: vec!["Narrator".to_string()],
            voice_profile_id: Some("voice-profile-narrator".to_string()),
            pronunciation_notes: vec!["SoundWorks uses a short o.".to_string()],
            target_duration_ms: Some(4200),
        }),
        "asset-voice-001",
    );

    generated_fixture(
        allocator,
        "asset-voice-001",
        "version-voice-001-a",
        AudioAssetKind::VoiceClip,
        "Narration scratch",
        recipe,
        TechnicalAudioMetadata {
            duration_ms: 4200,
            ..speech_metadata()
        },
    )
}

fn sfx_fixture(allocator: &StoragePathAllocator) -> Result<AssetFixture, StoragePathError> {
    let recipe = base_recipe(
        "recipe-sfx-001",
        RecipeWorkflow::Sfx,
        RecipeRequest::Sfx(SfxRecipe {
            prompt: "Close metallic hatch impact with short tail.".to_string(),
            negative_prompt: Some("music, dialogue".to_string()),
            category: Some("impact".to_string()),
            target_duration_ms: Some(1800),
            loopable: false,
        }),
        "asset-sfx-001",
    );

    generated_fixture(
        allocator,
        "asset-sfx-001",
        "version-sfx-001-a",
        AudioAssetKind::Sfx,
        "Metal hatch impact",
        recipe,
        TechnicalAudioMetadata {
            duration_ms: 1800,
            true_peak_dbfs: Some(-1.0),
            ..music_metadata()
        },
    )
}

fn instrument_sample_fixture(
    allocator: &StoragePathAllocator,
) -> Result<AssetFixture, StoragePathError> {
    let recipe = base_recipe(
        "recipe-sample-001",
        RecipeWorkflow::InstrumentSample,
        RecipeRequest::InstrumentSample(InstrumentSampleRecipe {
            prompt: "Warm analog pluck, single C3 note, dry.".to_string(),
            instrument: Some("synth pluck".to_string()),
            pitch: Some("C3".to_string()),
            velocity: Some(96),
            target_duration_ms: Some(2600),
        }),
        "asset-sample-001",
    );

    generated_fixture(
        allocator,
        "asset-sample-001",
        "version-sample-001-a",
        AudioAssetKind::InstrumentSample,
        "Analog pluck C3",
        recipe,
        TechnicalAudioMetadata {
            duration_ms: 2600,
            musical_key: Some("C".to_string()),
            ..music_metadata()
        },
    )
}

fn loop_fixture(allocator: &StoragePathAllocator) -> Result<AssetFixture, StoragePathError> {
    let recipe = base_recipe(
        "recipe-loop-001",
        RecipeWorkflow::Loop,
        RecipeRequest::Loop(LoopRecipe {
            prompt: "Four bar dusty trip-hop drum loop.".to_string(),
            bpm: 86.0,
            musical_key: None,
            bars: 4,
            loopable: true,
        }),
        "asset-loop-001",
    );

    generated_fixture(
        allocator,
        "asset-loop-001",
        "version-loop-001-a",
        AudioAssetKind::Loop,
        "Dusty trip-hop drums",
        recipe,
        TechnicalAudioMetadata {
            duration_ms: 11163,
            bpm: Some(86.0),
            loop_points: Some(LoopPoints {
                start_sample: 0,
                end_sample: 492_288,
            }),
            ..music_metadata()
        },
    )
}

fn song_fixture(allocator: &StoragePathAllocator) -> Result<AssetFixture, StoragePathError> {
    let recipe = base_recipe(
        "recipe-song-001",
        RecipeWorkflow::Song,
        RecipeRequest::Song(SongRecipe {
            prompt: "Uplifting synth pop cue for a product reveal.".to_string(),
            lyrics: "Lights rise / we find the signal".to_string(),
            style_tags: vec!["synth-pop".to_string(), "uplifting".to_string()],
            structure: SongStructure {
                sections: vec![
                    SongSection {
                        label: "verse".to_string(),
                        lyrics: Some("Lights rise".to_string()),
                        duration_ms: Some(24_000),
                    },
                    SongSection {
                        label: "chorus".to_string(),
                        lyrics: Some("We find the signal".to_string()),
                        duration_ms: Some(32_000),
                    },
                ],
            },
            requested_stems: vec![StemKind::Vocals, StemKind::Drums, StemKind::Instruments],
            reference_audio_ids: vec![],
        }),
        "asset-song-001",
    );

    generated_fixture(
        allocator,
        "asset-song-001",
        "version-song-001-a",
        AudioAssetKind::Song,
        "Signal reveal cue",
        recipe,
        TechnicalAudioMetadata {
            duration_ms: 132_000,
            bpm: Some(118.0),
            musical_key: Some("A major".to_string()),
            ..music_metadata()
        },
    )
}

fn generated_fixture(
    allocator: &StoragePathAllocator,
    asset_id: &str,
    version_id: &str,
    kind: AudioAssetKind,
    name: &str,
    recipe: GenerationRecipe,
    technical: TechnicalAudioMetadata,
) -> Result<AssetFixture, StoragePathError> {
    let scope = LibraryScope::Project {
        project_id: "project-demo".to_string(),
    };
    let storage = allocator.allocate_asset_version(
        &scope,
        kind,
        asset_id,
        version_id,
        AudioFileFormat::Wav,
    )?;
    let job_id = format!("job-{asset_id}");

    let version = AudioAssetVersion {
        id: version_id.to_string(),
        asset_id: asset_id.to_string(),
        version_index: 1,
        file: AudioFileReference {
            storage_path: storage.media_path,
            format: AudioFileFormat::Wav,
            codec: Some("pcm_s16le".to_string()),
            byte_size: None,
            content_hash: None,
        },
        technical,
        created_by: AssetCreation::Generated {
            recipe_id: recipe.id.clone(),
            job_id: job_id.clone(),
        },
        waveform_preview_cache: Some(storage.waveform_preview_path),
        spectrogram_preview_cache: Some(storage.spectrogram_preview_path),
    };

    let asset = AudioAsset {
        id: asset_id.to_string(),
        scope,
        kind,
        name: name.to_string(),
        tags: vec![kind.storage_dir().to_string()],
        collection_ids: vec![],
        current_version_id: version.id.clone(),
        version_ids: vec![version.id.clone()],
        rights: RightsMetadata::user_owned_commercial(),
        provenance_ids: vec![format!("provenance-{asset_id}")],
    };

    let job = GenerationJob {
        id: job_id,
        recipe_id: recipe.id.clone(),
        kind: JobKind::GenerateAudio,
        status: JobStatus::Succeeded,
        progress: None,
        output_version_ids: vec![version.id.clone()],
        error: None,
    };

    Ok(AssetFixture {
        asset,
        version,
        recipe,
        job,
    })
}

fn base_recipe(
    id: &str,
    workflow: RecipeWorkflow,
    request: RecipeRequest,
    output_asset_id: &str,
) -> GenerationRecipe {
    GenerationRecipe {
        id: id.to_string(),
        workflow,
        provider: ModelDescriptor {
            provider_id: "fixture-provider".to_string(),
            model_id: "fixture-audio-model".to_string(),
            model_version: Some("0.1.0".to_string()),
            model_hash: Some("sha256:fixture".to_string()),
            runtime: ModelRuntime::Local,
        },
        request,
        seed: Some(42),
        source_references: vec![],
        post_processing: vec![],
        parameter_overrides: BTreeMap::new(),
        output_asset_ids: vec![output_asset_id.to_string()],
    }
}

fn speech_metadata() -> TechnicalAudioMetadata {
    TechnicalAudioMetadata {
        sample_rate_hz: 48_000,
        bit_depth: Some(24),
        channels: 1,
        duration_ms: 0,
        loudness_lufs: Some(-18.0),
        true_peak_dbfs: Some(-2.0),
        has_clipping: false,
        bpm: None,
        musical_key: None,
        loop_points: None,
    }
}

fn music_metadata() -> TechnicalAudioMetadata {
    TechnicalAudioMetadata {
        sample_rate_hz: 48_000,
        bit_depth: Some(24),
        channels: 2,
        duration_ms: 0,
        loudness_lufs: Some(-14.0),
        true_peak_dbfs: Some(-1.5),
        has_clipping: false,
        bpm: None,
        musical_key: None,
        loop_points: None,
    }
}
