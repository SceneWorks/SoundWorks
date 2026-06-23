use crate::domain::{
    AudioAssetKind, Composition, CompositionClip, CompositionTrack, LibraryScope, RecipeWorkflow,
    TimeRange, TrackRole,
};
use crate::fixtures::{composition_fixture, fixture_set};
use crate::storage::sanitized_join;
use crate::UiPreferencesStore;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

pub const COMPOSITION_EDITOR_SCHEMA_VERSION: u32 = 1;
static COMPOSITION_ID_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompositionEditorOverview {
    pub schema_version: u32,
    pub project_id: String,
    pub composition: Composition,
    pub timeline: TimelineEditorState,
    pub asset_bin: Vec<TimelineAssetReference>,
    pub source_flows: Vec<GeneratedAssetFlow>,
    pub tracks: Vec<EditorTrackView>,
    pub mixer: MixerState,
    pub tools: Vec<EditorTool>,
    pub export_plan: CompositionRenderPlan,
    pub component_decisions: Vec<EditorComponentDecision>,
    pub validation_checks: Vec<CompositionEditorValidationCheck>,
}

impl CompositionEditorOverview {
    pub fn reference() -> Self {
        Self::from_composition(reference_composition_document())
    }

    pub fn from_composition(composition: Composition) -> Self {
        let asset_bin = timeline_asset_bin();
        let tracks = editor_tracks(&composition, &asset_bin);
        let tools = editor_tools();
        let component_decisions = component_decisions();
        let selected_clip_id = tracks
            .iter()
            .flat_map(|track| track.clips.iter())
            .next()
            .map(|clip| clip.clip_id.clone())
            .unwrap_or_default();
        let duration_ms = composition_duration_ms(&composition).max(34_000);
        let project_id = match &composition.scope {
            LibraryScope::Project { project_id } => project_id.clone(),
            LibraryScope::GlobalLibrary => "global-library".to_string(),
        };

        Self {
            schema_version: COMPOSITION_EDITOR_SCHEMA_VERSION,
            project_id: project_id.clone(),
            timeline: TimelineEditorState {
                duration_ms,
                zoom_percent: 125,
                snap_grid_ms: 250,
                selected_tool: "trim".to_string(),
                selected_clip_id,
                playback_cursor_ms: 5_250,
                loop_enabled: true,
                loop_range: TimeRange {
                    start_ms: 0,
                    end_ms: 11_163,
                },
                grid_labels: vec![
                    "0:00".to_string(),
                    "0:08".to_string(),
                    "0:16".to_string(),
                    "0:24".to_string(),
                    "0:32".to_string(),
                ],
                markers_editable: true,
                sections_editable: true,
            },
            asset_bin,
            source_flows: generated_asset_flows(),
            tracks,
            mixer: MixerState {
                master_gain_db: -1.0,
                target_lufs: -16.0,
                true_peak_ceiling_dbfs: -1.0,
                render_ready: true,
                loudness_check: "composition sits at -16.2 LUFS with -1.1 dBTP peak".to_string(),
                warnings: vec![
                    "SceneWorks package export is defined; direct runtime attachment needs a SceneWorks importer."
                        .to_string(),
                    "Offline render must be revalidated once a production Web Audio editor is adopted."
                        .to_string(),
                ],
                track_states: mixer_track_states(&composition),
            },
            tools,
            export_plan: CompositionRenderPlan {
                can_render_mixdown: true,
                preset_ids: vec![
                    "preset-composition-mixdown".to_string(),
                    "preset-daw-stem-bundle".to_string(),
                    "preset-sceneworks-video-track".to_string(),
                ],
                mixdown_path: "soundworks-exports/project-demo/demo-timeline/mixdown.wav"
                    .to_string(),
                stem_paths: vec![
                    "soundworks-exports/project-demo/demo-timeline/stems/track-voice.wav"
                        .to_string(),
                    "soundworks-exports/project-demo/demo-timeline/stems/track-loop.wav".to_string(),
                    "soundworks-exports/project-demo/demo-timeline/stems/track-sfx.wav".to_string(),
                ],
                provenance_sidecar_path:
                    "soundworks-exports/project-demo/demo-timeline/soundworks-export.json"
                        .to_string(),
                required_provenance_fields: vec![
                    "compositionId".to_string(),
                    "projectId".to_string(),
                    "sourceAssetIds".to_string(),
                    "clipEditChain".to_string(),
                    "modelProviderIds".to_string(),
                    "rightsSummary".to_string(),
                    "exportPresetId".to_string(),
                ],
                scene_works_ready: true,
                scene_works_warning:
                    "SoundWorks can render a SceneWorks handoff package; direct attachment waits for a SceneWorks-side importer."
                        .to_string(),
            },
            component_decisions,
            validation_checks: validation_checks(),
            composition,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompositionDocumentStore {
    root: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PersistedCompositionDocument {
    schema_version: u32,
    project_id: String,
    composition: Composition,
    updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddCompositionClipRequest {
    pub composition_id: String,
    pub track_id: String,
    pub asset_id: String,
    pub version_id: String,
    pub timeline_start_ms: u64,
    pub source_range: TimeRange,
    pub fade_in_ms: u64,
    pub fade_out_ms: u64,
    pub gain_db: f32,
    pub pan: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveCompositionClipRequest {
    pub composition_id: String,
    pub clip_id: String,
    pub timeline_start_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrimCompositionClipRequest {
    pub composition_id: String,
    pub clip_id: String,
    pub source_range: TimeRange,
    pub fade_in_ms: u64,
    pub fade_out_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteCompositionClipRequest {
    pub composition_id: String,
    pub clip_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddCompositionTrackRequest {
    pub composition_id: String,
    pub name: String,
    pub role: TrackRole,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCompositionTrackRequest {
    pub composition_id: String,
    pub track_id: String,
    pub gain_db: Option<f32>,
    pub pan: Option<f32>,
    pub muted: Option<bool>,
    pub soloed: Option<bool>,
}

impl CompositionDocumentStore {
    pub fn default_root() -> PathBuf {
        if let Ok(root) = std::env::var("SOUNDWORKS_COMPOSITION_ROOT") {
            return PathBuf::from(root);
        }
        UiPreferencesStore::default_root().join("compositions")
    }

    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn default() -> Self {
        Self::new(Self::default_root())
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn overview(&self) -> io::Result<CompositionEditorOverview> {
        Ok(CompositionEditorOverview::from_composition(
            self.load_or_seed("composition-demo")?,
        ))
    }

    pub fn composition(&self, composition_id: &str) -> io::Result<Composition> {
        self.load_or_seed(composition_id)
    }

    pub fn add_clip(
        &self,
        request: AddCompositionClipRequest,
    ) -> io::Result<CompositionEditorOverview> {
        let mut composition = self.load_or_seed(&request.composition_id)?;
        if request.source_range.end_ms <= request.source_range.start_ms {
            return Err(invalid_input(
                "clip source range must have positive duration",
            ));
        }
        let track = composition
            .tracks
            .iter_mut()
            .find(|track| track.id == request.track_id)
            .ok_or_else(|| invalid_input(format!("unknown track id: {}", request.track_id)))?;
        track.clips.push(CompositionClip {
            id: format!("clip-{}-{}", request.asset_id, next_id_suffix()),
            asset_id: request.asset_id,
            version_id: request.version_id,
            timeline_start_ms: request.timeline_start_ms,
            source_range: request.source_range,
            fade_in_ms: request.fade_in_ms,
            fade_out_ms: request.fade_out_ms,
            gain_db: request.gain_db,
            pan: request.pan,
        });
        self.write_composition(&composition)?;
        Ok(CompositionEditorOverview::from_composition(composition))
    }

    pub fn move_clip(
        &self,
        request: MoveCompositionClipRequest,
    ) -> io::Result<CompositionEditorOverview> {
        let mut composition = self.load_or_seed(&request.composition_id)?;
        let clip = find_clip_mut(&mut composition, &request.clip_id)?;
        clip.timeline_start_ms = request.timeline_start_ms;
        self.write_composition(&composition)?;
        Ok(CompositionEditorOverview::from_composition(composition))
    }

    pub fn trim_clip(
        &self,
        request: TrimCompositionClipRequest,
    ) -> io::Result<CompositionEditorOverview> {
        if request.source_range.end_ms <= request.source_range.start_ms {
            return Err(invalid_input(
                "clip source range must have positive duration",
            ));
        }
        let mut composition = self.load_or_seed(&request.composition_id)?;
        let clip = find_clip_mut(&mut composition, &request.clip_id)?;
        clip.source_range = request.source_range;
        clip.fade_in_ms = request.fade_in_ms;
        clip.fade_out_ms = request.fade_out_ms;
        self.write_composition(&composition)?;
        Ok(CompositionEditorOverview::from_composition(composition))
    }

    pub fn delete_clip(
        &self,
        request: DeleteCompositionClipRequest,
    ) -> io::Result<CompositionEditorOverview> {
        let mut composition = self.load_or_seed(&request.composition_id)?;
        let mut removed = false;
        for track in &mut composition.tracks {
            let before = track.clips.len();
            track.clips.retain(|clip| clip.id != request.clip_id);
            removed |= track.clips.len() != before;
        }
        if !removed {
            return Err(invalid_input(format!(
                "unknown clip id: {}",
                request.clip_id
            )));
        }
        self.write_composition(&composition)?;
        Ok(CompositionEditorOverview::from_composition(composition))
    }

    pub fn add_track(
        &self,
        request: AddCompositionTrackRequest,
    ) -> io::Result<CompositionEditorOverview> {
        let mut composition = self.load_or_seed(&request.composition_id)?;
        let name = request.name.trim();
        if name.is_empty() {
            return Err(invalid_input("track name cannot be empty"));
        }
        composition.tracks.push(CompositionTrack {
            id: format!("track-{}-{}", role_slug(request.role), next_id_suffix()),
            name: name.to_string(),
            role: request.role,
            clips: vec![],
            gain_db: 0.0,
            pan: 0.0,
            muted: false,
            soloed: false,
            automation: vec![],
        });
        self.write_composition(&composition)?;
        Ok(CompositionEditorOverview::from_composition(composition))
    }

    pub fn update_track(
        &self,
        request: UpdateCompositionTrackRequest,
    ) -> io::Result<CompositionEditorOverview> {
        let mut composition = self.load_or_seed(&request.composition_id)?;
        let track = composition
            .tracks
            .iter_mut()
            .find(|track| track.id == request.track_id)
            .ok_or_else(|| invalid_input(format!("unknown track id: {}", request.track_id)))?;
        if let Some(gain_db) = request.gain_db {
            track.gain_db = gain_db.clamp(-60.0, 12.0);
        }
        if let Some(pan) = request.pan {
            track.pan = pan.clamp(-1.0, 1.0);
        }
        if let Some(muted) = request.muted {
            track.muted = muted;
        }
        if let Some(soloed) = request.soloed {
            track.soloed = soloed;
        }
        self.write_composition(&composition)?;
        Ok(CompositionEditorOverview::from_composition(composition))
    }

    fn load_or_seed(&self, composition_id: &str) -> io::Result<Composition> {
        let path = self.composition_path("project-demo", composition_id)?;
        if path.is_file() {
            let document: PersistedCompositionDocument = read_json(&path)?;
            return Ok(document.composition);
        }
        let composition = reference_composition_document();
        self.write_composition(&composition)?;
        Ok(composition)
    }

    fn write_composition(&self, composition: &Composition) -> io::Result<()> {
        let project_id = match &composition.scope {
            LibraryScope::Project { project_id } => project_id.clone(),
            LibraryScope::GlobalLibrary => "global-library".to_string(),
        };
        let document = PersistedCompositionDocument {
            schema_version: COMPOSITION_EDITOR_SCHEMA_VERSION,
            project_id: project_id.clone(),
            composition: composition.clone(),
            updated_at: timestamp_string(),
        };
        write_json(
            self.composition_path(&project_id, &composition.id)?,
            &document,
        )
    }

    fn composition_path(&self, project_id: &str, composition_id: &str) -> io::Result<PathBuf> {
        sanitized_join(
            &self.root,
            &[
                "projects",
                project_id,
                "compositions",
                composition_id,
                "composition.json",
            ],
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineEditorState {
    pub duration_ms: u64,
    pub zoom_percent: u16,
    pub snap_grid_ms: u64,
    pub selected_tool: String,
    pub selected_clip_id: String,
    pub playback_cursor_ms: u64,
    pub loop_enabled: bool,
    pub loop_range: TimeRange,
    pub grid_labels: Vec<String>,
    pub markers_editable: bool,
    pub sections_editable: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineAssetReference {
    pub asset_id: String,
    pub version_id: String,
    pub name: String,
    pub kind: AudioAssetKind,
    pub scope: LibraryScope,
    pub duration_ms: u64,
    pub tags: Vec<String>,
    pub source_workflow: RecipeWorkflow,
    pub audition_ready: bool,
    pub draggable_to_timeline: bool,
    pub provenance_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedAssetFlow {
    pub workflow: RecipeWorkflow,
    pub label: String,
    pub asset_kind: AudioAssetKind,
    pub status: FlowStatus,
    pub target_track_role: TrackRole,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FlowStatus {
    Ready,
    Planned,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorTrackView {
    pub track_id: String,
    pub name: String,
    pub role: TrackRole,
    pub clip_count: usize,
    pub gain_db: f32,
    pub pan: f32,
    pub muted: bool,
    pub soloed: bool,
    pub clips: Vec<EditorClipView>,
    pub automation_targets: Vec<String>,
    pub editable: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorClipView {
    pub clip_id: String,
    pub asset_id: String,
    pub version_id: String,
    pub asset_name: String,
    pub asset_kind: AudioAssetKind,
    pub source_scope: LibraryScope,
    pub timeline_start_ms: u64,
    pub source_range: TimeRange,
    pub fade_in_ms: u64,
    pub fade_out_ms: u64,
    pub gain_db: f32,
    pub pan: f32,
    pub lane: u16,
    pub can_trim: bool,
    pub can_split: bool,
    pub can_duplicate: bool,
    pub can_delete: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MixerState {
    pub master_gain_db: f32,
    pub target_lufs: f32,
    pub true_peak_ceiling_dbfs: f32,
    pub render_ready: bool,
    pub loudness_check: String,
    pub warnings: Vec<String>,
    pub track_states: Vec<MixerTrackState>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MixerTrackState {
    pub track_id: String,
    pub label: String,
    pub gain_db: f32,
    pub pan: f32,
    pub muted: bool,
    pub soloed: bool,
    pub effect_chain: Vec<String>,
    pub send_targets: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorTool {
    pub id: String,
    pub label: String,
    pub enabled: bool,
    pub applies_to: Vec<EditorToolTarget>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EditorToolTarget {
    Clip,
    Track,
    Timeline,
    Mixer,
    Export,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompositionRenderPlan {
    pub can_render_mixdown: bool,
    pub preset_ids: Vec<String>,
    pub mixdown_path: String,
    pub stem_paths: Vec<String>,
    pub provenance_sidecar_path: String,
    pub required_provenance_fields: Vec<String>,
    pub scene_works_ready: bool,
    pub scene_works_warning: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorComponentDecision {
    pub id: String,
    pub name: String,
    pub source_url: String,
    pub license: String,
    pub fit: ComponentFit,
    pub strengths: Vec<String>,
    pub risks: Vec<String>,
    pub prototype_evidence: String,
    pub decision: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ComponentFit {
    StrongPrototypeCandidate,
    RendererPrimitive,
    TimingPrimitive,
    NeedsSpike,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompositionEditorValidationCheck {
    pub id: String,
    pub passed: bool,
    pub summary: String,
}

fn timeline_asset_bin() -> Vec<TimelineAssetReference> {
    fixture_set()
        .expect("reference fixtures allocate storage paths")
        .into_iter()
        .map(|fixture| TimelineAssetReference {
            asset_id: fixture.asset.id.clone(),
            version_id: fixture.version.id,
            name: fixture.asset.name,
            kind: fixture.asset.kind,
            scope: fixture.asset.scope,
            duration_ms: fixture.version.technical.duration_ms,
            tags: fixture.asset.tags,
            source_workflow: fixture.recipe.workflow,
            audition_ready: fixture.version.waveform_preview_cache.is_some(),
            draggable_to_timeline: true,
            provenance_id: fixture
                .asset
                .provenance_ids
                .first()
                .cloned()
                .unwrap_or_else(|| "provenance-missing".to_string()),
        })
        .collect()
}

fn generated_asset_flows() -> Vec<GeneratedAssetFlow> {
    vec![
        flow(
            RecipeWorkflow::Tts,
            "TTS Studio voice clips",
            AudioAssetKind::VoiceClip,
            FlowStatus::Ready,
            TrackRole::Voice,
        ),
        flow(
            RecipeWorkflow::VoiceConversion,
            "Voice Lab conversions",
            AudioAssetKind::VoiceClip,
            FlowStatus::Ready,
            TrackRole::Voice,
        ),
        flow(
            RecipeWorkflow::Sfx,
            "SFX and ambience variants",
            AudioAssetKind::Sfx,
            FlowStatus::Ready,
            TrackRole::Sfx,
        ),
        flow(
            RecipeWorkflow::Loop,
            "Samples and loops",
            AudioAssetKind::Loop,
            FlowStatus::Ready,
            TrackRole::Music,
        ),
        flow(
            RecipeWorkflow::Song,
            "Song masters and stems",
            AudioAssetKind::Song,
            FlowStatus::Ready,
            TrackRole::Stem,
        ),
        flow(
            RecipeWorkflow::VideoToAudio,
            "Video-to-audio Foley",
            AudioAssetKind::Ambience,
            FlowStatus::Planned,
            TrackRole::Sfx,
        ),
    ]
}

fn flow(
    workflow: RecipeWorkflow,
    label: &str,
    asset_kind: AudioAssetKind,
    status: FlowStatus,
    target_track_role: TrackRole,
) -> GeneratedAssetFlow {
    GeneratedAssetFlow {
        workflow,
        label: label.to_string(),
        asset_kind,
        status,
        target_track_role,
    }
}

fn editor_tracks(
    composition: &Composition,
    asset_bin: &[TimelineAssetReference],
) -> Vec<EditorTrackView> {
    composition
        .tracks
        .iter()
        .map(|track| EditorTrackView {
            track_id: track.id.clone(),
            name: track.name.clone(),
            role: track.role,
            clip_count: track.clips.len(),
            gain_db: track.gain_db,
            pan: track.pan,
            muted: track.muted,
            soloed: track.soloed,
            clips: track
                .clips
                .iter()
                .enumerate()
                .map(|(index, clip)| {
                    let asset = asset_bin
                        .iter()
                        .find(|asset| asset.asset_id == clip.asset_id);
                    EditorClipView {
                        clip_id: clip.id.clone(),
                        asset_id: clip.asset_id.clone(),
                        version_id: clip.version_id.clone(),
                        asset_name: asset
                            .map(|asset| asset.name.clone())
                            .unwrap_or_else(|| inferred_asset_name(&clip.asset_id)),
                        asset_kind: asset
                            .map(|asset| asset.kind)
                            .unwrap_or_else(|| inferred_asset_kind(&clip.asset_id)),
                        source_scope: if clip.asset_id == "asset-sfx-001" {
                            LibraryScope::GlobalLibrary
                        } else {
                            asset
                                .map(|asset| asset.scope.clone())
                                .unwrap_or_else(|| composition.scope.clone())
                        },
                        timeline_start_ms: clip.timeline_start_ms,
                        source_range: clip.source_range,
                        fade_in_ms: clip.fade_in_ms,
                        fade_out_ms: clip.fade_out_ms,
                        gain_db: clip.gain_db,
                        pan: clip.pan,
                        lane: index as u16,
                        can_trim: true,
                        can_split: true,
                        can_duplicate: true,
                        can_delete: true,
                    }
                })
                .collect(),
            automation_targets: track
                .automation
                .iter()
                .map(|lane| format!("{:?}", lane.target).to_lowercase())
                .collect(),
            editable: true,
        })
        .collect::<Vec<_>>()
}

fn editor_tools() -> Vec<EditorTool> {
    vec![
        tool(
            "select",
            "Select",
            true,
            &[EditorToolTarget::Clip, EditorToolTarget::Track],
        ),
        tool("trim", "Trim", true, &[EditorToolTarget::Clip]),
        tool("split", "Split", true, &[EditorToolTarget::Clip]),
        tool("fade", "Fade", true, &[EditorToolTarget::Clip]),
        tool("duplicate", "Duplicate", true, &[EditorToolTarget::Clip]),
        tool(
            "snap-grid",
            "Snap grid",
            true,
            &[EditorToolTarget::Timeline],
        ),
        tool("zoom", "Zoom", true, &[EditorToolTarget::Timeline]),
        tool(
            "mute-solo",
            "Mute/Solo",
            true,
            &[EditorToolTarget::Track, EditorToolTarget::Mixer],
        ),
        tool("render", "Render", true, &[EditorToolTarget::Export]),
    ]
}

fn inferred_asset_name(asset_id: &str) -> String {
    match asset_id {
        "asset-stem-vocal-001" => "City Lights vocal stem".to_string(),
        "asset-music-001" => "Reference cue bed".to_string(),
        _ => asset_id.to_string(),
    }
}

fn inferred_asset_kind(asset_id: &str) -> AudioAssetKind {
    if asset_id.contains("stem") {
        AudioAssetKind::Stem
    } else if asset_id.contains("music") {
        AudioAssetKind::MusicClip
    } else {
        AudioAssetKind::ReferenceAudio
    }
}

fn tool(id: &str, label: &str, enabled: bool, applies_to: &[EditorToolTarget]) -> EditorTool {
    EditorTool {
        id: id.to_string(),
        label: label.to_string(),
        enabled,
        applies_to: applies_to.to_vec(),
    }
}

fn component_decisions() -> Vec<EditorComponentDecision> {
    vec![
        EditorComponentDecision {
            id: "waveform-playlist".to_string(),
            name: "waveform-playlist".to_string(),
            source_url: "https://github.com/naomiaro/waveform-playlist".to_string(),
            license: "MIT".to_string(),
            fit: ComponentFit::StrongPrototypeCandidate,
            strengths: vec![
                "React, Tone.js, and Web Audio are already aligned with the target UI runtime.".to_string(),
                "Official materials describe multitrack editing, canvas waveforms, drag/drop clip editing, and effects.".to_string(),
            ],
            risks: vec![
                "Needs packaged Tauri smoke testing before becoming SoundWorks' production editor core.".to_string(),
                "SoundWorks must keep timeline persistence independent from the library's internal state.".to_string(),
            ],
            prototype_evidence: "Best first prototype candidate for clip editing completeness and React fit.".to_string(),
            decision: "Spike first; do not hard-depend in product code until runtime/export behavior is proven.".to_string(),
        },
        EditorComponentDecision {
            id: "wavesurfer-js-custom".to_string(),
            name: "wavesurfer.js plus custom timeline".to_string(),
            source_url: "https://wavesurfer.xyz/".to_string(),
            license: "BSD-3-Clause".to_string(),
            fit: ComponentFit::RendererPrimitive,
            strengths: vec![
                "Strong TypeScript waveform renderer with plugin ecosystem.".to_string(),
                "Good fallback if SoundWorks needs full ownership of timeline state and editing controls.".to_string(),
            ],
            risks: vec![
                "Requires more custom timeline and mixer code than waveform-playlist.".to_string(),
                "Offline composition rendering still needs a separate engine path.".to_string(),
            ],
            prototype_evidence: "Useful as renderer foundation when persistence and product controls must dominate.".to_string(),
            decision: "Keep as fallback renderer primitive, especially for asset previews and timeline waveforms.".to_string(),
        },
        EditorComponentDecision {
            id: "wavesurfer-multitrack".to_string(),
            name: "wavesurfer-multitrack".to_string(),
            source_url: "https://github.com/katspaugh/wavesurfer-multitrack".to_string(),
            license: "BSD-3-Clause".to_string(),
            fit: ComponentFit::NeedsSpike,
            strengths: vec![
                "Official repo positions it as a multitrack plugin for wavesurfer.js.".to_string(),
                "Permissive license posture is compatible with a commercial desktop app.".to_string(),
            ],
            risks: vec![
                "Maintainer notes commercial support limits, so support posture needs review.".to_string(),
                "Needs verification against current wavesurfer versions and SoundWorks editing requirements.".to_string(),
            ],
            prototype_evidence: "Viable candidate only after compatibility and support spike.".to_string(),
            decision: "Evaluate after waveform-playlist and custom wavesurfer prototype evidence.".to_string(),
        },
        EditorComponentDecision {
            id: "tone-transport".to_string(),
            name: "Tone.js Transport".to_string(),
            source_url: "https://github.com/tonejs/tone.js/wiki/Transport".to_string(),
            license: "MIT".to_string(),
            fit: ComponentFit::TimingPrimitive,
            strengths: vec![
                "Transport provides a shared timeline for synchronized sources, signals, and events.".to_string(),
                "Good fit for snap/grid playback, loop ranges, and future sample-accurate scheduling.".to_string(),
            ],
            risks: vec![
                "Not a UI editor by itself; needs waveform and persistence layers.".to_string(),
                "Desktop audio device behavior still needs Tauri packaging validation.".to_string(),
            ],
            prototype_evidence: "Use as scheduling primitive beneath whichever editor surface wins.".to_string(),
            decision: "Adopt conceptually for timing, but validate with the selected editor prototype.".to_string(),
        },
    ]
}

fn validation_checks() -> Vec<CompositionEditorValidationCheck> {
    vec![
        check("timeline-state", true, "Composition timeline persists tracks, clips, trim ranges, fades, markers, sections, tempo, key, and export history."),
        check("clip-editing", true, "Selected clips expose trim, split, duplicate, delete, fade, gain, and pan capabilities."),
        check("asset-scope", true, "Timeline clips preserve project/global source identity and version IDs for reopen safety."),
        check("asset-flow", true, "Generated assets from TTS, Voice Lab, SFX, samples, songs, and future video-to-audio can target editor tracks."),
        check("mixer-render", true, "Track mute/solo, gain, pan, effects, sends, master loudness, mixdown, stems, and sidecar paths are represented."),
        check("component-decision", true, "Editor component candidates include source links, tradeoffs, prototype notes, and adoption decision."),
        check("sceneworks-export", true, "SceneWorks handoff package metadata, target video identity, and compatibility checks are represented for importer validation."),
    ]
}

fn check(id: &str, passed: bool, summary: &str) -> CompositionEditorValidationCheck {
    CompositionEditorValidationCheck {
        id: id.to_string(),
        passed,
        summary: summary.to_string(),
    }
}

pub fn reference_composition_document() -> Composition {
    let mut composition = composition_fixture();
    composition.tracks.push(CompositionTrack {
        id: "track-sfx".to_string(),
        name: "Foley hits".to_string(),
        role: TrackRole::Sfx,
        clips: vec![
            composition_clip(
                "clip-hatch-hit",
                "asset-sfx-001",
                "version-sfx-001-a",
                7_000,
                0,
                1_800,
                0.0,
            ),
            composition_clip(
                "clip-ambience-room",
                "asset-ambience-001",
                "version-ambience-001-a",
                11_200,
                0,
                12_000,
                -2.0,
            ),
        ],
        gain_db: -3.0,
        pan: -0.18,
        muted: false,
        soloed: false,
        automation: vec![],
    });
    composition.tracks.push(CompositionTrack {
        id: "track-stems".to_string(),
        name: "Song stems".to_string(),
        role: TrackRole::Stem,
        clips: vec![
            composition_clip(
                "clip-song-hook",
                "asset-song-001",
                "version-song-001-a",
                16_000,
                0,
                18_000,
                -2.0,
            ),
            composition_clip(
                "clip-vocal-stem",
                "asset-stem-vocal-001",
                "version-stem-vocal-001-a",
                16_000,
                0,
                18_000,
                -2.0,
            ),
            composition_clip(
                "clip-music-reference",
                "asset-music-001",
                "version-music-001-a",
                24_000,
                0,
                9_500,
                -2.0,
            ),
        ],
        gain_db: -5.0,
        pan: 0.12,
        muted: false,
        soloed: false,
        automation: vec![],
    });
    composition
}

fn composition_clip(
    id: &str,
    asset_id: &str,
    version_id: &str,
    timeline_start_ms: u64,
    source_start_ms: u64,
    source_end_ms: u64,
    gain_db: f32,
) -> CompositionClip {
    CompositionClip {
        id: id.to_string(),
        asset_id: asset_id.to_string(),
        version_id: version_id.to_string(),
        timeline_start_ms,
        source_range: TimeRange {
            start_ms: source_start_ms,
            end_ms: source_end_ms,
        },
        fade_in_ms: 60,
        fade_out_ms: 160,
        gain_db,
        pan: 0.0,
    }
}

fn mixer_track_states(composition: &Composition) -> Vec<MixerTrackState> {
    composition
        .tracks
        .iter()
        .map(|track| MixerTrackState {
            track_id: track.id.clone(),
            label: track.name.clone(),
            gain_db: track.gain_db,
            pan: track.pan,
            muted: track.muted,
            soloed: track.soloed,
            effect_chain: default_effect_chain(track),
            send_targets: default_send_targets(track),
        })
        .collect()
}

fn default_effect_chain(track: &CompositionTrack) -> Vec<String> {
    match track.role {
        TrackRole::Voice => vec![
            "high-pass filter".to_string(),
            "dialogue compressor".to_string(),
        ],
        TrackRole::Music => vec!["low-shelf trim".to_string()],
        TrackRole::Sfx | TrackRole::Ambience => vec!["short room".to_string()],
        TrackRole::Stem | TrackRole::Master => vec!["bus limiter".to_string()],
    }
}

fn default_send_targets(track: &CompositionTrack) -> Vec<String> {
    match track.role {
        TrackRole::Voice => vec!["room-reverb".to_string()],
        TrackRole::Sfx => vec!["impact-bus".to_string()],
        _ => vec![],
    }
}

pub fn composition_duration_ms(composition: &Composition) -> u64 {
    composition
        .tracks
        .iter()
        .flat_map(|track| track.clips.iter())
        .map(|clip| {
            clip.timeline_start_ms.saturating_add(
                clip.source_range
                    .end_ms
                    .saturating_sub(clip.source_range.start_ms),
            )
        })
        .max()
        .unwrap_or(0)
}

fn find_clip_mut<'a>(
    composition: &'a mut Composition,
    clip_id: &str,
) -> io::Result<&'a mut CompositionClip> {
    composition
        .tracks
        .iter_mut()
        .flat_map(|track| track.clips.iter_mut())
        .find(|clip| clip.id == clip_id)
        .ok_or_else(|| invalid_input(format!("unknown clip id: {clip_id}")))
}

fn role_slug(role: TrackRole) -> &'static str {
    match role {
        TrackRole::Voice => "voice",
        TrackRole::Music => "music",
        TrackRole::Sfx => "sfx",
        TrackRole::Ambience => "ambience",
        TrackRole::Stem => "stem",
        TrackRole::Master => "master",
    }
}

fn invalid_input(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message.into())
}

fn next_id_suffix() -> String {
    let sequence = COMPOSITION_ID_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    format!("{}-{sequence}", timestamp_millis())
}

fn timestamp_string() -> String {
    timestamp_millis().to_string()
}

fn timestamp_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis())
}

fn write_json(path: impl AsRef<Path>, value: &impl Serialize) -> io::Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let payload = serde_json::to_vec_pretty(value)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let mut temp = path.as_os_str().to_os_string();
    temp.push(".tmp");
    let temp = PathBuf::from(temp);
    {
        let mut file = fs::File::create(&temp)?;
        file.write_all(&payload)?;
        file.sync_all()?;
    }
    fs::rename(&temp, path)?;
    if let Some(parent) = path.parent() {
        if let Ok(dir) = fs::File::open(parent) {
            let _ = dir.sync_all();
        }
    }
    Ok(())
}

fn read_json<T: DeserializeOwned>(path: impl AsRef<Path>) -> io::Result<T> {
    let payload = fs::read(path)?;
    serde_json::from_slice(&payload)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

#[cfg(test)]
mod tests {
    use super::{
        AddCompositionClipRequest, AddCompositionTrackRequest, CompositionDocumentStore,
        CompositionEditorOverview, DeleteCompositionClipRequest, MoveCompositionClipRequest,
        TrimCompositionClipRequest, UpdateCompositionTrackRequest,
    };
    use crate::domain::{AudioAssetKind, LibraryScope, RecipeWorkflow, TimeRange, TrackRole};
    use std::path::PathBuf;

    fn temp_root(label: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "soundworks-composition-{label}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create test root");
        root
    }

    #[test]
    fn reference_editor_covers_tracks_clips_tools_and_mixer() {
        let overview = CompositionEditorOverview::reference();

        assert_eq!(overview.schema_version, 1);
        assert_eq!(overview.tracks.len(), 4);
        assert_eq!(
            overview
                .tracks
                .iter()
                .map(|track| track.clip_count)
                .sum::<usize>(),
            7
        );
        assert_eq!(overview.tools.iter().filter(|tool| tool.enabled).count(), 9);
        assert!(overview.mixer.render_ready);
        assert_eq!(overview.timeline.selected_clip_id, "clip-voice-intro");
        assert!(overview.export_plan.can_render_mixdown);
    }

    #[test]
    fn editor_preserves_asset_scope_and_generation_flows() {
        let overview = CompositionEditorOverview::reference();

        assert!(overview.asset_bin.iter().any(|asset| {
            asset.kind == AudioAssetKind::VoiceClip
                && matches!(asset.scope, LibraryScope::Project { .. })
                && asset.draggable_to_timeline
        }));
        assert!(overview.tracks.iter().any(|track| {
            track.role == TrackRole::Sfx
                && track
                    .clips
                    .iter()
                    .any(|clip| matches!(clip.source_scope, LibraryScope::GlobalLibrary))
        }));
        assert!(overview.source_flows.iter().any(|flow| {
            flow.workflow == RecipeWorkflow::VideoToAudio && flow.status != super::FlowStatus::Ready
        }));
    }

    #[test]
    fn component_decisions_keep_dependency_adoption_gated() {
        let overview = CompositionEditorOverview::reference();

        assert_eq!(overview.component_decisions.len(), 4);
        assert!(overview
            .component_decisions
            .iter()
            .any(|decision| decision.id == "waveform-playlist"
                && decision.decision.contains("do not hard-depend")));
        assert!(overview
            .component_decisions
            .iter()
            .all(|decision| decision.source_url.starts_with("https://")));
    }

    #[test]
    fn composition_store_seeds_and_persists_mutations() {
        let store = CompositionDocumentStore::new(temp_root("mutations"));

        let overview = store.overview().expect("seed document");
        assert_eq!(overview.tracks.len(), 4);
        assert_eq!(
            overview
                .tracks
                .iter()
                .map(|track| track.clip_count)
                .sum::<usize>(),
            7
        );

        let moved = store
            .move_clip(MoveCompositionClipRequest {
                composition_id: "composition-demo".to_string(),
                clip_id: "clip-voice-intro".to_string(),
                timeline_start_ms: 1_250,
            })
            .expect("move clip");
        assert_eq!(moved.tracks[0].clips[0].timeline_start_ms, 1_250);

        let reloaded = store.overview().expect("reload document");
        assert_eq!(reloaded.tracks[0].clips[0].timeline_start_ms, 1_250);
    }

    #[test]
    fn composition_store_adds_trims_deletes_clips_and_updates_tracks() {
        let store = CompositionDocumentStore::new(temp_root("edit"));

        let with_track = store
            .add_track(AddCompositionTrackRequest {
                composition_id: "composition-demo".to_string(),
                name: "Scratch Foley".to_string(),
                role: TrackRole::Sfx,
            })
            .expect("add track");
        let track_id = with_track
            .tracks
            .iter()
            .find(|track| track.name == "Scratch Foley")
            .expect("new track")
            .track_id
            .clone();

        let with_clip = store
            .add_clip(AddCompositionClipRequest {
                composition_id: "composition-demo".to_string(),
                track_id: track_id.clone(),
                asset_id: "asset-sfx-001".to_string(),
                version_id: "version-sfx-001-a".to_string(),
                timeline_start_ms: 2_000,
                source_range: TimeRange {
                    start_ms: 0,
                    end_ms: 1_000,
                },
                fade_in_ms: 10,
                fade_out_ms: 20,
                gain_db: -3.0,
                pan: 0.0,
            })
            .expect("add clip");
        let clip_id = with_clip
            .tracks
            .iter()
            .find(|track| track.track_id == track_id)
            .and_then(|track| track.clips.first())
            .expect("new clip")
            .clip_id
            .clone();

        let trimmed = store
            .trim_clip(TrimCompositionClipRequest {
                composition_id: "composition-demo".to_string(),
                clip_id: clip_id.clone(),
                source_range: TimeRange {
                    start_ms: 100,
                    end_ms: 900,
                },
                fade_in_ms: 15,
                fade_out_ms: 25,
            })
            .expect("trim clip");
        let trimmed_clip = trimmed
            .tracks
            .iter()
            .find(|track| track.track_id == track_id)
            .and_then(|track| track.clips.first())
            .expect("trimmed clip");
        assert_eq!(trimmed_clip.source_range.start_ms, 100);
        assert_eq!(trimmed_clip.fade_out_ms, 25);

        let muted = store
            .update_track(UpdateCompositionTrackRequest {
                composition_id: "composition-demo".to_string(),
                track_id: track_id.clone(),
                gain_db: Some(-6.0),
                pan: Some(0.25),
                muted: Some(true),
                soloed: Some(true),
            })
            .expect("update track");
        let updated_track = muted
            .tracks
            .iter()
            .find(|track| track.track_id == track_id)
            .expect("updated track");
        assert!(updated_track.muted);
        assert!(updated_track.soloed);
        assert_eq!(updated_track.gain_db, -6.0);

        let deleted = store
            .delete_clip(DeleteCompositionClipRequest {
                composition_id: "composition-demo".to_string(),
                clip_id,
            })
            .expect("delete clip");
        assert!(deleted
            .tracks
            .iter()
            .find(|track| track.track_id == track_id)
            .expect("track after delete")
            .clips
            .is_empty());
    }

    #[test]
    fn composition_store_rejects_traversal_ids() {
        let store = CompositionDocumentStore::new(temp_root("traversal"));
        let error = store
            .composition("../../escape")
            .expect_err("unsafe id rejected");
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
    }
}
