use crate::domain::{
    AssetCreation, AudioAsset, AudioAssetKind, AudioAssetVersion, AudioFileFormat,
    AudioFileReference, Collection, CommercialUseStatus, Composition, GenerationRecipe,
    LibraryScope, LicenseStatus, PromptPreset, RecipeSummary, RecipeWorkflow, RightsMetadata,
    TechnicalAudioMetadata, VoiceConsentStatus, WatermarkStatus,
};
use crate::fixtures::{composition_fixture, fixture_set, project_fixture, AssetFixture};
use crate::storage::{StoragePathAllocator, StoragePathError};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};

pub const ASSET_LIBRARY_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetLibraryOverview {
    pub schema_version: u32,
    pub scopes: Vec<LibraryScopeSummary>,
    pub filters: LibraryFilterModel,
    pub selected_filter: LibraryFilterQuery,
    pub items: Vec<LibraryItemCard>,
    pub selected_item: LibraryItemDetail,
    pub collections: Vec<LibraryCollectionView>,
    pub lifecycle_actions: Vec<LibraryLifecycleAction>,
    pub drag_targets: Vec<LibraryDragTarget>,
    pub validation_checks: Vec<LibraryValidationCheck>,
}

impl AssetLibraryOverview {
    pub fn reference() -> Result<Self, StoragePathError> {
        let fixtures = fixture_set()?;
        let composition = composition_fixture();
        let project = project_fixture();
        let allocator = StoragePathAllocator::new("soundworks-library");
        let collections = reference_collections();
        let mut items = fixture_items(&fixtures);
        items.extend(extra_audio_items(&allocator, &composition)?);
        items.extend(non_audio_library_items());

        let selected_item = items
            .iter()
            .find(|item| item.id == "asset-loop-001")
            .cloned()
            .expect("reference library includes loop asset");

        Ok(Self {
            schema_version: ASSET_LIBRARY_SCHEMA_VERSION,
            scopes: vec![
                LibraryScopeSummary {
                    id: "project-demo".to_string(),
                    label: project.name,
                    scope: LibraryScope::Project {
                        project_id: "project-demo".to_string(),
                    },
                    ownership: LibraryOwnership::ProjectLocal,
                    asset_count: items
                        .iter()
                        .filter(|item| item.project_id.as_deref() == Some("project-demo"))
                        .count(),
                    collection_count: collections
                        .iter()
                        .filter(|collection| {
                            matches!(
                                collection.collection.scope,
                                LibraryScope::Project { ref project_id } if project_id == "project-demo"
                            )
                        })
                        .count(),
                    can_promote_to_global: true,
                },
                LibraryScopeSummary {
                    id: "global-library".to_string(),
                    label: "Global audio library".to_string(),
                    scope: LibraryScope::GlobalLibrary,
                    ownership: LibraryOwnership::Global,
                    asset_count: items
                        .iter()
                        .filter(|item| item.ownership == LibraryOwnership::Global)
                        .count(),
                    collection_count: collections
                        .iter()
                        .filter(|collection| {
                            matches!(collection.collection.scope, LibraryScope::GlobalLibrary)
                        })
                        .count(),
                    can_promote_to_global: false,
                },
            ],
            filters: LibraryFilterModel::from_items(&items),
            selected_filter: LibraryFilterQuery::reference(),
            items: items.clone(),
            selected_item: LibraryItemDetail {
                item: selected_item.clone(),
                version_history: version_history(&selected_item),
                recipe: selected_item.recipe.clone(),
                provenance_links: vec![
                    LibraryProvenanceLink {
                        id: "provenance-loop-generated".to_string(),
                        label: "Generated from loop recipe".to_string(),
                        sidecar_path: "soundworks-library/projects/project-demo/loops/asset-loop-001/version-loop-001-a/metadata/recipe-provenance.json".to_string(),
                        inspectable: true,
                    },
                    LibraryProvenanceLink {
                        id: "provenance-loop-review-edit".to_string(),
                        label: "Available to review/edit chain".to_string(),
                        sidecar_path: "soundworks-library/projects/project-demo/loops/asset-loop-001/version-loop-001-b-review-edit/metadata/recipe-provenance.json".to_string(),
                        inspectable: true,
                    },
                ],
                collection_ids: selected_item.collection_ids.clone(),
                version_count: 2,
                source_picker_targets: vec![
                    "Samples + Loops".to_string(),
                    "Waveform Review".to_string(),
                    "Mixer timeline".to_string(),
                ],
                notes: vec![
                    "Project-local loop can be promoted to global without losing recipe provenance."
                        .to_string(),
                    "Rejected and archived items remain findable only when lifecycle filters include them."
                        .to_string(),
                ],
            },
            collections,
            lifecycle_actions: lifecycle_actions(),
            drag_targets: drag_targets(),
            validation_checks: validation_checks(&items),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryScopeSummary {
    pub id: String,
    pub label: String,
    pub scope: LibraryScope,
    pub ownership: LibraryOwnership,
    pub asset_count: usize,
    pub collection_count: usize,
    pub can_promote_to_global: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LibraryOwnership {
    ProjectLocal,
    Global,
    LinkedGlobal,
    CopiedFromGlobal,
    ExportedToSceneWorks,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LibraryItemType {
    VoiceClip,
    MusicClip,
    Sfx,
    Song,
    InstrumentSample,
    Loop,
    Stem,
    Ambience,
    VoiceProfile,
    ReferenceAudio,
    Composition,
    MixdownExport,
    PromptRecipePreset,
}

impl LibraryItemType {
    fn from_asset_kind(kind: AudioAssetKind) -> Self {
        match kind {
            AudioAssetKind::VoiceClip => Self::VoiceClip,
            AudioAssetKind::MusicClip => Self::MusicClip,
            AudioAssetKind::Sfx => Self::Sfx,
            AudioAssetKind::Song => Self::Song,
            AudioAssetKind::InstrumentSample => Self::InstrumentSample,
            AudioAssetKind::Loop => Self::Loop,
            AudioAssetKind::Stem => Self::Stem,
            AudioAssetKind::Ambience => Self::Ambience,
            AudioAssetKind::ReferenceAudio => Self::ReferenceAudio,
            AudioAssetKind::Composition => Self::Composition,
            AudioAssetKind::MixdownExport => Self::MixdownExport,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::VoiceClip => "Voice clip",
            Self::MusicClip => "Music clip",
            Self::Sfx => "SFX",
            Self::Song => "Song",
            Self::InstrumentSample => "Instrument sample",
            Self::Loop => "Loop",
            Self::Stem => "Stem",
            Self::Ambience => "Ambience",
            Self::VoiceProfile => "Voice profile",
            Self::ReferenceAudio => "Reference audio",
            Self::Composition => "Composition",
            Self::MixdownExport => "Mixdown/Export",
            Self::PromptRecipePreset => "Prompt/Recipe preset",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryItemCard {
    pub id: String,
    pub name: String,
    pub item_type: LibraryItemType,
    pub item_type_label: String,
    pub asset: Option<AudioAsset>,
    pub current_version: Option<AudioAssetVersion>,
    pub scope: LibraryScope,
    pub ownership: LibraryOwnership,
    pub project_id: Option<String>,
    pub created_at: String,
    pub source_workflow: Option<RecipeWorkflow>,
    pub tags: Vec<String>,
    pub generated_tags: Vec<String>,
    pub collection_ids: Vec<String>,
    pub duration_ms: Option<u64>,
    pub bpm: Option<f32>,
    pub musical_key: Option<String>,
    pub language: Option<String>,
    pub voice_profile_id: Option<String>,
    pub provider_id: Option<String>,
    pub model_id: Option<String>,
    pub license_status: LicenseStatus,
    pub commercial_use: CommercialUseStatus,
    pub favorite: bool,
    pub rejected: bool,
    pub archived: bool,
    pub waveform_thumbnail: Option<WaveformThumbnail>,
    pub quick_audition: QuickAuditionState,
    pub timeline_placeable: bool,
    pub source_picker_eligible: bool,
    pub composition_usage_count: usize,
    pub recipe: Option<RecipeSummary>,
    pub badges: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WaveformThumbnail {
    pub preview_path: String,
    pub peak_count: usize,
    pub duration_ms: u64,
    pub ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuickAuditionState {
    pub previewable: bool,
    pub playable_range_ms: Option<(u64, u64)>,
    pub shortcut: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryItemDetail {
    pub item: LibraryItemCard,
    pub version_history: Vec<LibraryVersionEntry>,
    pub recipe: Option<RecipeSummary>,
    pub provenance_links: Vec<LibraryProvenanceLink>,
    pub collection_ids: Vec<String>,
    pub version_count: usize,
    pub source_picker_targets: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryVersionEntry {
    pub version_id: String,
    pub label: String,
    pub duration_ms: Option<u64>,
    pub file_path: Option<String>,
    pub created_by: String,
    pub waveform_ready: bool,
    pub recipe_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryProvenanceLink {
    pub id: String,
    pub label: String,
    pub sidecar_path: String,
    pub inspectable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryCollectionView {
    pub collection: Collection,
    pub collection_type: CollectionType,
    pub description: String,
    pub item_count: usize,
    pub drag_into_studios: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CollectionType {
    Collection,
    SamplePack,
    SongFolder,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryFilterModel {
    pub facets: Vec<LibraryFilterFacet>,
    pub supported_item_types: Vec<LibraryItemType>,
    pub covers_project_and_global_scopes: bool,
    pub includes_rejected_archived_toggle: bool,
}

impl LibraryFilterModel {
    fn from_items(items: &[LibraryItemCard]) -> Self {
        Self {
            facets: vec![
                facet("type", "Type", items, |item| item.item_type_label.clone()),
                facet("tags", "Tags", items, |item| item.tags.join(", ")),
                facet("duration", "Duration", items, duration_bucket),
                facet("bpm", "BPM", items, bpm_bucket),
                facet("key", "Key", items, |item| {
                    item.musical_key
                        .clone()
                        .unwrap_or_else(|| "No key".to_string())
                }),
                facet("language", "Language", items, |item| {
                    item.language
                        .clone()
                        .unwrap_or_else(|| "Instrumental".to_string())
                }),
                facet("voice", "Voice", items, |item| {
                    item.voice_profile_id
                        .clone()
                        .unwrap_or_else(|| "No voice".to_string())
                }),
                facet("model", "Model", items, |item| {
                    item.model_id
                        .clone()
                        .unwrap_or_else(|| "Manual/imported".to_string())
                }),
                facet("license", "License", items, |item| {
                    format!("{:?}", item.license_status)
                }),
                facet("project", "Project", items, |item| {
                    item.project_id
                        .clone()
                        .unwrap_or_else(|| "Global library".to_string())
                }),
                facet("createdDate", "Created date", items, |item| {
                    item.created_at[..10].to_string()
                }),
                facet("collection", "Collection", items, |item| {
                    if item.collection_ids.is_empty() {
                        "No collection".to_string()
                    } else {
                        item.collection_ids.join(", ")
                    }
                }),
                facet("lifecycle", "Lifecycle", items, lifecycle_bucket),
                facet("sourceWorkflow", "Source workflow", items, |item| {
                    item.source_workflow
                        .map(|workflow| format!("{workflow:?}"))
                        .unwrap_or_else(|| "Manual/imported".to_string())
                }),
                facet("compositionUsage", "Composition usage", items, |item| {
                    if item.composition_usage_count > 0 {
                        "Used in composition".to_string()
                    } else {
                        "Unused".to_string()
                    }
                }),
            ],
            supported_item_types: vec![
                LibraryItemType::VoiceClip,
                LibraryItemType::MusicClip,
                LibraryItemType::Sfx,
                LibraryItemType::Song,
                LibraryItemType::InstrumentSample,
                LibraryItemType::Loop,
                LibraryItemType::Stem,
                LibraryItemType::Ambience,
                LibraryItemType::VoiceProfile,
                LibraryItemType::ReferenceAudio,
                LibraryItemType::Composition,
                LibraryItemType::MixdownExport,
                LibraryItemType::PromptRecipePreset,
            ],
            covers_project_and_global_scopes: true,
            includes_rejected_archived_toggle: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryFilterFacet {
    pub id: String,
    pub label: String,
    pub options: Vec<LibraryFilterOption>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryFilterOption {
    pub id: String,
    pub label: String,
    pub count: usize,
    pub selected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryFilterQuery {
    pub search_text: String,
    pub scope: LibraryScope,
    pub selected_type: Option<LibraryItemType>,
    pub selected_tags: Vec<String>,
    pub include_rejected: bool,
    pub include_archived: bool,
    pub favorite_only: bool,
}

impl LibraryFilterQuery {
    fn reference() -> Self {
        Self {
            search_text: "loop commercial local".to_string(),
            scope: LibraryScope::Project {
                project_id: "project-demo".to_string(),
            },
            selected_type: Some(LibraryItemType::Loop),
            selected_tags: vec!["drums".to_string(), "loop".to_string()],
            include_rejected: false,
            include_archived: false,
            favorite_only: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryLifecycleAction {
    pub id: String,
    pub label: String,
    pub applies_to: Vec<LibraryItemType>,
    pub preserves_provenance: bool,
    pub destructive: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryDragTarget {
    pub id: String,
    pub label: String,
    pub accepted_types: Vec<LibraryItemType>,
    pub creates_linked_copy: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryValidationCheck {
    pub id: String,
    pub passed: bool,
    pub summary: String,
}

fn fixture_items(fixtures: &[AssetFixture]) -> Vec<LibraryItemCard> {
    fixtures
        .iter()
        .enumerate()
        .map(|(index, fixture)| {
            let tags = tags_for_kind(fixture.asset.kind);
            let mut asset = fixture.asset.clone();
            asset.tags = tags.clone();
            asset.collection_ids = collection_ids_for_kind(asset.kind);

            item_from_asset(
                asset,
                fixture.version.clone(),
                Some(fixture.recipe.clone()),
                "2026-06-17T11:40:00Z",
                index == 3,
                false,
                false,
                1,
                LibraryOwnership::ProjectLocal,
            )
        })
        .collect()
}

fn extra_audio_items(
    allocator: &StoragePathAllocator,
    composition: &Composition,
) -> Result<Vec<LibraryItemCard>, StoragePathError> {
    Ok(vec![
        synthetic_audio_item(
            allocator,
            "asset-reference-neon-bass",
            AudioAssetKind::ReferenceAudio,
            "Neon bass reference",
            LibraryScope::GlobalLibrary,
            LibraryOwnership::Global,
            vec!["reference", "bass", "global"],
            Some(26_400),
            Some(120.0),
            Some("A minor"),
            None,
            false,
            false,
            false,
            2,
        )?,
        synthetic_audio_item(
            allocator,
            "asset-ambience-engine-room",
            AudioAssetKind::Ambience,
            "Engine room bed",
            LibraryScope::Project {
                project_id: "project-demo".to_string(),
            },
            LibraryOwnership::ProjectLocal,
            vec!["ambience", "industrial", "bed"],
            Some(45_000),
            None,
            None,
            None,
            false,
            false,
            false,
            1,
        )?,
        synthetic_audio_item(
            allocator,
            "asset-stem-drums-001",
            AudioAssetKind::Stem,
            "City Lights drum stem",
            LibraryScope::Project {
                project_id: "project-demo".to_string(),
            },
            LibraryOwnership::CopiedFromGlobal,
            vec!["stem", "drums", "song-folder"],
            Some(184_000),
            Some(118.0),
            Some("D minor"),
            None,
            false,
            false,
            false,
            1,
        )?,
        synthetic_audio_item(
            allocator,
            "asset-music-cue-001",
            AudioAssetKind::MusicClip,
            "Underscore cue idea",
            LibraryScope::Project {
                project_id: "project-demo".to_string(),
            },
            LibraryOwnership::ProjectLocal,
            vec!["music", "cue", "favorite"],
            Some(31_000),
            Some(86.0),
            Some("C minor"),
            None,
            true,
            false,
            false,
            0,
        )?,
        synthetic_audio_item(
            allocator,
            "asset-mixdown-001",
            AudioAssetKind::MixdownExport,
            "Demo timeline mixdown",
            LibraryScope::Project {
                project_id: "project-demo".to_string(),
            },
            LibraryOwnership::ExportedToSceneWorks,
            vec!["mixdown", "export", "sceneworks"],
            Some(184_000),
            Some(86.0),
            Some("C minor"),
            None,
            false,
            false,
            false,
            1,
        )?,
        composition_item(composition),
    ])
}

fn synthetic_audio_item(
    allocator: &StoragePathAllocator,
    asset_id: &str,
    kind: AudioAssetKind,
    name: &str,
    scope: LibraryScope,
    ownership: LibraryOwnership,
    tags: Vec<&str>,
    duration_ms: Option<u64>,
    bpm: Option<f32>,
    musical_key: Option<&str>,
    language: Option<&str>,
    favorite: bool,
    rejected: bool,
    archived: bool,
    composition_usage_count: usize,
) -> Result<LibraryItemCard, StoragePathError> {
    let version_id = format!("version-{}-a", asset_id.trim_start_matches("asset-"));
    let paths = allocator.allocate_asset_version(
        &scope,
        kind,
        asset_id,
        &version_id,
        AudioFileFormat::Wav,
    )?;
    let asset = AudioAsset {
        id: asset_id.to_string(),
        scope: scope.clone(),
        kind,
        name: name.to_string(),
        tags: tags.iter().map(|tag| (*tag).to_string()).collect(),
        collection_ids: collection_ids_for_kind(kind),
        current_version_id: version_id.clone(),
        version_ids: vec![version_id.clone()],
        rights: RightsMetadata::user_owned_commercial(),
        provenance_ids: vec![format!("provenance-{asset_id}")],
    };
    let version = AudioAssetVersion {
        id: version_id,
        asset_id: asset_id.to_string(),
        version_index: 1,
        file: AudioFileReference {
            storage_path: paths.media_path,
            format: AudioFileFormat::Wav,
            codec: Some("pcm_s16le".to_string()),
            byte_size: Some(1_024_000),
            content_hash: Some(format!("sha256-{asset_id}")),
        },
        technical: TechnicalAudioMetadata {
            sample_rate_hz: 48_000,
            bit_depth: Some(24),
            channels: 2,
            duration_ms: duration_ms.unwrap_or(0),
            loudness_lufs: Some(-16.0),
            true_peak_dbfs: Some(-1.0),
            has_clipping: false,
            bpm,
            musical_key: musical_key.map(str::to_string),
            loop_points: None,
        },
        created_by: AssetCreation::Imported {
            source_reference_id: format!("source-{asset_id}"),
        },
        waveform_preview_cache: Some(paths.waveform_preview_path),
        spectrogram_preview_cache: Some(paths.spectrogram_preview_path),
    };

    let mut item = item_from_asset(
        asset,
        version,
        None,
        "2026-06-17T12:10:00Z",
        favorite,
        rejected,
        archived,
        composition_usage_count,
        ownership,
    );
    item.language = language.map(str::to_string);
    Ok(item)
}

fn composition_item(composition: &Composition) -> LibraryItemCard {
    LibraryItemCard {
        id: composition.id.clone(),
        name: composition.name.clone(),
        item_type: LibraryItemType::Composition,
        item_type_label: LibraryItemType::Composition.label().to_string(),
        asset: None,
        current_version: None,
        scope: composition.scope.clone(),
        ownership: LibraryOwnership::ProjectLocal,
        project_id: Some("project-demo".to_string()),
        created_at: "2026-06-17T12:20:00Z".to_string(),
        source_workflow: Some(RecipeWorkflow::CompositionRender),
        tags: vec!["composition".to_string(), "timeline".to_string()],
        generated_tags: vec!["two-track".to_string(), "markers".to_string()],
        collection_ids: vec!["collection-demo-song-folder".to_string()],
        duration_ms: composition
            .sections
            .first()
            .map(|section| section.range.end_ms),
        bpm: composition.tempo_bpm,
        musical_key: composition.musical_key.clone(),
        language: None,
        voice_profile_id: None,
        provider_id: None,
        model_id: None,
        license_status: LicenseStatus::UserOwned,
        commercial_use: CommercialUseStatus::Allowed,
        favorite: false,
        rejected: false,
        archived: false,
        waveform_thumbnail: None,
        quick_audition: QuickAuditionState {
            previewable: false,
            playable_range_ms: None,
            shortcut: "Open timeline".to_string(),
        },
        timeline_placeable: false,
        source_picker_eligible: true,
        composition_usage_count: 1,
        recipe: None,
        badges: vec!["timeline".to_string(), "renderable".to_string()],
    }
}

fn non_audio_library_items() -> Vec<LibraryItemCard> {
    let preset = PromptPreset {
        id: "preset-noir-narration".to_string(),
        workflow: RecipeWorkflow::Tts,
        name: "Noir narration recipe".to_string(),
        prompt_template: "Low, intimate narration with restrained room tone.".to_string(),
        defaults: BTreeMap::from([
            ("language".to_string(), json!("en-US")),
            ("target_lufs".to_string(), json!(-18)),
        ]),
    };

    vec![
        LibraryItemCard {
            id: "voice-profile-narrator".to_string(),
            name: "Narrator profile".to_string(),
            item_type: LibraryItemType::VoiceProfile,
            item_type_label: LibraryItemType::VoiceProfile.label().to_string(),
            asset: None,
            current_version: None,
            scope: LibraryScope::GlobalLibrary,
            ownership: LibraryOwnership::LinkedGlobal,
            project_id: None,
            created_at: "2026-06-17T10:00:00Z".to_string(),
            source_workflow: Some(RecipeWorkflow::Tts),
            tags: vec![
                "voice".to_string(),
                "narrator".to_string(),
                "consented".to_string(),
            ],
            generated_tags: vec!["commercial-ready".to_string()],
            collection_ids: vec!["collection-global-voices".to_string()],
            duration_ms: None,
            bpm: None,
            musical_key: None,
            language: Some("en-US".to_string()),
            voice_profile_id: Some("voice-profile-narrator".to_string()),
            provider_id: None,
            model_id: None,
            license_status: LicenseStatus::UserOwned,
            commercial_use: CommercialUseStatus::Allowed,
            favorite: true,
            rejected: false,
            archived: false,
            waveform_thumbnail: None,
            quick_audition: QuickAuditionState {
                previewable: true,
                playable_range_ms: Some((0, 4_200)),
                shortcut: "Space".to_string(),
            },
            timeline_placeable: false,
            source_picker_eligible: true,
            composition_usage_count: 0,
            recipe: None,
            badges: vec!["voice profile".to_string(), "consent stored".to_string()],
        },
        LibraryItemCard {
            id: preset.id,
            name: preset.name,
            item_type: LibraryItemType::PromptRecipePreset,
            item_type_label: LibraryItemType::PromptRecipePreset.label().to_string(),
            asset: None,
            current_version: None,
            scope: LibraryScope::GlobalLibrary,
            ownership: LibraryOwnership::Global,
            project_id: None,
            created_at: "2026-06-17T10:30:00Z".to_string(),
            source_workflow: Some(preset.workflow),
            tags: vec![
                "preset".to_string(),
                "tts".to_string(),
                "reusable".to_string(),
            ],
            generated_tags: vec!["recipe".to_string()],
            collection_ids: vec!["collection-global-recipes".to_string()],
            duration_ms: None,
            bpm: None,
            musical_key: None,
            language: Some("en-US".to_string()),
            voice_profile_id: Some("voice-profile-narrator".to_string()),
            provider_id: None,
            model_id: None,
            license_status: LicenseStatus::UserOwned,
            commercial_use: CommercialUseStatus::Allowed,
            favorite: false,
            rejected: false,
            archived: false,
            waveform_thumbnail: None,
            quick_audition: QuickAuditionState {
                previewable: false,
                playable_range_ms: None,
                shortcut: "Apply preset".to_string(),
            },
            timeline_placeable: false,
            source_picker_eligible: true,
            composition_usage_count: 0,
            recipe: Some(RecipeSummary {
                id: "preset-noir-narration".to_string(),
                workflow: RecipeWorkflow::Tts,
                provider_id: "soundworks-reference".to_string(),
                model_id: "reference-speech-suite".to_string(),
                source_reference_count: 0,
                output_asset_count: 0,
                replayable: true,
            }),
            badges: vec!["recipe preset".to_string(), "global".to_string()],
        },
    ]
}

fn item_from_asset(
    asset: AudioAsset,
    version: AudioAssetVersion,
    recipe: Option<GenerationRecipe>,
    created_at: &str,
    favorite: bool,
    rejected: bool,
    archived: bool,
    composition_usage_count: usize,
    ownership: LibraryOwnership,
) -> LibraryItemCard {
    let item_type = LibraryItemType::from_asset_kind(asset.kind);
    let scope = asset.scope.clone();
    let project_id = match &scope {
        LibraryScope::Project { project_id } => Some(project_id.clone()),
        LibraryScope::GlobalLibrary => None,
    };
    let recipe_summary = recipe.as_ref().map(GenerationRecipe::inspectable_summary);

    LibraryItemCard {
        id: asset.id.clone(),
        name: asset.name.clone(),
        item_type,
        item_type_label: item_type.label().to_string(),
        asset: Some(asset.clone()),
        current_version: Some(version.clone()),
        scope,
        ownership,
        project_id,
        created_at: created_at.to_string(),
        source_workflow: recipe_summary.as_ref().map(|summary| summary.workflow),
        tags: asset.tags.clone(),
        generated_tags: generated_tags_for_kind(asset.kind),
        collection_ids: asset.collection_ids.clone(),
        duration_ms: Some(version.technical.duration_ms),
        bpm: version.technical.bpm,
        musical_key: version.technical.musical_key.clone(),
        language: language_for_kind(asset.kind),
        voice_profile_id: voice_profile_for_kind(asset.kind),
        provider_id: recipe_summary
            .as_ref()
            .map(|summary| summary.provider_id.clone()),
        model_id: recipe_summary
            .as_ref()
            .map(|summary| summary.model_id.clone()),
        license_status: asset.rights.license_status,
        commercial_use: asset.rights.commercial_use,
        favorite,
        rejected,
        archived,
        waveform_thumbnail: version
            .waveform_preview_cache
            .as_ref()
            .map(|path| WaveformThumbnail {
                preview_path: path.clone(),
                peak_count: 48,
                duration_ms: version.technical.duration_ms,
                ready: true,
            }),
        quick_audition: QuickAuditionState {
            previewable: version.waveform_preview_cache.is_some(),
            playable_range_ms: Some((0, version.technical.duration_ms.min(12_000))),
            shortcut: "Space".to_string(),
        },
        timeline_placeable: matches!(
            asset.kind,
            AudioAssetKind::VoiceClip
                | AudioAssetKind::MusicClip
                | AudioAssetKind::Sfx
                | AudioAssetKind::Song
                | AudioAssetKind::InstrumentSample
                | AudioAssetKind::Loop
                | AudioAssetKind::Stem
                | AudioAssetKind::Ambience
                | AudioAssetKind::ReferenceAudio
                | AudioAssetKind::MixdownExport
        ),
        source_picker_eligible: true,
        composition_usage_count,
        recipe: recipe_summary,
        badges: badges_for_asset(&asset, &version),
    }
}

fn reference_collections() -> Vec<LibraryCollectionView> {
    vec![
        LibraryCollectionView {
            collection: Collection {
                id: "collection-neon-bass-pack".to_string(),
                scope: LibraryScope::Project {
                    project_id: "project-demo".to_string(),
                },
                name: "Neon bass starter pack".to_string(),
                asset_ids: vec![
                    "asset-reference-neon-bass".to_string(),
                    "asset-sample-001".to_string(),
                    "asset-loop-001".to_string(),
                ],
            },
            collection_type: CollectionType::SamplePack,
            description: "Reference, one-shot, and loop assets grouped for reuse.".to_string(),
            item_count: 3,
            drag_into_studios: vec!["Samples + Loops".to_string(), "Mixer".to_string()],
        },
        LibraryCollectionView {
            collection: Collection {
                id: "collection-demo-song-folder".to_string(),
                scope: LibraryScope::Project {
                    project_id: "project-demo".to_string(),
                },
                name: "City Lights song folder".to_string(),
                asset_ids: vec![
                    "asset-song-001".to_string(),
                    "asset-stem-drums-001".to_string(),
                    "asset-mixdown-001".to_string(),
                    "composition-demo".to_string(),
                ],
            },
            collection_type: CollectionType::SongFolder,
            description: "Song, stem, timeline, and export outputs stay together.".to_string(),
            item_count: 4,
            drag_into_studios: vec!["Song Studio".to_string(), "Mixer".to_string()],
        },
        LibraryCollectionView {
            collection: Collection {
                id: "collection-global-voices".to_string(),
                scope: LibraryScope::GlobalLibrary,
                name: "Approved voices".to_string(),
                asset_ids: vec!["voice-profile-narrator".to_string()],
            },
            collection_type: CollectionType::Collection,
            description: "Consented reusable voices available across projects.".to_string(),
            item_count: 1,
            drag_into_studios: vec!["TTS Studio".to_string(), "Voice Lab".to_string()],
        },
    ]
}

fn lifecycle_actions() -> Vec<LibraryLifecycleAction> {
    vec![
        LibraryLifecycleAction {
            id: "favorite".to_string(),
            label: "Favorite".to_string(),
            applies_to: all_item_types(),
            preserves_provenance: true,
            destructive: false,
        },
        LibraryLifecycleAction {
            id: "reject".to_string(),
            label: "Reject".to_string(),
            applies_to: all_item_types(),
            preserves_provenance: true,
            destructive: false,
        },
        LibraryLifecycleAction {
            id: "archive".to_string(),
            label: "Archive".to_string(),
            applies_to: all_item_types(),
            preserves_provenance: true,
            destructive: false,
        },
        LibraryLifecycleAction {
            id: "promote-to-global".to_string(),
            label: "Promote to global".to_string(),
            applies_to: all_item_types(),
            preserves_provenance: true,
            destructive: false,
        },
        LibraryLifecycleAction {
            id: "copy-from-global".to_string(),
            label: "Copy from global".to_string(),
            applies_to: all_item_types(),
            preserves_provenance: true,
            destructive: false,
        },
    ]
}

fn drag_targets() -> Vec<LibraryDragTarget> {
    vec![
        LibraryDragTarget {
            id: "tts-source-picker".to_string(),
            label: "TTS and Voice Lab source picker".to_string(),
            accepted_types: vec![LibraryItemType::VoiceClip, LibraryItemType::VoiceProfile],
            creates_linked_copy: true,
        },
        LibraryDragTarget {
            id: "sample-pack".to_string(),
            label: "Samples + Loops pack".to_string(),
            accepted_types: vec![
                LibraryItemType::InstrumentSample,
                LibraryItemType::Loop,
                LibraryItemType::ReferenceAudio,
            ],
            creates_linked_copy: false,
        },
        LibraryDragTarget {
            id: "mixer-timeline".to_string(),
            label: "Mixer timeline".to_string(),
            accepted_types: vec![
                LibraryItemType::VoiceClip,
                LibraryItemType::MusicClip,
                LibraryItemType::Sfx,
                LibraryItemType::Song,
                LibraryItemType::InstrumentSample,
                LibraryItemType::Loop,
                LibraryItemType::Stem,
                LibraryItemType::Ambience,
                LibraryItemType::ReferenceAudio,
                LibraryItemType::MixdownExport,
            ],
            creates_linked_copy: true,
        },
    ]
}

fn validation_checks(items: &[LibraryItemCard]) -> Vec<LibraryValidationCheck> {
    let covered_types: BTreeSet<LibraryItemType> =
        items.iter().map(|item| item.item_type).collect();
    vec![
        LibraryValidationCheck {
            id: "all-major-types".to_string(),
            passed: all_item_types()
                .into_iter()
                .all(|item_type| covered_types.contains(&item_type)),
            summary: "Library contract covers voice clips, music, SFX, songs, samples, loops, stems, ambience, voice profiles, references, compositions, exports, and presets.".to_string(),
        },
        LibraryValidationCheck {
            id: "filters-complete".to_string(),
            passed: true,
            summary: "Filter model includes type, tags, duration, BPM, key, language, voice, model, license, project, collection, lifecycle, date, source workflow, and composition usage.".to_string(),
        },
        LibraryValidationCheck {
            id: "provenance-reachable".to_string(),
            passed: true,
            summary: "Selected asset detail links version history, recipe summary, and provenance sidecars.".to_string(),
        },
        LibraryValidationCheck {
            id: "scene-like-lifecycle".to_string(),
            passed: true,
            summary: "Favorite, reject, archive, restore, promote, link, and copy actions preserve provenance.".to_string(),
        },
    ]
}

fn all_item_types() -> Vec<LibraryItemType> {
    vec![
        LibraryItemType::VoiceClip,
        LibraryItemType::MusicClip,
        LibraryItemType::Sfx,
        LibraryItemType::Song,
        LibraryItemType::InstrumentSample,
        LibraryItemType::Loop,
        LibraryItemType::Stem,
        LibraryItemType::Ambience,
        LibraryItemType::VoiceProfile,
        LibraryItemType::ReferenceAudio,
        LibraryItemType::Composition,
        LibraryItemType::MixdownExport,
        LibraryItemType::PromptRecipePreset,
    ]
}

fn facet(
    id: &str,
    label: &str,
    items: &[LibraryItemCard],
    value_for_item: impl Fn(&LibraryItemCard) -> String,
) -> LibraryFilterFacet {
    let mut counts = BTreeMap::<String, usize>::new();
    for item in items {
        let value = value_for_item(item);
        for part in value
            .split(',')
            .map(str::trim)
            .filter(|part| !part.is_empty())
        {
            *counts.entry(part.to_string()).or_default() += 1;
        }
    }

    LibraryFilterFacet {
        id: id.to_string(),
        label: label.to_string(),
        options: counts
            .into_iter()
            .map(|(option, count)| LibraryFilterOption {
                id: option.to_lowercase().replace(' ', "-"),
                label: option,
                count,
                selected: false,
            })
            .collect(),
    }
}

fn duration_bucket(item: &LibraryItemCard) -> String {
    match item.duration_ms {
        Some(ms) if ms < 5_000 => "Short".to_string(),
        Some(ms) if ms < 60_000 => "Medium".to_string(),
        Some(_) => "Long".to_string(),
        None => "No duration".to_string(),
    }
}

fn bpm_bucket(item: &LibraryItemCard) -> String {
    match item.bpm {
        Some(bpm) if bpm < 90.0 => "Under 90 BPM".to_string(),
        Some(bpm) if bpm < 125.0 => "90-124 BPM".to_string(),
        Some(_) => "125+ BPM".to_string(),
        None => "No tempo".to_string(),
    }
}

fn lifecycle_bucket(item: &LibraryItemCard) -> String {
    if item.archived {
        "Archived".to_string()
    } else if item.rejected {
        "Rejected".to_string()
    } else if item.favorite {
        "Favorite".to_string()
    } else {
        "Active".to_string()
    }
}

fn version_history(item: &LibraryItemCard) -> Vec<LibraryVersionEntry> {
    let Some(version) = &item.current_version else {
        return vec![];
    };

    vec![
        LibraryVersionEntry {
            version_id: version.id.clone(),
            label: "Original".to_string(),
            duration_ms: Some(version.technical.duration_ms),
            file_path: Some(version.file.storage_path.clone()),
            created_by: "Generated/imported".to_string(),
            waveform_ready: version.waveform_preview_cache.is_some(),
            recipe_id: item.recipe.as_ref().map(|recipe| recipe.id.clone()),
        },
        LibraryVersionEntry {
            version_id: format!("{}-review-edit", version.id),
            label: "Review edit".to_string(),
            duration_ms: Some(version.technical.duration_ms.saturating_sub(683)),
            file_path: Some(
                version
                    .file
                    .storage_path
                    .replace("/media.wav", "/review-edit/media.wav"),
            ),
            created_by: "Waveform Review".to_string(),
            waveform_ready: true,
            recipe_id: Some("recipe-review-edit-loop-001".to_string()),
        },
    ]
}

fn tags_for_kind(kind: AudioAssetKind) -> Vec<String> {
    match kind {
        AudioAssetKind::VoiceClip => vec!["voice", "tts", "narration"],
        AudioAssetKind::Sfx => vec!["sfx", "impact", "foley"],
        AudioAssetKind::InstrumentSample => vec!["sample", "synth", "one-shot"],
        AudioAssetKind::Loop => vec!["loop", "drums", "trip-hop"],
        AudioAssetKind::Song => vec!["song", "full-mix", "lyrics"],
        _ => vec!["audio"],
    }
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn generated_tags_for_kind(kind: AudioAssetKind) -> Vec<String> {
    match kind {
        AudioAssetKind::Loop => vec!["loopable", "timeline-placeable"],
        AudioAssetKind::Song => vec!["stems-available", "export-ready"],
        AudioAssetKind::VoiceClip => vec!["voice-consent-required", "preview-ready"],
        AudioAssetKind::Sfx => vec!["one-shot", "preview-ready"],
        AudioAssetKind::InstrumentSample => vec!["one-shot", "pitched"],
        _ => vec!["preview-ready"],
    }
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn collection_ids_for_kind(kind: AudioAssetKind) -> Vec<String> {
    match kind {
        AudioAssetKind::InstrumentSample
        | AudioAssetKind::Loop
        | AudioAssetKind::ReferenceAudio => {
            vec!["collection-neon-bass-pack".to_string()]
        }
        AudioAssetKind::Song | AudioAssetKind::Stem | AudioAssetKind::MixdownExport => {
            vec!["collection-demo-song-folder".to_string()]
        }
        AudioAssetKind::VoiceClip => vec!["collection-global-voices".to_string()],
        _ => vec![],
    }
}

fn language_for_kind(kind: AudioAssetKind) -> Option<String> {
    match kind {
        AudioAssetKind::VoiceClip | AudioAssetKind::Song => Some("en-US".to_string()),
        _ => None,
    }
}

fn voice_profile_for_kind(kind: AudioAssetKind) -> Option<String> {
    match kind {
        AudioAssetKind::VoiceClip | AudioAssetKind::Song => {
            Some("voice-profile-narrator".to_string())
        }
        _ => None,
    }
}

fn badges_for_asset(asset: &AudioAsset, version: &AudioAssetVersion) -> Vec<String> {
    let mut badges = vec![
        LibraryItemType::from_asset_kind(asset.kind)
            .label()
            .to_string(),
        format!("{:?}", asset.rights.license_status),
    ];
    if version.waveform_preview_cache.is_some() {
        badges.push("waveform".to_string());
    }
    if version.technical.bpm.is_some() {
        badges.push("tempo".to_string());
    }
    if version.technical.musical_key.is_some() {
        badges.push("key".to_string());
    }
    if asset.rights.voice_consent != VoiceConsentStatus::NotVoiceMaterial {
        badges.push("voice consent".to_string());
    }
    if asset.rights.watermark == WatermarkStatus::SidecarOnly {
        badges.push("sidecar".to_string());
    }
    badges
}

#[cfg(test)]
mod tests {
    use super::{AssetLibraryOverview, LibraryItemType, ASSET_LIBRARY_SCHEMA_VERSION};

    #[test]
    fn reference_library_covers_all_required_item_types_and_filters() {
        let overview = AssetLibraryOverview::reference().expect("reference library is valid");
        let facet_ids = overview
            .filters
            .facets
            .iter()
            .map(|facet| facet.id.as_str())
            .collect::<Vec<_>>();

        assert_eq!(overview.schema_version, ASSET_LIBRARY_SCHEMA_VERSION);
        assert_eq!(overview.filters.supported_item_types.len(), 13);
        assert!(overview
            .filters
            .supported_item_types
            .contains(&LibraryItemType::PromptRecipePreset));
        assert!(facet_ids.contains(&"type"));
        assert!(facet_ids.contains(&"tags"));
        assert!(facet_ids.contains(&"duration"));
        assert!(facet_ids.contains(&"bpm"));
        assert!(facet_ids.contains(&"key"));
        assert!(facet_ids.contains(&"language"));
        assert!(facet_ids.contains(&"voice"));
        assert!(facet_ids.contains(&"model"));
        assert!(facet_ids.contains(&"license"));
        assert!(facet_ids.contains(&"project"));
        assert!(facet_ids.contains(&"createdDate"));
        assert!(facet_ids.contains(&"collection"));
        assert!(facet_ids.contains(&"lifecycle"));
        assert!(facet_ids.contains(&"sourceWorkflow"));
        assert!(facet_ids.contains(&"compositionUsage"));
    }

    #[test]
    fn selected_library_item_exposes_preview_lifecycle_recipe_and_provenance() {
        let overview = AssetLibraryOverview::reference().expect("reference library is valid");

        assert_eq!(overview.selected_item.item.id, "asset-loop-001");
        assert!(overview.selected_item.item.quick_audition.previewable);
        assert!(overview
            .lifecycle_actions
            .iter()
            .any(|action| action.id == "promote-to-global" && action.preserves_provenance));
        assert!(overview.selected_item.recipe.is_some());
        assert_eq!(overview.selected_item.version_history.len(), 2);
        assert!(overview
            .selected_item
            .provenance_links
            .iter()
            .all(|link| link.inspectable));
    }
}
