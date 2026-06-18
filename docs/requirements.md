# SoundWorks Requirements

## Source Of Truth

- Shortcut epic: `6148` - SoundWorks: Audio-First Sister Product Planning.
- Baseline story: `sc-6149` - SoundWorks: establish architecture baseline and product parity map.
- Repository baseline date: 2026-06-17.

## Product Objective

SoundWorks must provide an audio-first creative environment that lets users generate, refine, organize, combine, and export AI-assisted audio assets for standalone use and for SceneWorks video projects.

Success means a user can move from an idea or source reference to generated audio, review and edit the result, preserve the recipe and provenance, reuse the asset across projects, assemble compositions, and export in DAW- and SceneWorks-friendly forms.

## Required Capability Surface

### Studios And Workflows

- Text-to-speech studio with script segmentation, speaker labels, voice selection, pronunciation entries, generation settings, provider limitation visibility, consent-gated voice cloning, replayable recipes, generated previews, saved voice-clip assets, and version comparison.
- Voice Lab for cloning, fine-tuning, voice conversion, consent tracking, source management, and generated voice artifact review.
- SFX and ambience workflow for prompt-driven effects, environmental beds, one-shots, loops, and variations.
- Instrument sample and loop workflow for pitched samples, rhythmic loops, tempo/key-aware outputs, and reusable instrument assets.
- Complete song generation workflow for lyrics, style, arrangement, sections, stems, and full mixes.
- Multimodal SFX and video-to-audio workflow that accepts visual/video references and produces synchronized or semantically matched audio.
- Multitrack composition editor and mixer for arranging clips, stems, generated takes, fades, gain, pan, mute/solo, and timeline export.

### Asset And Project Management

- Project/workspace model that supports local projects and a global audio asset library.
- Audio asset library with tagging, collections, search, source references, generated variants, and project reuse.
- Recipe persistence for every generated result, including model, provider, seed/randomness, prompt inputs, source references, and generation settings.
- Version comparison for generated takes, edits, and exported variants.

### Export And Interop

- Export presets for common audio deliverables.
- Stem export and full-mix export.
- Metadata sidecars that preserve recipes, provenance, rights, model details, and source links.
- DAW-friendly handoff, including predictable naming and format choices.
- SceneWorks export path for using SoundWorks compositions as video audio tracks.

### Model And Runtime Platform

- Capability-based provider and model manifest system.
- Worker runtime for local model execution, installation, capability discovery, and job execution.
- Model evaluation harness and scorecard grounded in source-backed model metadata and reproducible fixtures.
- Packaging strategy for desktop distribution and local model installation.
- Shipped desktop runtime paths must not depend on Python. Python can be used for tests, research spikes, model proof-of-concepts, and build-time tooling.
- Candidate model recommendations must distinguish source metadata from runnable SoundWorks evidence.

### Safety, Rights, And Provenance

- Consent and rights controls for source voices, source audio, generated voices, and user-provided media.
- Provenance capture for prompts, source material, model/provider versions, generated outputs, edits, and exports.
- Safety controls appropriate to audio generation, voice cloning, impersonation risk, and copyrighted/source-derived content.

### Validation

- MVP validation matrix with fixtures and demo workflows.
- Automated tests for domain contracts, storage, manifests, job routing, export metadata, and safety/provenance behavior.
- Manual validation paths for audio playback, waveform review, timeline editing, export files, and SceneWorks handoff.

## Non-Functional Requirements

- Local-first: user projects and generated assets must remain usable without a hosted backend.
- Portable: generated assets and metadata sidecars should remain interpretable outside the app.
- Reproducible: generated assets should retain enough recipe and model metadata to understand or replay generation when the provider supports it.
- Extensible: new model providers and workflows should be added through manifests and typed contracts, not one-off UI/backend branching.
- Auditable: rights, consent, and provenance metadata should be visible and queryable.
- Recoverable: long-running generation jobs and export jobs should survive app restarts where practical.

## Open Questions

- Exact initial model/provider list is not selected in this slice.
- Concrete provider adapters still need to choose between Rust-native execution, native library bindings, external executables, and managed APIs.
- SceneWorks export should be validated against current SceneWorks import/media contracts before implementation.
- Voice cloning and voice conversion require explicit consent UX and policy decisions before they can be considered shippable.

Confidence: medium-high for product scope because it is derived from live Shortcut story coverage; medium for provider execution details because concrete model adapters are not implemented yet.
