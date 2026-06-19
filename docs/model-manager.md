# Model Manager Recovery

`sc-6467` adds the recovery surface between source-backed model evaluation and runtime jobs. The model manager is responsible for candidate revalidation, cache verification, install/retry actions, and honest blocked states.

Confidence: medium-high. The manager now covers every epic 6148 candidate and verifies local cache paths from disk. Actual provider execution and generated audio remain blocked until later recovery stories wire jobs and adapters.

## Guarantees

- All 28 candidates named in epic 6148 and follow-up comments are represented.
- No candidate is marked installed unless required files exist in the SoundWorks model cache.
- Product and spike candidates expose expected files, cache path, download mechanism, source URL, license notes, runtime path, and install/revalidate actions.
- Research-only, Python-only, noncommercial, or unresolved-license candidates stay blocked or research-only with visible reasons.
- Missing-cache and failed-download states are visible in Rust, Tauri, React, and tests.

## Cache Roots

- macOS: `~/Library/Application Support/SoundWorks/models`
- Override for tests or custom runs: `SOUNDWORKS_MODEL_CACHE`

Each candidate gets a cache subdirectory named by candidate ID, such as `kokoro-82m` or `moss-soundeffect`. Required files are checked from disk before install state changes.

## First Selected Lanes

- TTS: `kokoro-82m` is the first product candidate, but remains missing-cache until the ONNX model and voice files verify.
- Voice clone: `chatterbox` needs a product-safe provider package before enablement.
- Voice conversion: `rvc` remains consent-gated and needs isolated external-executable packaging.
- SFX: `moss-soundeffect` is the first SFX product candidate, but remains missing-cache until the MLX/provider files verify.
- Song: `ace-step-1-5` needs packaged runtime proof before enablement.
- Video-to-audio: `mmaudio` remains blocked for shipped product use because current evidence points to a Python research stack.

## Commands

- `get_model_manager_overview`
- `install_model_candidate`
- `revalidate_model_candidate`

Install and revalidate commands return an operation record. If the expected files are still absent, the operation fails with recovery instructions instead of fabricating success.
