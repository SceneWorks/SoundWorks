# Model Manager Recovery

`sc-6467` adds the recovery surface between source-backed model evaluation and runtime jobs. The model manager is responsible for candidate revalidation, cache verification, install/retry actions, and honest blocked states.

Confidence: high for cache revalidation and model-manager state; medium for full provider readiness. The manager now covers every epic 6148 candidate, verifies Kokoro from the downloaded Hugging Face snapshot cache, and verifies SoundWorks cache paths from disk. `sc-6468` consumes this verified state for durable runtime jobs and a native smoke adapter; full provider inference and workflow-quality audio remain owned by the end-to-end lane stories.

## Guarantees

- All 28 candidates named in epic 6148 and follow-up comments are represented.
- No candidate is marked installed unless required files exist in the SoundWorks cache or an equivalent provider snapshot cache that the app can inspect.
- Product and spike candidates expose expected files, cache path, download mechanism, source URL, license notes, runtime path, and install/revalidate actions.
- Research-only, Python-only, noncommercial, or unresolved-license candidates stay blocked or research-only with visible reasons.
- Missing-cache and failed-download states are visible in Rust, Tauri, React, and tests.

## Cache Roots

- macOS: `~/Library/Application Support/SoundWorks/models`
- Override for tests or custom runs: `SOUNDWORKS_MODEL_CACHE`
- Hugging Face snapshots: `HF_HOME/hub` or `~/.cache/huggingface/hub`

Each candidate gets a cache subdirectory named by candidate ID, such as `kokoro-82m` or `moss-soundeffect`. For Hugging Face candidates, the manager also checks matching downloaded snapshot directories, such as `models--onnx-community--Kokoro-82M-v1.0-ONNX/snapshots/<revision>`. Required files are checked from disk before install state changes.

## First Selected Lanes

- TTS: `kokoro-82m` is the first product candidate and verifies when `config.json`, `onnx/model.onnx`, and `voices/af_heart.bin` exist in the downloaded Kokoro snapshot.
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
