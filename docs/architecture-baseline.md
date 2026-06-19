# SoundWorks Architecture Baseline

## Baseline Context

The repository now has an initial Rust/React/Tauri scaffold and first-pass Rust domain/storage contracts. This document establishes the target architecture for the first implementation wave and gives later stories concrete boundaries to execute against.

Confidence: medium-high. The architecture aligns with the live Shortcut epic and the intended Rust, React, and Tauri direction, and the first core crate contracts now exist. Real provider adapters and persistence service details still need story-level validation.

## Architectural Goals

- Preserve SceneWorks-style durable concepts where they help: projects, assets, recipes, generated outputs, workers, provider capabilities, and queued jobs.
- Treat audio concerns as native concepts: waveform metadata, clips, stems, voices, loops, tempo, key, loudness, channels, sample rate, duration, sync, and mix state.
- Keep provider/model support capability-driven so workflows can adapt to model strengths without hard-coding a single backend.
- Keep generated outputs portable through filesystem assets plus structured metadata sidecars.
- Make safety, rights, and provenance part of the core domain model, not a late UI-only layer.

## Target Runtime Shape

```text
React UI
  -> Tauri commands
    -> Rust app services
      -> soundworks-core domain/storage/job/export APIs
      -> local worker runtime
        -> provider adapters and installed models
      -> filesystem asset store and SQLite metadata store
```

## Primary Subsystems

### Desktop Shell

The desktop shell owns local app lifecycle, safe filesystem access, menus, native dialogs, packaging, and update hooks. It should expose narrow Tauri commands rather than letting UI code reach directly into persistence or model execution.

### React UI

The UI should be organized around production workflows:

- Studio surfaces for TTS, Voice Lab, SFX/ambience, samples/loops, songs, and video-to-audio.
- Library surfaces for assets, collections, tags, search, provenance, and reuse.
- Review/edit surfaces for waveform inspection, take comparison, trims, fades, and lightweight edits.
- Composition surfaces for multitrack timeline editing, mixer controls, stems, and export.

### Core Domain

The core crate should own stable types and validation:

- Project and workspace model.
- Audio asset metadata and filesystem references.
- Generation recipes and workflow-specific parameter sets.
- Provider/model manifests and capability matching.
- Source-backed model evaluation scorecards, fixtures, and product eligibility gates.
- Job model and execution state.
- Rights, consent, provenance, and safety metadata.
- Export presets, render requests, stems, and metadata sidecars.

Current implementation: `crates/soundworks-core/src/domain.rs`, `storage.rs`, `fixtures.rs`, `workspace.rs`, `manifests.rs`, `runtime.rs`, `evaluation.rs`, `tts.rs`, `voice_lab.rs`, `sfx.rs`, `video_to_audio.rs`, `samples.rs`, `songs.rs`, `composition_editor.rs`, and `review.rs` define these first-pass contracts, project/global workspace behavior, provider capability matching, runtime packaging policy, model scorecards, workflow contracts, and fixture coverage for generated audio outputs.

### Persistence

Use SQLite for structured metadata and filesystem storage for media assets.

Recommended split:

- Project database: project-local assets, compositions, recipes, jobs, and exports.
- Global library database: reusable assets, collections, tags, installed voices, provider/model registry, and user-level settings.
- Workspace records: active/recent project state, source picker policy, global asset link/copy/promote events, and composition links back to reusable global assets.
- Asset files: original sources, generated outputs, edits/renders, previews, and exports.
- Sidecars: portable metadata for exports and selected generated outputs.

### Worker Runtime

The worker runtime should own:

- Model installation and discovery.
- Device and capability reporting.
- Job claiming and execution.
- Progress, cancellation, errors, and logs.
- Provider-specific request translation.
- Output registration back into the core asset/job model.

Current implementation: `crates/soundworks-core/src/runtime.rs` defines the worker/runtime contract for packaged desktop policy, no-Python product validation, model install/cache state, hardware compatibility, job admission, progress, cancellation, logs, and actionable failures. `docs/runtime-packaging.md` captures the story-level packaging strategy.

### Provider And Model Manifests

Provider/model manifests should describe capabilities, constraints, and defaults, including:

- Workflow support: TTS, voice clone, voice conversion, SFX, ambience, loops, samples, song, stems, video-to-audio.
- Input requirements: text, source voice, source audio, MIDI, image/video, tempo, key, duration, language, speaker count.
- Output types: mono/stereo, stems, clips, loops, full mix, metadata.
- Runtime needs: device, memory, dependencies, license, model source, install size.
- Safety constraints: consent requirements, disallowed use cases, watermarking/provenance support.

Current implementation: `crates/soundworks-core/src/manifests.rs` defines the manifest schema, built-in reference catalog, full initial capability workflow surface, matching queries, defaults, install metadata, and safety constraints. `docs/provider-manifests.md` captures the story-level contract.

### Model Evaluation

Model selection should be source-backed and repeatable before workflow slices hard-code providers.

Current implementation: `crates/soundworks-core/src/evaluation.rs` defines candidate metadata, source evidence, license/runtime assessments, no-Python product eligibility, lane fixtures, score axes, smoke test plans, and first spike recommendations. `docs/model-evaluation.md` captures all 28 candidates named in the epic/story comments and their current status.

### Export

Export should be treated as a first-class job type, not a UI save action. Export jobs should produce files plus metadata sidecars, and should be able to target:

- Single audio file.
- Stems.
- Loop/sample packs.
- DAW-friendly folder layout.
- SceneWorks-compatible video audio track package.

## Core Domain Objects

- `Workspace` - user-level scope containing global library, installed models, settings, and recent projects.
- `Project` - local creative container for compositions, assets, jobs, recipes, and exports.
- `AudioAsset` - source, generated, edited, or exported audio with technical metadata.
- `SourceReference` - voice, audio, video, image, text, or external reference used as generation input.
- `GenerationRecipe` - replayable request record with provider, model, inputs, parameters, seed/randomness, and source references.
- `GenerationJob` - queued or completed execution record.
- `VoiceProfile` - managed voice identity with source links, consent, provenance, and allowed uses.
- `Composition` - multitrack timeline, clips, mixer state, automation, and export targets.
- `Clip` - timeline placement of an audio asset or generated region.
- `ExportPreset` - render format, naming, sidecar, stem, and target integration settings.
- `ProvenanceRecord` - trace of source material, model/provider, prompt, edits, and export history.
- `RuntimeOverview` - packaged runtime policy, device inventory, model availability, cache state, validation checks, and job snapshots.
- `ModelEvaluationCatalog` - source-backed candidate scorecards, fixtures, recommendation status, and product eligibility gates.

## Initial Implementation Order

1. Scaffold Rust workspace, React app, and Tauri shell.
2. Define core domain model, storage migrations, and recipe contracts.
3. Add provider/model manifest schema and capability matching.
4. Add worker runtime boundary and job execution contracts.
5. Add model evaluation harness and fixtures.
6. Implement generation studios and review surfaces in workflow slices.
7. Add library, composition, export, SceneWorks handoff, and validation slices.

## Success Criteria For This Baseline

- The repo explains the intended architecture before source code exists.
- Every live epic story can be mapped to a subsystem or implementation phase.
- Future agents can start implementation without guessing the high-level boundaries.
- Open questions are explicit rather than hidden as implied defaults.
