# Worker Runtime And Packaging Contract

`sc-6158` establishes the first executable runtime boundary for SoundWorks. It does not port a specific audio model stack; it defines the contract every local, external executable, native binding, or managed API provider must satisfy before workflow stories can submit jobs.

Confidence: medium-high. The status, install, cache, progress, cancellation, packaging, and validation surfaces are now typed and tested. `docs/model-evaluation.md` tracks source-backed candidates and the platform-specific execution evidence still required before product enablement.

`sc-6467` adds the model-manager recovery layer in `docs/model-manager.md`. Runtime jobs still consume verified model state, but model download, cache inspection, failed install recovery, and per-candidate revalidation now live in that model-manager surface.

## Standing Product Rule

Shipped SoundWorks desktop builds must not depend on Python at runtime.

Allowed shipped runtime modes:

- Rust-native providers.
- Native library bindings.
- External executable providers with declared packages.
- Managed API providers.

Python remains allowed for tests, research spikes, model proof-of-concepts, and build-time tooling. A provider manifest that declares Python or Torch as a product runtime dependency is blocked by `RuntimePackagingPolicy::validate_catalog()`.

## Runtime Surface

`crates/soundworks-core/src/runtime.rs` defines:

- `RuntimeOverview` for the Tauri/UI status payload.
- `RuntimePackagingPolicy` for macOS and Windows shipped desktop rules.
- `DeviceInventory` and `DeviceReport` for available CPU/GPU/API capability.
- `ModelRuntimeState` for installed, available, and unavailable model states.
- `ModelCacheState` for package id, cache readiness, disk usage, license, and warmup state.
- `RuntimeCompatibility` for accelerator and memory preflight checks.
- `RuntimeJobSnapshot` for queued/running/failed job status, progress, cancellation, logs, retries, and actionable errors.
- `RuntimeJobAdmission` for pre-submission blocking of unsupported provider/model pairs.
- `RuntimeValidationCheck` for product-policy and device checks.

The Tauri command `get_runtime_overview` exposes the runtime payload. The React workspace renders model state, package policy, device count, validation checks, and reference job progress.

## Persistence Contract

Storage migration version 4 adds the first runtime tables:

- `runtime_model_states`
- `runtime_jobs`
- `runtime_validation_checks`

These tables intentionally store nested payloads as JSON while the runtime service boundary is still evolving. Later persistence work can index hot fields without breaking the serialized contract.

## Packaging Strategy

The initial packaging policy targets:

- macOS cache: `~/Library/Application Support/SoundWorks/models`
- Windows cache: `%APPDATA%\SoundWorks\models`
- App-managed sidecar worker process.
- Per-provider dependency isolation to prevent one model stack from contaminating another.
- Explicit runtime validation before job submission.

Model installation must report package id, expected disk size, actual disk use when known, license acceptance state, warmup state, and actionable unavailable reasons.

## Validation

Current checks:

- `RuntimeOverview::reference()` reports device inventory, cache state, progress, cancellation, and packaging policy without treating manifest-only models as installed.
- `shipped_runtime_policy_rejects_python_dependency` fails Python/Torch product dependencies.
- Runtime state distinguishes installed, available, and unavailable models.
- Job admission blocks unavailable models before execution.
- `ModelManagerOverview::reference()` covers all 28 epic candidates and refuses installed state unless expected cache files verify on disk.
- Cancellable jobs can transition to cancelled snapshots.

The full project check path runs these through `npm run check`, which includes `cargo test --workspace`.
