# SoundWorks Domain Contracts

`sc-6150` establishes the first executable Rust contracts for audio assets, generated outputs, storage paths, recipes, jobs, provenance, and timeline composition state.

Confidence: medium-high. These contracts cover the full story acceptance surface, but the exact SQLite service layer and provider-specific request details will evolve as later stories introduce real manifests, workers, and UI workflows.

## Core Modules

- `crates/soundworks-core/src/domain.rs` defines stable serializable domain types.
- `crates/soundworks-core/src/storage.rs` defines schema migration contracts and collision-resistant media/preview/sidecar paths.
- `crates/soundworks-core/src/fixtures.rs` defines canonical fixture outputs for TTS, SFX, instrument samples, loops, songs, projects, and timeline compositions.
- `crates/soundworks-core/src/tts.rs` defines the Text-to-Speech Studio workflow state layered over the shared domain, recipe, provider, runtime, and storage contracts.
- `crates/soundworks-core/src/voice_lab.rs` defines the Voice Lab workflow state for consented profiles, clone/fine-tune/conversion modes, source-audio conversion, safety gates, provider scorecards, and saved voice-clip outputs.
- `crates/soundworks-core/src/sfx.rs` defines the SFX Studio workflow state for text-first sound effects and ambience generation, provider capability controls, variant comparison, loop inspection, post-processing, export state, scorecards, and saved outputs.
- `crates/soundworks-core/src/samples.rs` defines the Samples + Loops workflow state for instrument one-shots, loop generation, BPM/key/bar controls, sample-pack organization, provider scorecards, post-processing, QA checks, and saved sample/loop outputs.

## Represented Concepts

- Workspaces, projects, global-library scope, and project-local scope.
- Audio assets and asset versions, including voice clips, music clips, SFX, songs, instrument samples, loops, stems, ambience, reference audio, compositions, and mixdowns.
- Technical metadata: sample rate, bit depth, channel count, duration, loudness, true peak, clipping, BPM, key, and loop points.
- Generation recipes for TTS, voice conversion, SFX, instrument samples, loops, songs, video-to-audio, edits, and composition renders.
- Model/provider descriptors with version/hash/runtime so recipes remain inspectable and replayable when the provider supports replay.
- Rights and safety metadata for license state, commercial-use eligibility, voice consent, AI disclosure, watermarking, and ownership notes.
- Source references, post-processing steps, prompt presets, voice profiles, collections, generation jobs, export presets, and provenance records.
- Multitrack composition state: tracks, clips, source trims, fades, gain, pan, mute/solo, automation, markers, sections, and export history.
- TTS Studio state: scripts, ordered speaker segments, pronunciation entries, voice-profile assignments, provider limitations, consent gates, chunking/stitching plans, submission previews, and saved voice-clip outputs.
- Voice Lab state: zero-shot clone, few-shot fine-tune, and voice conversion modes; consented voice profile readiness; reference clips; RVC-style source-audio conversion previews; provider readiness scorecards; safety gates; QA checks; and saved converted voice clips.
- SFX Studio state: prompts, negative prompts, categories, tags, reference audio, provider-derived controls, SFX and ambience variants, selected saves, loop points, scorecards, post-processing actions, export state, and saved SFX/ambience outputs.
- Samples + Loops state: instrument prompt controls, articulation, tempo/key/scale, bars/beats, one-shot and loop variants, sample-pack collections, favorites, duplicate/version handling, provider scorecards, QA checks, post-processing actions, and saved instrument sample/loop outputs.

## Storage Contract

Structured metadata is intended for SQLite. The first schema contract covers:

- `projects`
- `audio_assets`
- `audio_asset_versions`
- `generation_recipes`
- `generation_jobs`
- `voice_profiles`
- `compositions`
- `collections`
- `prompt_presets`
- `storage_paths`
- `tts_scripts`
- `tts_script_segments`
- `tts_speakers`
- `tts_generation_submissions`
- `tts_saved_outputs`
- `voice_lab_profiles`
- `voice_lab_reference_clips`
- `voice_lab_provider_scorecards`
- `voice_lab_safety_gates`
- `voice_lab_qa_checks`
- `voice_lab_conversion_submissions`
- `sfx_studio_prompts`
- `sfx_studio_variants`
- `sfx_studio_provider_scorecards`
- `sfx_studio_post_processing_actions`
- `sfx_studio_submissions`
- `sfx_studio_saved_outputs`
- `samples_studio_prompts`
- `samples_studio_variants`
- `samples_studio_provider_scorecards`
- `samples_studio_pack_collections`
- `samples_studio_post_processing_actions`
- `samples_studio_qa_checks`
- `samples_studio_submissions`
- `samples_studio_saved_outputs`

Media and derived files are stored outside SQLite. The path allocator emits version-specific paths:

```text
<root>/<scope>/<asset-kind>/<asset-id>/<version-id>/media.<ext>
<root>/<scope>/<asset-kind>/<asset-id>/<version-id>/previews/waveform.json
<root>/<scope>/<asset-kind>/<asset-id>/<version-id>/previews/spectrogram.bin
<root>/<scope>/<asset-kind>/<asset-id>/<version-id>/metadata/recipe-provenance.json
```

Only safe path segments are accepted, which keeps asset IDs and version IDs from escaping the managed storage root.

## Current Validation

Rust tests verify:

- Fixture coverage for TTS, SFX, instrument sample, loop, and song outputs.
- Project and composition timeline serialization, including tracks, clips, trims, fades, automation, sections, markers, and export history.
- Recipe serialization and replayability summaries.
- Version-specific storage paths and preview/sidecar locations.
- Rejection of unsafe storage path segments.
- Migration coverage for the required domain tables.
- TTS Studio serialization, consent/provider gating, generation-plan chunking, and saved voice-clip output linkage.
- Voice Lab serialization, distinct mode coverage, RVC voice-conversion routing, consent/provider gating, and saved converted voice-clip output linkage.
- SFX Studio serialization, capability-driven controls, MOSS scorecard recommendation, multimodal boundary deferral, variant comparison, recipe provenance, saved SFX/ambience output linkage, and loop-point coverage.
- Samples + Loops serialization, capability-driven controls, sample/loop scorecard readiness, separate sample and loop recipes, sample-pack collection membership, BPM/key metadata, loop points, and QA coverage for clipping/silence/loudness/duration/loop seams.
