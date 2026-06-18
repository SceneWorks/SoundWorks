# TTS Studio Workflow Contract

`sc-6152` establishes the first Text-to-Speech Studio workflow contract across Rust core, Tauri, and React.

Confidence: medium-high. The workflow surface, safety gates, persistence shape, and app-visible state are represented. Real provider adapters and generated audio bytes still require later model integration stories and runnable smoke outputs.

## Implemented Surface

- `crates/soundworks-core/src/tts.rs` defines the TTS Studio overview contract.
- `get_tts_studio_overview` exposes the contract through the Tauri command boundary.
- The React workspace renders a TTS Studio panel with script segments, speaker/voice consent status, provider limitations, generation plan summary, queued submission state, and saved voice-clip output.
- `AppOverview::baseline()` marks TTS Studio as scaffolded and includes a summary for dashboard-level routing.

## Studio State

The studio overview includes:

- Script metadata, language, ordered segments, scene labels, per-segment target duration, and regenerate policy.
- Pronunciation dictionary entries scoped by language.
- Speaker assignments mapped to consented voice profiles.
- Provider options derived from the capability manifest and runtime state.
- Visible limitations for install state, license state, voice consent, and unsupported provider capabilities.
- Controls for speed, style, emotion, loudness normalization, segment timing preservation, and project-library promotion.
- Chunking and stitching plan with segment-to-chunk mapping, voice profile references, crossfade, silence trim, and loudness target.
- Submission preview with generation job, recipe, blocking reasons, warnings, and output asset IDs.
- Saved output represented as a project `VoiceClip` asset version with media, waveform, spectrogram, and provenance sidecar paths.

## Safety And Rights

Voice-clone capable TTS providers require explicit consent for every selected voice profile before submission. Provider limitations are visible before queueing generation, and the saved output carries rights metadata for provider license state, commercial use, voice consent, disclosure requirement, watermarking, and provenance.

RVC-style source-audio voice conversion remains outside this TTS workflow and belongs to `sc-6177` Voice Lab.

## Persistence Contract

Migration `tts_studio_workflow` adds:

- `tts_scripts`
- `tts_script_segments`
- `tts_speakers`
- `tts_generation_submissions`
- `tts_saved_outputs`

The studio reuses the existing `voice_profiles`, `generation_recipes`, `generation_jobs`, `audio_assets`, `audio_asset_versions`, and `storage_paths` tables.

## Validation

Rust tests verify:

- Multi-speaker script segmentation and generation chunking.
- Consent and provider limitations are visible before submission.
- Submitted output is saved as a project voice clip with recipe output linkage.
- App overview and Tauri command serialization include the TTS Studio contract.

Frontend tests verify the TTS panel renders the script, provider limitation, queued job state, and saved output.
