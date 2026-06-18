# Rights, Consent, Safety, and Provenance

SoundWorks treats rights policy as a product contract, not a final export warning. The `RightsSafetyOverview` contract records the decisions needed before generated audio can be queued or exported.

## Contract

- `RightsPolicy` defines the launch rules: consent-required voice workflows, commercial export requirements, blocked prompt categories, warning categories, watermark posture, and mandatory provenance sidecars.
- `ConsentCheck` stores allow/block decisions for clone, fine-tune, and voice-conversion paths with the `RightsMetadata` that must travel with generated outputs.
- `ModelUseDecision` evaluates source-backed model candidates for commercial export using license status, product eligibility, runtime path, Python-runtime dependency, and evaluation status.
- `ContentPolicyGate` exposes product gates for public-figure voice cloning, unauthorized references, artist/style imitation, copyrighted lyrics, disclosure, and watermark gaps.
- `ProvenanceSidecar` describes export metadata files containing recipe, model, source media, rights, disclosure, watermark, and edit-chain data.

## Current Launch Rules

- Voice cloning, voice conversion, and few-shot voice fine-tuning require explicit consent metadata before submission.
- Public-figure or celebrity voice imitation is blocked, not queued for manual cleanup.
- Noncommercial, research-only, blocked, unknown-commercial-use, or Python-runtime-only model candidates cannot be used for commercial export.
- Provider-terms models can warn instead of block only when the terms can be accepted and attached to the export record.
- Generated or AI-edited audio requires disclosure metadata in the export sidecar.
- Watermark embedding is advisory until provider support is selected; sidecar disclosure is mandatory now.

## Validation

The reference contract includes tests for:

- Blocking voice workflows without explicit consent metadata.
- Blocking noncommercial, unknown-commercial-use, research-only, and Python-runtime-only model export paths.
- Including recipe, model, source media, rights, disclosure, and edit-chain fields in provenance sidecars.
- Keeping watermark policy visible as a warning rather than silently treating it as solved.
