# Waveform Review Workflow Contract

`sc-6156` establishes the audio-native review, lightweight editing, and version comparison contract across Rust core, Tauri, and React.

Confidence: medium-high. The product contract covers the full story surface for preview, edit metadata, versioning, and provenance. Actual waveform rendering/playback engines and media mutation backends still need implementation behind this stable contract.

## Implementation Surface

- `crates/soundworks-core/src/review.rs` defines the Waveform Review overview contract.
- `apps/desktop/src/lib.rs` exposes `get_review_workspace_overview` through the Tauri command boundary.
- The React workspace renders a Waveform Review panel with generated asset coverage, waveform transport, edit actions, version comparison, saved edited version state, provenance, validation checks, and shortcuts.
- `AppOverview::baseline()` marks Waveform Review as scaffolded and includes dashboard summary metadata for routing and command discovery.

## Review State

The contract represents:

- Generated audio assets across voice clips, SFX, instrument samples, loops, and songs.
- Previewable waveform and spectrogram cache state for each generated asset version.
- Transport controls for play/pause, seek, scrub, zoom, loop region, selection range, time display, keyboard shortcuts, and accessible labels.
- Lightweight edit actions for trim, fade in, fade out, normalize/loudness target, silence removal, loop crossfade, format conversion, and metadata edits.
- Non-destructive save semantics where edits append a new `AudioAssetVersion` created by `AssetCreation::Edited` rather than replacing the original generated version.
- A/B comparison between original and edited versions, including duration, loudness, true peak, and waveform difference metadata.
- Recipe and provenance inspection linking the original generation recipe, edit recipe, source version, edited version, provenance IDs, and recipe sidecar path.

## Persistence

Storage migration `review_workspace_workflow` adds tables for:

- `review_workspace_assets`
- `review_preview_caches`
- `review_edit_actions`
- `review_edit_submissions`
- `review_version_comparisons`
- `review_provenance_links`

Media remains versioned under the shared storage allocator. Edited outputs receive new media, waveform preview, spectrogram preview, and recipe-provenance sidecar paths under the edited version ID.

## Validation

Rust tests verify:

- Generated fixture coverage for voice clips, SFX, instrument samples, loops, and songs.
- Transport coverage for play/pause, seek, scrub, zoom, selection, loop region, keyboard shortcuts, and accessibility labels.
- Edit action coverage for every lightweight edit required by the story.
- Saving an edit appends a new version while preserving the original generated version.
- Version comparison and provenance stay inspectable.

Frontend tests verify the Waveform Review panel renders the selected asset, save-version action, edit actions, version comparison, edited version ID, and provenance validation message.
