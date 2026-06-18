use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Workspace {
    pub id: String,
    pub global_library_id: String,
    pub recent_project_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    pub storage_root: String,
    pub asset_ids: Vec<String>,
    pub composition_ids: Vec<String>,
    pub recipe_ids: Vec<String>,
    pub job_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum LibraryScope {
    GlobalLibrary,
    Project { project_id: String },
}

impl LibraryScope {
    pub fn storage_prefix(&self) -> String {
        match self {
            Self::GlobalLibrary => "global".to_string(),
            Self::Project { project_id } => format!("projects/{project_id}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AudioAssetKind {
    VoiceClip,
    MusicClip,
    Sfx,
    Song,
    InstrumentSample,
    Loop,
    Stem,
    Ambience,
    ReferenceAudio,
    Composition,
    MixdownExport,
}

impl AudioAssetKind {
    pub fn storage_dir(self) -> &'static str {
        match self {
            Self::VoiceClip => "voice-clips",
            Self::MusicClip => "music-clips",
            Self::Sfx => "sfx",
            Self::Song => "songs",
            Self::InstrumentSample => "instrument-samples",
            Self::Loop => "loops",
            Self::Stem => "stems",
            Self::Ambience => "ambience",
            Self::ReferenceAudio => "reference-audio",
            Self::Composition => "compositions",
            Self::MixdownExport => "mixdowns",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioAsset {
    pub id: String,
    pub scope: LibraryScope,
    pub kind: AudioAssetKind,
    pub name: String,
    pub tags: Vec<String>,
    pub collection_ids: Vec<String>,
    pub current_version_id: String,
    pub version_ids: Vec<String>,
    pub rights: RightsMetadata,
    pub provenance_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioAssetVersion {
    pub id: String,
    pub asset_id: String,
    pub version_index: u32,
    pub file: AudioFileReference,
    pub technical: TechnicalAudioMetadata,
    pub created_by: AssetCreation,
    pub waveform_preview_cache: Option<String>,
    pub spectrogram_preview_cache: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioFileReference {
    pub storage_path: String,
    pub format: AudioFileFormat,
    pub codec: Option<String>,
    pub byte_size: Option<u64>,
    pub content_hash: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AudioFileFormat {
    Wav,
    Flac,
    Mp3,
    Ogg,
    Aiff,
}

impl AudioFileFormat {
    pub fn extension(self) -> &'static str {
        match self {
            Self::Wav => "wav",
            Self::Flac => "flac",
            Self::Mp3 => "mp3",
            Self::Ogg => "ogg",
            Self::Aiff => "aiff",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TechnicalAudioMetadata {
    pub sample_rate_hz: u32,
    pub bit_depth: Option<u16>,
    pub channels: u16,
    pub duration_ms: u64,
    pub loudness_lufs: Option<f32>,
    pub true_peak_dbfs: Option<f32>,
    pub has_clipping: bool,
    pub bpm: Option<f32>,
    pub musical_key: Option<String>,
    pub loop_points: Option<LoopPoints>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoopPoints {
    pub start_sample: u64,
    pub end_sample: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum AssetCreation {
    Imported {
        source_reference_id: String,
    },
    Generated {
        recipe_id: String,
        job_id: String,
    },
    Edited {
        source_version_id: String,
        edit_recipe_id: String,
    },
    RenderedComposition {
        composition_id: String,
        export_id: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RightsMetadata {
    pub license_status: LicenseStatus,
    pub commercial_use: CommercialUseStatus,
    pub voice_consent: VoiceConsentStatus,
    pub ai_disclosure_required: bool,
    pub watermark: WatermarkStatus,
    pub reference_media_ownership: Option<String>,
}

impl RightsMetadata {
    pub fn user_owned_commercial() -> Self {
        Self {
            license_status: LicenseStatus::UserOwned,
            commercial_use: CommercialUseStatus::Allowed,
            voice_consent: VoiceConsentStatus::NotVoiceMaterial,
            ai_disclosure_required: true,
            watermark: WatermarkStatus::NotPresent,
            reference_media_ownership: Some("user-owned".to_string()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LicenseStatus {
    UserOwned,
    ProviderLicensed,
    OpenLicense,
    Restricted,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CommercialUseStatus {
    Allowed,
    RequiresReview,
    Disallowed,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VoiceConsentStatus {
    NotVoiceMaterial,
    ExplicitConsentRecorded,
    ProviderStockVoice,
    RequiresReview,
    Prohibited,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WatermarkStatus {
    NotPresent,
    Embedded,
    SidecarOnly,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerationRecipe {
    pub id: String,
    pub workflow: RecipeWorkflow,
    pub provider: ModelDescriptor,
    pub request: RecipeRequest,
    pub seed: Option<u64>,
    pub source_references: Vec<SourceReference>,
    pub post_processing: Vec<PostProcessingStep>,
    pub parameter_overrides: BTreeMap<String, Value>,
    pub output_asset_ids: Vec<String>,
}

impl GenerationRecipe {
    pub fn inspectable_summary(&self) -> RecipeSummary {
        RecipeSummary {
            id: self.id.clone(),
            workflow: self.workflow,
            provider_id: self.provider.provider_id.clone(),
            model_id: self.provider.model_id.clone(),
            source_reference_count: self.source_references.len(),
            output_asset_count: self.output_asset_ids.len(),
            replayable: self.provider.model_version.is_some() && self.request.has_primary_prompt(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipeSummary {
    pub id: String,
    pub workflow: RecipeWorkflow,
    pub provider_id: String,
    pub model_id: String,
    pub source_reference_count: usize,
    pub output_asset_count: usize,
    pub replayable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RecipeWorkflow {
    Tts,
    VoiceConversion,
    Sfx,
    InstrumentSample,
    Loop,
    Song,
    VideoToAudio,
    Edit,
    CompositionRender,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelDescriptor {
    pub provider_id: String,
    pub model_id: String,
    pub model_version: Option<String>,
    pub model_hash: Option<String>,
    pub runtime: ModelRuntime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ModelRuntime {
    Local,
    ExternalApi,
    ResearchOnly,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum RecipeRequest {
    Tts(TtsRecipe),
    VoiceConversion(VoiceConversionRecipe),
    Sfx(SfxRecipe),
    InstrumentSample(InstrumentSampleRecipe),
    Loop(LoopRecipe),
    Song(SongRecipe),
    VideoToAudio(VideoToAudioRecipe),
    Edit(EditRecipe),
    CompositionRender(CompositionRenderRecipe),
}

impl RecipeRequest {
    fn has_primary_prompt(&self) -> bool {
        match self {
            Self::Tts(recipe) => !recipe.script.trim().is_empty(),
            Self::VoiceConversion(recipe) => !recipe.target_voice_profile_id.trim().is_empty(),
            Self::Sfx(recipe) => !recipe.prompt.trim().is_empty(),
            Self::InstrumentSample(recipe) => !recipe.prompt.trim().is_empty(),
            Self::Loop(recipe) => !recipe.prompt.trim().is_empty(),
            Self::Song(recipe) => {
                !recipe.prompt.trim().is_empty() || !recipe.lyrics.trim().is_empty()
            }
            Self::VideoToAudio(recipe) => !recipe.prompt.trim().is_empty(),
            Self::Edit(recipe) => !recipe.edit_instructions.trim().is_empty(),
            Self::CompositionRender(recipe) => !recipe.composition_id.trim().is_empty(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TtsRecipe {
    pub script: String,
    pub language: Option<String>,
    pub speaker_labels: Vec<String>,
    pub voice_profile_id: Option<String>,
    pub pronunciation_notes: Vec<String>,
    pub target_duration_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceConversionRecipe {
    pub source_audio_asset_id: String,
    pub target_voice_profile_id: String,
    pub preserve_timing: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SfxRecipe {
    pub prompt: String,
    pub negative_prompt: Option<String>,
    pub category: Option<String>,
    pub target_duration_ms: Option<u64>,
    pub loopable: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstrumentSampleRecipe {
    pub prompt: String,
    pub instrument: Option<String>,
    pub pitch: Option<String>,
    pub velocity: Option<u8>,
    pub target_duration_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoopRecipe {
    pub prompt: String,
    pub bpm: f32,
    pub musical_key: Option<String>,
    pub bars: u16,
    pub loopable: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SongRecipe {
    pub prompt: String,
    pub lyrics: String,
    pub style_tags: Vec<String>,
    pub structure: SongStructure,
    pub requested_stems: Vec<StemKind>,
    pub reference_audio_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SongStructure {
    pub sections: Vec<SongSection>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SongSection {
    pub label: String,
    pub lyrics: Option<String>,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StemKind {
    Vocals,
    Drums,
    Bass,
    Instruments,
    Effects,
    FullMix,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoToAudioRecipe {
    pub prompt: String,
    pub source_video_reference_id: String,
    pub sync_mode: SyncMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SyncMode {
    SemanticMatch,
    FrameSynchronized,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditRecipe {
    pub source_asset_version_id: String,
    pub edit_instructions: String,
    pub trim: Option<TimeRange>,
    pub normalize_loudness_lufs: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompositionRenderRecipe {
    pub composition_id: String,
    pub preset_id: String,
    pub include_sidecar: bool,
    pub stems: Vec<StemKind>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceReference {
    pub id: String,
    pub source_type: SourceReferenceType,
    pub asset_id: Option<String>,
    pub external_uri: Option<String>,
    pub ownership_note: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SourceReferenceType {
    Voice,
    Audio,
    Video,
    Image,
    Text,
    ExternalModel,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostProcessingStep {
    pub id: String,
    pub operation: PostProcessingOperation,
    pub parameters: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PostProcessingOperation {
    Normalize,
    Trim,
    Fade,
    RemoveSilence,
    LoopCrossfade,
    Denoise,
    Master,
    ConvertFormat,
    EditMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceProfile {
    pub id: String,
    pub display_name: String,
    pub source_reference_ids: Vec<String>,
    pub consent: VoiceConsentStatus,
    pub allowed_uses: Vec<VoiceUse>,
    pub provenance_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VoiceUse {
    Tts,
    VoiceConversion,
    FineTuning,
    ProjectOnly,
    Commercial,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptPreset {
    pub id: String,
    pub workflow: RecipeWorkflow,
    pub name: String,
    pub prompt_template: String,
    pub defaults: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Collection {
    pub id: String,
    pub scope: LibraryScope,
    pub name: String,
    pub asset_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerationJob {
    pub id: String,
    pub recipe_id: String,
    pub kind: JobKind,
    pub status: JobStatus,
    pub progress: Option<JobProgress>,
    pub output_version_ids: Vec<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum JobKind {
    GenerateAudio,
    RenderComposition,
    Export,
    EvaluateModel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum JobStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobProgress {
    pub percent: f32,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Composition {
    pub id: String,
    pub scope: LibraryScope,
    pub name: String,
    pub tempo_bpm: Option<f32>,
    pub musical_key: Option<String>,
    pub tracks: Vec<CompositionTrack>,
    pub markers: Vec<TimelineMarker>,
    pub sections: Vec<TimelineSection>,
    pub export_history: Vec<CompositionExport>,
    pub provenance_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompositionTrack {
    pub id: String,
    pub name: String,
    pub role: TrackRole,
    pub clips: Vec<CompositionClip>,
    pub gain_db: f32,
    pub pan: f32,
    pub muted: bool,
    pub soloed: bool,
    pub automation: Vec<AutomationLane>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TrackRole {
    Voice,
    Music,
    Sfx,
    Ambience,
    Stem,
    Master,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompositionClip {
    pub id: String,
    pub asset_id: String,
    pub version_id: String,
    pub timeline_start_ms: u64,
    pub source_range: TimeRange,
    pub fade_in_ms: u64,
    pub fade_out_ms: u64,
    pub gain_db: f32,
    pub pan: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeRange {
    pub start_ms: u64,
    pub end_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationLane {
    pub target: AutomationTarget,
    pub points: Vec<AutomationPoint>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AutomationTarget {
    Gain,
    Pan,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationPoint {
    pub at_ms: u64,
    pub value: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineMarker {
    pub id: String,
    pub at_ms: u64,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineSection {
    pub id: String,
    pub range: TimeRange,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompositionExport {
    pub id: String,
    pub job_id: String,
    pub output_asset_id: String,
    pub preset_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportPreset {
    pub id: String,
    pub name: String,
    pub format: AudioFileFormat,
    pub sample_rate_hz: u32,
    pub bit_depth: Option<u16>,
    pub include_sidecar: bool,
    pub include_stems: bool,
    pub target: ExportTarget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExportTarget {
    AudioFile,
    StemFolder,
    DawHandoff,
    SceneWorksVideoTrack,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProvenanceRecord {
    pub id: String,
    pub subject_id: String,
    pub events: Vec<ProvenanceEvent>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProvenanceEvent {
    pub event_type: ProvenanceEventType,
    pub actor: String,
    pub summary: String,
    pub metadata: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProvenanceEventType {
    Imported,
    Generated,
    Edited,
    Rendered,
    Exported,
    RightsReviewed,
}
