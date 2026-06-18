# Asset Library

SoundWorks asset library is the reuse layer between generation studios, waveform review, collections, and the multitrack editor.

## Capability Surface

- Supports voice clips, music clips, SFX, songs, instrument samples, loops, stems, ambience, voice profiles, reference audio, compositions, mixdowns/exports, and prompt/recipe presets.
- Separates project-local assets from global reusable assets while preserving linked-global, copied-from-global, and exported-to-SceneWorks ownership state.
- Tracks user tags, generated/system tags, collections, sample packs, song folders, favorite/rejected/archived lifecycle state, and composition usage.
- Stores waveform thumbnail readiness and quick-audition metadata for previewable audio assets.
- Keeps version history, recipe summaries, and provenance sidecar links reachable from asset detail.

## Filtering

The reference filter model covers the story requirements: type, tags, duration, BPM, key, language, voice/profile, model/provider, license/commercial status, project, collection, created date, favorite/rejected/archived state, source workflow, and composition usage.

Rejected and archived assets are not hidden forever; they require explicit lifecycle filters, matching SceneWorks-style source-picker behavior.

## Persistence

Migration `asset_library_workflow` adds library indexes for item overlays, generated tags, collection metadata, collection membership, saved filters, and reuse events. Core media metadata remains in the existing `audio_assets`, `audio_asset_versions`, `collections`, and provenance tables.

## UI Contract

The desktop app exposes `get_asset_library_overview`, and the React workspace renders:

- project/global scope metrics,
- searchable filter facets,
- waveform-thumbnail asset rows with quick preview affordances,
- collection cards for sample packs and song folders,
- lifecycle actions that preserve provenance,
- selected asset detail with version history and recipe/provenance links.
