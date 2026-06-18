# Export Workflow

SoundWorks export workflow turns generated and edited assets into production-usable audio packages.

## Presets

The reference export contract includes presets for:

- podcast/dialogue exports,
- game SFX exports,
- sample packs,
- loop packs,
- song masters,
- DAW stem bundles,
- SceneWorks video audio-track handoff.

Each preset declares source kinds, asset kinds, output formats, package artifacts, sample-rate and bit-depth defaults, sidecar behavior, stem behavior, loudness targets, and whether BPM/key/loop metadata must be preserved.

## Formats And Packages

The export surface covers WAV, FLAC, MP3, and OGG. DAW-oriented exports can write zip bundles, stem folders, normalized filenames, cue markers, loop markers, lyrics text, provider-native artifacts, and metadata sidecars.

Loop and sample pack exports preserve BPM, key, and loop marker metadata so assets remain useful after leaving SoundWorks.

## Sidecars

Export sidecars carry recipe, model, source media, rights, edit-chain, disclosure, and provenance-event metadata. They are required for every reference preset.

## SceneWorks Handoff

`preset-sceneworks-video-track` writes a SceneWorks-compatible package shape: rendered mixdown, optional stems, duration, sample rate, channel count, loudness/peak values, markers, sections, intended target IDs when known, and provenance sidecar path.

The actual SceneWorks import/attachment contract remains tracked by `sc-6202`; this story establishes the SoundWorks-side export package.
