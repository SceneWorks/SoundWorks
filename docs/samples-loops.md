# Samples And Loops Studio

`sc-6154` establishes the workflow contract for reusable instrument samples, one-shots, riffs, rhythmic loops, and sample-pack style assets.

Confidence: medium-high. The workflow state, provider capability controls, sample-pack organization, BPM/key/loop metadata, evaluation-backed scorecards, QA checks, recipe provenance, and saved outputs are represented. Real provider adapters and generated audio bytes still require later model integration stories and runnable smoke outputs.

## Implementation Surface

- `crates/soundworks-core/src/samples.rs` defines the Samples + Loops Studio overview contract.
- `get_samples_studio_overview` exposes the contract through the Tauri command boundary.
- The React workspace renders a Samples + Loops panel with instrument prompt state, BPM/key/bar controls, provider options, scorecards, sample-pack variants, saved outputs, post-processing actions, and QA checks.
- `AppOverview::baseline()` marks Samples + Loops as scaffolded and includes dashboard summary metadata for routing and command discovery.

## Workflow Contract

The reference workflow represents:

- Prompt, negative prompt, instrument family, articulation, genre/style tags, and optional reference audio.
- Controls for musical key, scale, BPM, bars/beats, loopability, dry/wet ambience, velocity/energy, variation count, batch size, and project-library promotion.
- Provider options derived from `instrument-sample` and `loop` capability manifests, including tempo/key support, loop-point support, runtime/install state, commercial-use state, and available controls.
- Provider scorecards for Stable Audio 3, ACE-Step 1.5, HeartMuLa, Muse, and Stable Audio Open 1.0 with product/runtime eligibility kept explicit.
- Variants for one-shot instrument samples and loops, including transient/sample isolation, BPM, key, time signature, loop points, loudness, peak, clipping, favorite state, duplicate/version linkage, tags, and collection assignment.
- Separate `InstrumentSample` and `Loop` generation recipes so one-shots and loops remain first-class request types.
- Saved outputs as project-local `InstrumentSample` and `Loop` assets with versioned media paths, waveform/spectrogram preview caches, recipe provenance, BPM/key metadata, loop points, and metadata sidecars.

## Storage

Migration `samples_studio_workflow` adds:

- `samples_studio_prompts`
- `samples_studio_variants`
- `samples_studio_provider_scorecards`
- `samples_studio_pack_collections`
- `samples_studio_post_processing_actions`
- `samples_studio_qa_checks`
- `samples_studio_submissions`
- `samples_studio_saved_outputs`

## Validation

Local tests cover:

- Capability-driven sample and loop controls.
- Provider scorecards for sample/loop candidates without overstating runtime readiness.
- Separate sample and loop recipes with BPM/key/bar metadata.
- Saved outputs with project collection membership, instrument sample metadata, loop BPM/key metadata, and loop points.
- Tauri command exposure and `AppOverview` summary fields.

Real audio quality validation remains future model-adapter work. This slice defines the product contract and review surface that those adapters must satisfy.
