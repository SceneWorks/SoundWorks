# Video to Audio Workflow Contract

`sc-6183` establishes the multimodal SFX and video-to-audio workflow contract across Rust core, Tauri, and React.

Confidence: medium-high. The workflow state, target-range editing, provider scorecards, provenance, safety gates, synchronized preview, export package, and saved output are represented. Real provider adapters and generated audio bytes still require later model integration stories and runnable smoke outputs.

## Implemented Surface

- `crates/soundworks-core/src/video_to_audio.rs` defines the Video to Audio overview contract.
- `get_video_to_audio_overview` exposes the contract through the Tauri command boundary.
- The React workspace renders a Video to Audio panel with source media, prompt direction, target ranges, sync preview, detected events, provider scorecards, output state, safety gates, and sidecar metadata.
- `AppOverview::baseline()` marks Video to Audio as scaffolded and includes a dashboard summary for routing and command discovery.

## Workflow State

The overview includes:

- Source media for silent video, optional image/keyframe references, reference audio, duration, frame rate, resolution, and ownership attestation.
- Natural-language direction, negative prompt, sync mode, requested output asset kinds, regeneration policy, and intended export target.
- Target ranges with time spans, object labels, optional regions, and requested Foley actions.
- Detected events and sync points with confidence scores for frame-aligned preview.
- A synchronized audio preview with editable segments for servo, footsteps, pressure seal, and room tone.
- A submission preview with generation job, recipe, blocking reasons, warnings, and output asset IDs.
- Saved synchronized Foley output represented as a project `Sfx` asset version with media, waveform, spectrogram, and provenance sidecar paths.
- Export package metadata for SoundWorks composition reuse and future SceneWorks video audio-track handoff.

## Provider Scorecards

Video to Audio scorecards are derived from the source-backed model evaluation catalog:

- MMAudio is the recommended public video-to-audio benchmark candidate because upstream describes video/text-conditioned synchronized audio. It remains research-only until license and no-Python product runtime paths are resolved.
- AudioX is tracked as a broad anything-to-audio benchmark with text, video, image, and audio conditioning evidence, but it is not product-ready.
- ThinkSound is tracked for object-centric refinement and natural-language video audio editing evidence, but remains research-only until packaging and license posture are resolved.
- MOSS-SoundEffect remains the first text-to-SFX candidate, but it is explicitly marked text-only for this workflow and cannot satisfy video-conditioned synchronization by itself.
- SC-6471 keeps video-to-audio product execution blocked after revalidation: MMAudio checkpoints are non-commercial, AudioX models are non-commercial/watermarked, and ThinkSound states its code/models/dataset are for research/education only with no commercial use. These candidates also remain Python/PyTorch-oriented, so none can satisfy the shipped Rust/Tauri no-Python runtime gate yet.

Primary evidence refreshed during this slice:

- AudioX official repo/project page: https://github.com/zeyuet/AudioX and https://zeyuet.github.io/AudioX/
- MMAudio official repo: https://github.com/hkchengrex/MMAudio
- ThinkSound official repo/project evidence: https://github.com/FunAudioLLM/ThinkSound and https://thinksound-project.github.io/
- MOSS-SoundEffect model card: https://github.com/OpenMOSS/MOSS-TTS/blob/main/docs/moss_sound_effect_model_card.md

## Provenance, Safety, And Export

The workflow preserves multimodal provenance for source video, image keyframes, reference audio, target ranges, detected events, sync points, prompt direction, provider/model/runtime, license gate state, and output asset IDs.

Safety gates cover:

- Source media rights and ownership attestation.
- Protected-media imitation language.
- The distinction between a queueable reference contract and real generated provider audio.

The export package includes mixdown path, sidecar path, source-media references, detected events, sync points, rights/disclosure fields, and destination targets for SoundWorks composition reuse plus future SceneWorks handoff.

## Persistence Contract

Migration `video_to_audio_workflow` adds:

- `video_to_audio_sources`
- `video_to_audio_target_ranges`
- `video_to_audio_detected_events`
- `video_to_audio_provider_scorecards`
- `video_to_audio_submissions`
- `video_to_audio_sync_previews`
- `video_to_audio_saved_outputs`
- `video_to_audio_export_packages`
- `video_to_audio_safety_gates`

The workflow reuses `model_evaluation_candidates`, `generation_recipes`, `generation_jobs`, `audio_assets`, `audio_asset_versions`, and `storage_paths`.

## Validation

Rust tests verify:

- Video, image keyframe, reference audio, and natural-language direction are represented.
- Target ranges preserve time spans, object labels, regions, and requested sounds.
- Provider scorecards distinguish video-conditioned candidates from text-only SFX candidates.
- Submission recipes preserve target ranges, sync points, source references, provider descriptors, and output asset IDs.
- Saved output and export package include synchronized audio, sidecar metadata, source media, detected events, rights, and disclosure fields.

Frontend tests verify the Video to Audio panel renders the queue action, target ranges, sync events, MMAudio/AudioX/ThinkSound/MOSS scorecards, saved output, source-rights gate, sidecar path, and real-provider limitation.
