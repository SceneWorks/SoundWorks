# Voice Lab Workflow Contract

`sc-6177` establishes the Voice Lab workflow contract for consented voice cloning, few-shot/fine-tuning preparation, and RVC-style speech-to-speech voice conversion across Rust core, Tauri, and React.

Confidence: medium-high. The workflow state, safety gates, persistence shape, provider scorecards, and saved conversion output are represented. Real provider adapters, training jobs, and generated audio bytes still require later model integration stories and runnable smoke outputs.

## Implemented Surface

- `crates/soundworks-core/src/voice_lab.rs` defines the Voice Lab overview contract.
- `get_voice_lab_overview` exposes the contract through the Tauri command boundary.
- The React workspace renders a Voice Lab panel with mode cards, profile readiness, reference clips, conversion source/output state, provider scorecards, safety gates, and QA checks.
- `AppOverview::baseline()` marks Voice Lab as scaffolded and includes a dashboard summary for routing and command discovery.

## Lab State

The lab overview includes:

- Three distinct modes: zero-shot clone, few-shot fine-tune, and voice conversion.
- Consented and review-required voice profiles with allowed uses, source clip IDs, commercial-use state, and per-mode readiness.
- Reference clips with owner attestation, consent state, duration, profile linkage, and accepted modes.
- A voice conversion source audio asset separate from target voice profiles.
- A conversion submission preview using `RecipeWorkflow::VoiceConversion` and `VoiceConversionRecipe`.
- Saved conversion output represented as a project `VoiceClip` asset version with media, waveform, spectrogram, and provenance sidecar paths.

## Provider Scorecards

Voice Lab scorecards are derived from the source-backed model evaluation catalog:

- Chatterbox and OpenVoice V2 are represented as voice clone/conversion candidates that still need runtime ports.
- RVC is represented only as speech-to-speech voice conversion, not TTS.
- GPT-SoVITS and CosyVoice 2 remain research-only for product enablement.
- F5-TTS and XTTS-v2 remain blocked by licensing/runtime constraints.
- Chatterbox Turbo is visible as unsuitable for Voice Lab because the tracked candidate is TTS-only.

## Safety And Rights

Voice Lab gates block cloning, fine-tuning, and conversion unless explicit voice consent is recorded for the target profile. Review-required profiles remain visible for management but cannot queue clone/conversion jobs. RVC-style conversion requires source audio plus a consented target profile, and noncommercial or unknown licenses remain visible as blocked or research-only scorecards.

Saved conversion output carries rights metadata for user-owned source material, explicit target voice consent, commercial-use allowance, AI disclosure, sidecar watermarking, and provenance.

## Persistence Contract

Migration `voice_lab_workflow` adds:

- `voice_lab_profiles`
- `voice_lab_reference_clips`
- `voice_lab_provider_scorecards`
- `voice_lab_safety_gates`
- `voice_lab_qa_checks`
- `voice_lab_conversion_submissions`

The lab reuses `voice_profiles`, `model_evaluation_candidates`, `generation_recipes`, `generation_jobs`, `audio_assets`, `audio_asset_versions`, and `storage_paths`.

## Validation

Rust tests verify:

- Zero-shot, few-shot/fine-tune, and conversion modes are represented separately.
- Provider scorecards keep RVC as voice conversion, not TTS.
- Chatterbox Turbo is marked unsuitable for Voice Lab when the tracked candidate is TTS-only.
- Conversion preview uses source audio and a `VoiceConversionRecipe`.
- Safety gates and saved output capture the export contract.

Frontend tests verify the Voice Lab panel renders the conversion action, provider scorecards, RVC speech-to-speech output, safety gates, and saved conversion output.
