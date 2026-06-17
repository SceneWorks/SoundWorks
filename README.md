# SoundWorks

SoundWorks is the audio-first sister product to SceneWorks. It is planned as a local-first creative desktop app for generating, organizing, editing, and exporting AI-assisted audio assets and compositions.

Current status: architecture baseline, initial Rust/React/Tauri scaffold, Rust audio domain/storage/recipe contracts, provider manifests, and worker runtime/packaging contracts. The implementation source of truth is Shortcut epic [6148](https://app.shortcut.com/trefry/epic/6148), with the initial baseline captured in this repository.

## Product Scope

SoundWorks covers the full audio creation surface tracked in the epic:

- Text-to-speech studio
- Voice lab for cloning, fine-tuning, and voice conversion
- Sound effect and ambience generation
- Instrument sample and loop generation
- Complete song generation
- Multimodal video-to-audio generation
- Waveform review, lightweight editing, and version comparison
- Multitrack composition editing and mixing
- Project/workspace management with a global audio asset library
- Export presets, stems, metadata sidecars, DAW handoff, and SceneWorks video-audio export
- Rights, consent, safety, provenance, validation, and model evaluation

## Repository Map

- `CODEGRAPH.md` - structural overview for future CodeGraph indexing and agent orientation.
- `docs/requirements.md` - implementation-ready product and platform requirements.
- `docs/architecture-baseline.md` - target app architecture, subsystem boundaries, data flow, and confidence notes.
- `docs/domain-contracts.md` - Rust domain, storage path, recipe, and fixture contract reference.
- `docs/provider-manifests.md` - provider/model manifest schema, capability matching, defaults, and app boundary.
- `docs/runtime-packaging.md` - worker runtime, model install/cache, job progress/cancellation, and no-Python shipped runtime contract.
- `docs/product-parity-map.md` - SceneWorks-to-SoundWorks parity map and story coverage.
- `docs/implementation-plan.md` - Shortcut-backed implementation order and success criteria.
- `apps/web/` - React/Vite UI workspace.
- `apps/desktop/` - Tauri desktop shell and Rust command boundary.
- `crates/soundworks-core/` - shared Rust contracts for the app shell, domain model, storage schema, provider manifests, worker runtime state, recipe fixtures, jobs, provenance, and composition state.

## Development

Install dependencies:

```sh
npm install
```

Run the web app:

```sh
npm run dev
```

Run checks:

```sh
npm run check
```

Run the Tauri desktop shell:

```sh
npm run tauri:dev
```

## Current Assumptions

- The target app architecture is Rust backend plus React UI in a Tauri desktop shell.
- SoundWorks should mirror SceneWorks' durable concepts where useful: projects, assets, recipes, provider capabilities, job orchestration, and generated outputs.
- Audio-specific decisions must be first-class rather than thin renames of image concepts.
- Shipped desktop builds must not depend on Python at runtime; Python is allowed for tests, research spikes, model proof-of-concepts, and build-time tooling.

Confidence: medium. The Shortcut epic and story set are live and concrete, but the repo is not implemented yet and longer story descriptions were not available through the current connector.
