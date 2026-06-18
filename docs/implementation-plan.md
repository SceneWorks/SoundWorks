# Implementation Plan

This plan maps the SoundWorks epic into executable Shortcut-backed slices. It does not replace Shortcut; the epic and stories remain the operational source of truth.

Confidence: medium-high for story coverage; medium for technical sequencing because the codebase is currently unimplemented.

## Phase 1: Foundation

### `sc-6149` - Architecture Baseline And Product Parity Map

Outcome:

- Establish repository docs, requirements, architecture baseline, parity map, and implementation order.
- Confirm the starting repo state and CodeGraph state.
- Make future implementation stories executable without re-litigating the product surface.

Success criteria:

- `README.md`, `CODEGRAPH.md`, and `docs/` baseline files exist.
- Every live story in epic 6148 maps to at least one subsystem or phase.
- Open questions and confidence levels are explicit.

### `sc-6250` - Rust Backend, React UI, Tauri App Architecture

Outcome:

- Scaffold the app shell and workspace structure.
- Add baseline dev commands, formatting, linting, and test wiring.
- Define UI/backend command boundaries.

Success criteria:

- A developer can install dependencies, run the app, run tests, and understand source layout.
- The app shell loads a real first screen rather than a placeholder marketing page.

### `sc-6150` - Audio Domain Model, Storage, And Recipe Contracts

Outcome:

- Implement core Rust domain types and storage migration contracts for projects, assets, versions, recipes, jobs, source references, provenance, rights/safety, compositions, and exports.

Success criteria:

- Domain validation and storage tests cover asset registration, versioned storage paths, recipe persistence/serialization, generated-output metadata, and fixture coverage for major audio asset types.

### `sc-6151` - Provider And Model Manifest System

Outcome:

- Implement manifest schema, capability matching, defaults, and model/provider registry.

Success criteria:

- Workflows can ask for capabilities such as TTS, voice conversion, SFX, loops, song, stems, or video-to-audio and receive compatible providers/models.

### `sc-6158` - Worker Runtime, Packaging, And Model Installation

Outcome:

- Implement worker boundary, job execution contracts, model installation/discovery, and packaging strategy.
- Enforce the shipped-product rule that desktop runtime paths do not depend on Python.

Success criteria:

- The app can discover installed model capabilities, queue jobs, report progress, cancel jobs, and register outputs.
- Model install/cache state, device compatibility, validation checks, and actionable runtime errors are visible through the app boundary.

### `sc-6157` - Model Evaluation Harness And Scorecard

Outcome:

- Create source-backed model metadata, fixtures, and evaluation scorecards.
- Cover every named candidate with status, evidence, license/runtime notes, product eligibility, and first spike recommendations.

Success criteria:

- Candidate models are compared through repeatable fixtures and documented constraints rather than preference-only selection.
- No model is marked ready without runnable SoundWorks evidence.

## Phase 2: Core Generation Workflows

- `sc-6152` - Text-to-Speech Studio: script segmentation, speaker/voice assignments, pronunciation entries, provider limitation visibility, consent-gated voice-clone capable submission, chunking/stitching plan, queued generation preview, and saved project voice-clip output.
- `sc-6177` - Voice Lab for cloning, fine-tuning, and conversion: consented profile management, distinct zero-shot/few-shot/conversion modes, RVC-style source-audio conversion, safety gates, provider scorecards, queued conversion preview, and saved project voice-clip output.
- `sc-6153` - SFX and ambience generation.
- `sc-6154` - Instrument sample and loop generation.
- `sc-6155` - Complete song generation.
- `sc-6183` - Multimodal SFX and video-to-audio generation.

Success criteria:

- Each workflow produces persisted assets, recipes, jobs, provenance, playback/review metadata, and safety/rights metadata as applicable.
- Each workflow uses provider capabilities instead of hard-coded model assumptions.

## Phase 3: Review, Library, And Composition

- `sc-6156` - Waveform review, lightweight editing, and version comparison.
- `sc-6198` - Project/workspace model with global asset library.
- `sc-6160` - Audio asset library, tagging, collections, and search.
- `sc-6190` - Multitrack composition editor and mixer.

Success criteria:

- Users can organize generated/source audio, compare versions, make lightweight edits, arrange clips, mix tracks, and reuse assets across projects.

## Phase 4: Export, Integration, And Governance

- `sc-6161` - Export presets, stems, metadata sidecars, and DAW-friendly handoff.
- `sc-6202` - Export compositions to SceneWorks video audio tracks.
- `sc-6159` - Rights, consent, safety, and provenance controls.
- `sc-6162` - MVP validation matrix, fixtures, and demo workflows.

Success criteria:

- Exports include correct media files and metadata sidecars.
- SceneWorks handoff is validated against current SceneWorks contracts.
- Rights, consent, safety, and provenance are visible and enforced at the right workflow points.
- MVP readiness is measured through automated tests, fixtures, and demo workflows.

## Cross-Cutting Rules

- Do not silently narrow workflow scope. If a model or runtime cannot support a story's full capability surface, document the blocker on the story and create or update tracked follow-up work.
- Do not treat provider/model choice as done without source-backed evaluation.
- Do not mark a story done until the repo artifacts and Shortcut story both reflect validation results.
- Keep generated media and local model artifacts out of git unless a fixture is intentionally small and reviewed.
- Keep shipped runtime paths Python-free; Python-only providers must stay research-only, API-only, or blocked before product enablement.
