# SoundWorks Domain Contracts

`sc-6150` establishes the first executable Rust contracts for audio assets, generated outputs, storage paths, recipes, jobs, provenance, and timeline composition state.

Confidence: medium-high. These contracts cover the full story acceptance surface, but the exact SQLite service layer and provider-specific request details will evolve as later stories introduce real manifests, workers, and UI workflows.

## Core Modules

- `crates/soundworks-core/src/domain.rs` defines stable serializable domain types.
- `crates/soundworks-core/src/storage.rs` defines schema migration contracts and collision-resistant media/preview/sidecar paths.
- `crates/soundworks-core/src/fixtures.rs` defines canonical fixture outputs for TTS, SFX, instrument samples, loops, songs, projects, and timeline compositions.
- `crates/soundworks-core/src/workspace.rs` defines the project workspace, global library, source picker, link/copy/promote reuse actions, composition links, SceneWorks parity notes, and validation checks.
- `crates/soundworks-core/src/tts.rs` defines the Text-to-Speech Studio workflow state layered over the shared domain, recipe, provider, runtime, and storage contracts.
- `crates/soundworks-core/src/voice_lab.rs` defines the Voice Lab workflow state for consented profiles, clone/fine-tune/conversion modes, source-audio conversion, safety gates, provider scorecards, and saved voice-clip outputs.
- `crates/soundworks-core/src/sfx.rs` defines the SFX Studio workflow state for text-first sound effects and ambience generation, provider capability controls, variant comparison, loop inspection, post-processing, export state, scorecards, and saved outputs.
- `crates/soundworks-core/src/video_to_audio.rs` defines the Video to Audio workflow state for silent video, image keyframe, reference-audio, and text-conditioned Foley generation, target ranges, sync preview, provider scorecards, provenance, safety gates, and saved synchronized SFX output.
- `crates/soundworks-core/src/samples.rs` defines the Samples + Loops workflow state for instrument one-shots, loop generation, BPM/key/bar controls, sample-pack organization, provider scorecards, post-processing, QA checks, and saved sample/loop outputs.
- `crates/soundworks-core/src/songs.rs` defines the Song Studio workflow state for lyrics, song sections, style controls, vocal/instrumental generation, provider scorecards, arrangement previews, variants, stems, export targets, and saved song/music-clip outputs.
- `crates/soundworks-core/src/review.rs` defines the Waveform Review workflow state for previewable generated assets, waveform/spectrogram caches, transport controls, lightweight edits, non-destructive edited versions, version comparison, and provenance inspection.

## Represented Concepts

- Workspaces, projects, global-library scope, and project-local scope.
- Project workspace state: active and recent projects, global library state, project/global scope controls, source picker policy, global asset transfer actions, composition asset links, and SceneWorks-style parity notes.
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
- Video to Audio state: source video, keyframe references, reference audio, prompt direction, target ranges, detected events, sync points, synchronized preview segments, provider readiness scorecards, source-media safety gates, export sidecar fields, and saved synchronized SFX output.
- Samples + Loops state: instrument prompt controls, articulation, tempo/key/scale, bars/beats, one-shot and loop variants, sample-pack collections, favorites, duplicate/version handling, provider scorecards, QA checks, post-processing actions, and saved instrument sample/loop outputs.
- Song Studio state: song drafts, lyrics, style tags, language, singer hints, reference audio, ordered sections, BPM/key/time-signature/duration controls, requested stems, provider scorecards, arrangement previews, song/music-clip variants, export targets, QA checks, and saved outputs.
- Waveform Review state: all generated fixture asset kinds, waveform transport, seek/scrub/zoom/loop controls, keyboard shortcuts, cached waveform/spectrogram previews, lightweight edit action metadata, edited version saves, A/B version comparison, and recipe/provenance sidecars.

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
- `video_to_audio_sources`
- `video_to_audio_target_ranges`
- `video_to_audio_detected_events`
- `video_to_audio_provider_scorecards`
- `video_to_audio_submissions`
- `video_to_audio_sync_previews`
- `video_to_audio_saved_outputs`
- `video_to_audio_export_packages`
- `video_to_audio_safety_gates`
- `workspace_records`
- `workspace_project_cards`
- `workspace_global_libraries`
- `workspace_scope_controls`
- `workspace_source_picker_policies`
- `workspace_transfer_actions`
- `workspace_composition_asset_links`
- `workspace_parity_notes`
- `workspace_validation_checks`
- `samples_studio_prompts`
- `samples_studio_variants`
- `samples_studio_provider_scorecards`
- `samples_studio_pack_collections`
- `samples_studio_post_processing_actions`
- `samples_studio_qa_checks`
- `samples_studio_submissions`
- `samples_studio_saved_outputs`
- `song_studio_drafts`
- `song_studio_sections`
- `song_studio_variants`
- `song_studio_provider_scorecards`
- `song_studio_submissions`
- `song_studio_saved_outputs`
- `song_studio_export_targets`
- `review_workspace_assets`
- `review_preview_caches`
- `review_edit_actions`
- `review_edit_submissions`
- `review_version_comparisons`
- `review_provenance_links`

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
- Workspace serialization, project/global scope coverage, source picker policy, link/copy/promote reuse actions, composition links, and provenance preservation.
- TTS Studio serialization, consent/provider gating, generation-plan chunking, and saved voice-clip output linkage.
- Voice Lab serialization, distinct mode coverage, RVC voice-conversion routing, consent/provider gating, and saved converted voice-clip output linkage.
- SFX Studio serialization, capability-driven controls, MOSS scorecard recommendation, multimodal boundary deferral, variant comparison, recipe provenance, saved SFX/ambience output linkage, and loop-point coverage.
- Video to Audio serialization, source-video/keyframe/reference-audio coverage, target-range refinement, video-conditioned provider scorecards, sync preview events, source-media safety gates, saved synchronized SFX output, and export sidecar metadata.
- Samples + Loops serialization, capability-driven controls, sample/loop scorecard readiness, separate sample and loop recipes, sample-pack collection membership, BPM/key metadata, loop points, and QA coverage for clipping/silence/loudness/duration/loop seams.
- Song Studio serialization, capability-driven controls, complete-song scorecard readiness, lyrics/section/stem recipe preservation, saved song/music-clip output linkage, export target sidecars, and provider-gate QA coverage.
- Waveform Review serialization, generated asset preview coverage, accessible transport controls, lightweight edit coverage, non-destructive edited-version saves, A/B comparison, and inspectable recipe/provenance linkage.
