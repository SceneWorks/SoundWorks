# Multitrack Composition Editor

SoundWorks compositions are project artifacts that arrange generated, imported, and reusable audio assets on a timeline.

## Shipped Contract

- `crates/soundworks-core/src/composition_editor.rs` defines the reference editor overview.
- `apps/desktop/src/lib.rs` exposes `get_composition_editor_overview` through the Tauri command boundary.
- The React workspace renders a Multitrack Editor panel with tool controls, timeline lanes, clip selection, project/global asset-bin entries, generated-asset flows, mixer state, render plan, component decision evidence, and validation checks.

The editor contract preserves composition state separately from raw assets. Clips store asset IDs, version IDs, source scope, timeline offsets, trim ranges, fades, gain, pan, lane, and edit capabilities. This lets a project composition reopen even when clips reference a global asset or a generated output from another studio.

## Timeline And Mixer

The reference editor covers:

- Voice clip, Music clip, SFX, Song, Loop, Stem, and Ambience timeline placement.
- Trim, split, duplicate, delete, fade, gain, pan, snap grid, zoom, mute, solo, and render tools.
- Track-level gain, pan, mute/solo, effect chains, sends, master gain, loudness target, and true-peak ceiling.
- Markers, sections, loop playback range, playback cursor, and selected clip state.
- Mixdown, stem output, provenance sidecar, and SceneWorks package metadata.

## Storage

Migration `composition_editor_workflow` adds durable tables for:

- `composition_editor_sessions`
- `composition_editor_tracks`
- `composition_editor_clips`
- `composition_editor_mixer_state`
- `composition_editor_component_decisions`
- `composition_editor_render_plans`

## Editor Component Decision

The current product contract does not hard-depend on a browser editor package. It records a decision matrix so the first production editor can be selected after a packaged Tauri prototype proves editing, timing, rendering, persistence, and support posture.

| Candidate | Fit | Current Decision |
| --- | --- | --- |
| [waveform-playlist](https://github.com/naomiaro/waveform-playlist) | Best first prototype candidate | Spike first because it is a React, Tone.js, and Web Audio multitrack editor with waveform, clip editing, and effects coverage. |
| [wavesurfer.js](https://wavesurfer.xyz/) plus custom timeline | Renderer primitive | Keep as fallback when SoundWorks needs more ownership over timeline state and controls. |
| [wavesurfer-multitrack](https://github.com/katspaugh/wavesurfer-multitrack) | Needs support/compatibility spike | Evaluate after compatibility with current wavesurfer and commercial support posture are checked. |
| [Tone.js Transport](https://github.com/tonejs/tone.js/wiki/Transport) | Timing primitive | Use conceptually for synchronized timing, but it is not an editor UI by itself. |

## Validation

Automated coverage verifies that the editor:

- exposes four reference tracks and seven timeline clips,
- keeps global/project clip source identity,
- represents generated-asset flows from all shipped studios plus planned video-to-audio,
- keeps render/mixer state ready while preserving the SceneWorks import limitation,
- records component source links, tradeoffs, prototype evidence, and adoption decisions.

End-to-end SceneWorks attachment remains tracked by `sc-6202`.
