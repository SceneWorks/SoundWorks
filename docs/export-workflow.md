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

`preset-sceneworks-video-track` writes a SceneWorks-compatible file-package handoff: rendered mixdown, optional stems, `sceneworks-handoff.json`, duration, target-video duration, start offset, sample rate, channel count, loudness/peak values, markers, sections, intended SceneWorks project and video asset IDs when known, target sidecar path, provenance sidecar path, and a `soundworks://` round-trip recipe URL.

The handoff records current SceneWorks source evidence:

- Manual SceneWorks imports accept `image/*` and `video/*`, write `.sceneworks.json` sidecars, and can carry provenance in the asset `extra` field.
- SceneWorks timelines already model `audio` tracks and `audio` timeline items.
- SceneWorks video generation can carry synchronized PCM audio internally with sample rate and channel metadata.
- Current SceneWorks persisted asset types do not include a standalone audio asset type, so direct runtime attachment needs a SceneWorks-side importer or endpoint.

Compatibility checks cover target video identity, duration fit, sample-rate/channel conversion, loudness/true-peak readiness, direct-import limitations, stale-version risk, and round-trip provenance. The first integration shape is a file package that SceneWorks can import later without SoundWorks guessing at SceneWorks project internals.
