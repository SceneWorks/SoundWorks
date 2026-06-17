# SoundWorks Structural Overview

This repository is not yet indexed in CodeGraph. As of the first baseline pass, the repo contains planning artifacts only and no application source.

## Intended Architecture

SoundWorks is planned as a local-first Tauri desktop application:

- React frontend for studio workflows, asset browsing, waveform review, multitrack composition, and project navigation.
- Rust application/backend layer for commands, persistence, job orchestration, media metadata, audio graph state, export coordination, and safe filesystem access.
- Worker runtime for model execution, model installation, capability discovery, queued generation, and device-specific execution.
- SQLite-backed project and global library metadata, with generated audio stored as filesystem assets.
- Provider and model manifests that expose capabilities instead of hard-coding workflow-to-model assumptions.

## Planned Source Layout

The exact layout should be created by implementation stories, but the baseline target is:

- `apps/desktop/` - Tauri shell and desktop packaging.
- `apps/web/` - React UI workspace.
- `apps/worker/` - local worker runtime and model execution bridge.
- `crates/soundworks-core/` - shared domain model, storage contracts, recipes, job model, validation, and export types.
- `docs/` - requirements, architecture, parity, and planning artifacts.

## Discovery Notes

- Shortcut epic: `6148` - SoundWorks: Audio-First Sister Product Planning.
- First executable story: `sc-6149` - establish architecture baseline and product parity map.
- CodeGraph had no SoundWorks analysis available during baseline creation.
- SceneWorks CodeGraph summary was used only as orientation for sibling-product concepts. Current SoundWorks files remain the source of truth for this repo.

## Update Rule

When implementation begins, update this file whenever the source layout or major runtime boundary changes. Keep it concise and structural so future agents can orient before broad file reads.
