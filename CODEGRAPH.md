# SoundWorks Structural Overview

This repository is not yet meaningfully indexed in CodeGraph. It contains a Rust workspace, a Tauri desktop shell, a React frontend, Rust domain/storage contracts for audio assets, recipes, jobs, provenance, and compositions, provider/model manifest contracts for capability routing, and worker runtime/packaging contracts.

## Intended Architecture

SoundWorks is planned as a local-first Tauri desktop application:

- React frontend for studio workflows, asset browsing, waveform review, multitrack composition, and project navigation.
- Rust application/backend layer for commands, persistence, job orchestration, media metadata, audio graph state, export coordination, and safe filesystem access.
- Worker runtime contract for model execution state, model installation/cache visibility, capability discovery, queued generation, progress, cancellation, and device-specific execution.
- SQLite-backed project and global library metadata, with generated audio stored as filesystem assets.
- Provider and model manifests that expose capabilities instead of hard-coding workflow-to-model assumptions.

## Planned Source Layout

The exact layout should be created by implementation stories, but the baseline target is:

- `apps/desktop/` - Tauri shell and desktop packaging.
- `apps/web/` - React UI workspace.
- `apps/worker/` - future concrete worker process if provider execution moves out of the Tauri backend.
- `crates/soundworks-core/` - shared app overview, domain model, provider/model manifests, runtime contracts, schema migration contracts, storage path allocation, recipes, job model, validation fixtures, provenance, and export/composition types.
- `docs/` - requirements, architecture, parity, and planning artifacts.

## Discovery Notes

- Shortcut epic: `6148` - SoundWorks: Audio-First Sister Product Planning.
- First executable story: `sc-6149` - establish architecture baseline and product parity map.
- App scaffold story: `sc-6250` - codify Rust backend, React UI, Tauri app architecture.
- Domain contract story: `sc-6150` - define audio domain model, storage, and recipe contracts.
- Provider manifest story: `sc-6151` - build capability-based provider and model manifest system.
- Runtime story: `sc-6158` - implement worker runtime, packaging, and model installation strategy.
- CodeGraph had no SoundWorks analysis available during baseline creation.
- SceneWorks CodeGraph summary was used only as orientation for sibling-product concepts. Current SoundWorks files remain the source of truth for this repo.

## Update Rule

When implementation begins, update this file whenever the source layout or major runtime boundary changes. Keep it concise and structural so future agents can orient before broad file reads.
