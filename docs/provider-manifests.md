# Provider And Model Manifests

`sc-6151` establishes the first provider/model manifest contract for SoundWorks. The goal is capability-based routing: workflow code asks for required inputs, outputs, runtime needs, and safety constraints, then the catalog returns compatible providers and defaults.

Confidence: medium-high for the contract shape; medium for future provider details. The current registry uses built-in reference manifests; `docs/model-evaluation.md` now tracks source-backed candidates, product eligibility, and the evidence required before replacing fixture entries with real provider defaults.

## Core Module

- `crates/soundworks-core/src/manifests.rs` defines provider catalogs, model manifests, capabilities, install requirements, default parameters, safety constraints, capability queries, and match results.
- `ProviderCatalog::reference()` returns the built-in reference registry used by tests, the Tauri command boundary, and the app overview summary.
- `ProviderCatalog::find_matches()` filters by workflow, required inputs, runtime, output asset kind, sample rate, channel layout, stems, language, duration, commercial-use requirements, and runnable install state.
- `ProviderCatalog::default_for()` selects the highest-priority runnable capability for a workflow.

## Capability Surface

The manifest workflow enum intentionally covers the full epic-facing model surface, including capabilities that do not yet have dedicated generation UI:

- TTS
- Voice clone
- Voice conversion
- SFX
- Ambience
- Instrument sample
- Loop
- Song
- Stem separation
- Video-to-audio
- Edit
- Composition render

This is separate from saved recipe workflows so persisted recipes can stay stable while provider manifests express a richer model capability surface.

## Persistence Contract

Schema migration `provider_model_manifests` adds:

- `provider_manifests`
- `model_manifests`

The initial schema stores provider, install, requirement, and capability payloads as structured JSON. Later persistence services can index frequently queried fields without changing the serialized manifest contract.

## App Boundary

The desktop shell exposes:

- `get_app_overview` for the existing app summary plus provider catalog counts and workflow defaults.
- `get_provider_catalog` for the full manifest registry.

The React shell renders the provider coverage summary from `AppOverview.providerCatalog`.

## Current Validation

Rust tests verify:

- The reference catalog covers all 12 initial capability workflows.
- Capability matching filters by workflow, inputs, runtime, output kind, sample rate, channel layout, stems, language, duration, commercial-use eligibility, and runnable install state.
- Default selection uses runnable priority ordering.
- Catalog JSON is serializable for Tauri and storage boundaries.
- App overview and schema migrations include provider catalog coverage.
