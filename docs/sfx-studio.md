# SFX Studio Workflow Contract

`sc-6153` establishes the text-first SFX and ambience generation workflow contract across Rust core, Tauri, and React.

Confidence: medium-high. The workflow state, provider capability controls, persistence shape, evaluation-backed scorecards, variant comparison, loop inspection, export shape, and saved outputs are represented. Real provider adapters and generated audio bytes still require later model integration stories and runnable smoke outputs.

## Implemented Surface

- `crates/soundworks-core/src/sfx.rs` defines the SFX Studio overview contract.
- `get_sfx_studio_overview` exposes the contract through the Tauri command boundary.
- The React workspace renders an SFX + Ambience panel with prompt controls, category presets, provider capability options, provider scorecards, generated variants, loop-point state, post-processing actions, and saved outputs.
- `AppOverview::baseline()` marks SFX + Ambience as scaffolded and includes a dashboard summary for routing and command discovery.

## Studio State

The studio overview includes:

- Prompt text, negative prompt, category, tags, and optional reference audio.
- Controls for duration, variation count, intensity, realism/stylization, loopability, silence trim, loudness normalization, fades, loop crossfade, and project-library promotion.
- Category presets for Foley impacts, ambience beds, transitions, and UI sounds.
- Provider options derived from SFX and ambience capability manifests, including available controls, output kind, format, sample rate, channel layout, duration limits, reference-audio support, loop support, commercial-use state, watermark state, and limitations.
- A submission preview with generation job, recipe, blocking reasons, warnings, and output asset IDs.
- Generated variants for one-shot SFX and loopable ambience, with tags, loudness, peak, duration, selected-for-save state, and loop points where present.
- Saved outputs represented as project `Sfx` and `Ambience` asset versions with media, waveform, spectrogram, and provenance sidecar paths.

## Provider Scorecards

SFX Studio scorecards are derived from the source-backed model evaluation catalog:

- MOSS-SoundEffect remains the first recommended ML SFX spike because current evidence includes an Apache-licensed upstream/MLX path, but it is not product-enabled until a verified cache and product-safe adapter are present.
- SC-6471 adds a Rust-native procedural SFX/ambience adapter (`soundworks-native/native-procedural-sfx`) so the recovered app can generate real playable WAV artifacts immediately without Python, a model cache, or fixture-only output.
- Stable Audio 3 and Stable Audio Open remain promising candidates that need runtime and licensing gates before product enablement.
- AudioCraft / AudioGen and AudioLDM 2 remain research-only.
- AudioLDM remains blocked by noncommercial checkpoint constraints.
- AudioX, MMAudio, and ThinkSound are visible as deferred to `sc-6183` because their full workflow is multimodal/video-to-audio rather than this text-first SFX story.

## Post-Processing And Export

The workflow represents trim silence, normalize, fade/loop-crossfade, and convert/export actions. Saved outputs keep recipe provenance, generated tags, commercial-use/disclosure metadata, waveform previews, and metadata sidecar paths. Loopable ambience output includes inspectable loop points.

## Persistence Contract

Migration `sfx_studio_workflow` adds:

- `sfx_studio_prompts`
- `sfx_studio_variants`
- `sfx_studio_provider_scorecards`
- `sfx_studio_post_processing_actions`
- `sfx_studio_submissions`
- `sfx_studio_saved_outputs`

The studio reuses `model_evaluation_candidates`, `generation_recipes`, `generation_jobs`, `audio_assets`, `audio_asset_versions`, and `storage_paths`.

## Validation

Rust tests verify:

- Text-first SFX and ambience controls are derived from provider capabilities.
- MOSS-SoundEffect is selected as the recommended ML SFX scorecard, with cache/adapter validation still required before it can claim model inference.
- The native procedural adapter writes real audio previews plus output manifests with prompt, category, duration, loop, loudness, sample-rate, channel, and provenance metadata. Generated artifacts can be imported into the project library and exported through normal sidecar flows.
- Multimodal/video-to-audio candidates remain deferred to `sc-6183`.
- Submission recipes preserve prompt, reference audio, source references, post-processing, and output IDs.
- Saved outputs include SFX tags, export state, ambience output, and loop points.

Frontend tests verify the SFX panel renders the generation action, variants, MOSS recommendation, deferred multimodal boundary, and saved output state.
