use crate::domain::{
    AudioAssetKind, Composition, LibraryScope, RecipeWorkflow, TimeRange, TrackRole,
};
use crate::fixtures::{composition_fixture, fixture_set};
use serde::{Deserialize, Serialize};

pub const COMPOSITION_EDITOR_SCHEMA_VERSION: u32 = 1;

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
        let composition = composition_fixture();
        let asset_bin = timeline_asset_bin();
        let tracks = editor_tracks(&composition);
        let tools = editor_tools();
        let component_decisions = component_decisions();

        Self {
            schema_version: COMPOSITION_EDITOR_SCHEMA_VERSION,
            project_id: "project-demo".to_string(),
            timeline: TimelineEditorState {
                duration_ms: 34_000,
                zoom_percent: 125,
                snap_grid_ms: 250,
                selected_tool: "trim".to_string(),
                selected_clip_id: "clip-voice-intro".to_string(),
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
                track_states: vec![
                    MixerTrackState {
                        track_id: "track-voice".to_string(),
                        label: "Narration".to_string(),
                        gain_db: 0.0,
                        pan: 0.0,
                        muted: false,
                        soloed: false,
                        effect_chain: vec!["high-pass filter".to_string(), "dialogue compressor".to_string()],
                        send_targets: vec!["room-reverb".to_string()],
                    },
                    MixerTrackState {
                        track_id: "track-loop".to_string(),
                        label: "Loop bed".to_string(),
                        gain_db: -2.0,
                        pan: 0.0,
                        muted: false,
                        soloed: false,
                        effect_chain: vec!["low-shelf trim".to_string()],
                        send_targets: vec![],
                    },
                    MixerTrackState {
                        track_id: "track-sfx".to_string(),
                        label: "Foley hits".to_string(),
                        gain_db: -3.0,
                        pan: -0.18,
                        muted: false,
                        soloed: false,
                        effect_chain: vec!["short room".to_string()],
                        send_targets: vec!["impact-bus".to_string()],
                    },
                    MixerTrackState {
                        track_id: "track-stems".to_string(),
                        label: "Song stems".to_string(),
                        gain_db: -5.0,
                        pan: 0.12,
                        muted: false,
                        soloed: false,
                        effect_chain: vec!["bus limiter".to_string()],
                        send_targets: vec![],
                    },
                ],
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

fn editor_tracks(composition: &Composition) -> Vec<EditorTrackView> {
    let mut tracks = composition
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
                .map(|(index, clip)| EditorClipView {
                    clip_id: clip.id.clone(),
                    asset_id: clip.asset_id.clone(),
                    version_id: clip.version_id.clone(),
                    asset_name: if clip.asset_id == "asset-loop-001" {
                        "Dusty trip-hop drums".to_string()
                    } else {
                        "Narration scratch".to_string()
                    },
                    asset_kind: if clip.asset_id == "asset-loop-001" {
                        AudioAssetKind::Loop
                    } else {
                        AudioAssetKind::VoiceClip
                    },
                    source_scope: LibraryScope::Project {
                        project_id: "project-demo".to_string(),
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
                })
                .collect(),
            automation_targets: track
                .automation
                .iter()
                .map(|lane| format!("{:?}", lane.target).to_lowercase())
                .collect(),
            editable: true,
        })
        .collect::<Vec<_>>();

    tracks.push(EditorTrackView {
        track_id: "track-sfx".to_string(),
        name: "Foley hits".to_string(),
        role: TrackRole::Sfx,
        clip_count: 2,
        gain_db: -3.0,
        pan: -0.18,
        muted: false,
        soloed: false,
        clips: vec![
            extra_clip(
                "clip-hatch-hit",
                "asset-sfx-001",
                "version-sfx-001-a",
                "Metal hatch impact",
                AudioAssetKind::Sfx,
                7_000,
                0,
                1_800,
                0,
            ),
            extra_clip(
                "clip-ambience-room",
                "asset-ambience-001",
                "version-ambience-001-a",
                "Engine room bed",
                AudioAssetKind::Ambience,
                11_200,
                0,
                12_000,
                1,
            ),
        ],
        automation_targets: vec!["gain".to_string(), "pan".to_string()],
        editable: true,
    });

    tracks.push(EditorTrackView {
        track_id: "track-stems".to_string(),
        name: "Song stems".to_string(),
        role: TrackRole::Stem,
        clip_count: 3,
        gain_db: -5.0,
        pan: 0.12,
        muted: false,
        soloed: false,
        clips: vec![
            extra_clip(
                "clip-song-hook",
                "asset-song-001",
                "version-song-001-a",
                "City Lights full mix",
                AudioAssetKind::Song,
                16_000,
                0,
                18_000,
                0,
            ),
            extra_clip(
                "clip-vocal-stem",
                "asset-stem-vocal-001",
                "version-stem-vocal-001-a",
                "City Lights vocal stem",
                AudioAssetKind::Stem,
                16_000,
                0,
                18_000,
                1,
            ),
            extra_clip(
                "clip-music-reference",
                "asset-music-001",
                "version-music-001-a",
                "Reference cue bed",
                AudioAssetKind::MusicClip,
                24_000,
                0,
                9_500,
                2,
            ),
        ],
        automation_targets: vec!["gain".to_string()],
        editable: true,
    });

    tracks
}

fn extra_clip(
    clip_id: &str,
    asset_id: &str,
    version_id: &str,
    asset_name: &str,
    asset_kind: AudioAssetKind,
    timeline_start_ms: u64,
    source_start_ms: u64,
    source_end_ms: u64,
    lane: u16,
) -> EditorClipView {
    EditorClipView {
        clip_id: clip_id.to_string(),
        asset_id: asset_id.to_string(),
        version_id: version_id.to_string(),
        asset_name: asset_name.to_string(),
        asset_kind,
        source_scope: if asset_id == "asset-sfx-001" {
            LibraryScope::GlobalLibrary
        } else {
            LibraryScope::Project {
                project_id: "project-demo".to_string(),
            }
        },
        timeline_start_ms,
        source_range: TimeRange {
            start_ms: source_start_ms,
            end_ms: source_end_ms,
        },
        fade_in_ms: 60,
        fade_out_ms: 160,
        gain_db: -2.0,
        pan: 0.0,
        lane,
        can_trim: true,
        can_split: true,
        can_duplicate: true,
        can_delete: true,
    }
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

#[cfg(test)]
mod tests {
    use super::CompositionEditorOverview;
    use crate::domain::{AudioAssetKind, LibraryScope, RecipeWorkflow, TrackRole};

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
}
