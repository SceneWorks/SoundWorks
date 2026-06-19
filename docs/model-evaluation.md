# Model Evaluation Harness And Scorecard

`sc-6157` establishes the first source-backed model evaluation contract for SoundWorks. The goal is to keep model selection evidence-driven: every candidate must carry source URLs, license/runtime notes, fixture coverage, lane-specific scoring axes, product eligibility, and the next runnable evidence needed before workflow implementation depends on it.

Confidence: medium-high for coverage and gating shape; medium for candidate rankings. This slice captures source metadata and repeatable test plans, but it does not claim real model quality until SoundWorks-owned smoke outputs and metrics exist.

`sc-6467` consumes this catalog through the model-manager recovery surface. Evaluation decides which candidates are worth considering; `docs/model-manager.md` decides whether their local cache, expected files, install actions, and blockers are real enough for runtime use.

## Core Module

- `crates/soundworks-core/src/evaluation.rs` defines `ModelEvaluationCatalog`, candidates, evidence sources, license/runtime assessments, fixtures, score axes, and lane recommendations.
- `ModelEvaluationCatalog::reference()` covers every candidate named in `sc-6157` and its comments.
- `AppOverview.model_evaluation` summarizes candidate count, source count, fixtures, lanes, statuses, product eligibility, and recommended spike IDs.
- The desktop shell exposes `get_model_evaluation_catalog` for the full scorecard payload.

## Product Gating

A candidate cannot be `Ready` until it has runnable SoundWorks evidence. Source metadata alone can only produce:

- `promising-spike` for candidates worth installing and measuring.
- `blocked` for license/runtime blockers.
- `unsuitable` for candidates that fail a lane-specific requirement.

Shipped product eligibility follows the `sc-6158` rule: Python is allowed for research and proof-of-concepts, but product-enabled paths must use Rust-native execution, native bindings, external executables, or managed APIs without a Python runtime dependency.

## Required Smoke Tests

Every candidate carries the same minimum smoke plan:

- Install.
- Load.
- First generation.
- Cancellation.
- Error recovery.
- Repeatability.
- License gate.
- Packaging preflight.

## Measurements

Fixtures collect latency, warmup, peak memory, output duration, sample rate, loudness, true peak, clipping, artifact frequency, and repeatability hash. Generated media should remain out of git unless a fixture is intentionally small and reviewed.

## Candidate Coverage

| Candidate | Lanes | Status | Product eligibility | Primary source |
| --- | --- | --- | --- | --- |
| Stable Audio 3 | song, SFX, ambience | promising spike | needs runtime port | https://github.com/Stability-AI/stable-audio-3 |
| ACE-Step 1.5 | song, loop | promising spike | needs runtime port | https://github.com/ace-step/ACE-Step-1.5 |
| LeVo 2 / SongGeneration 2 | song | promising spike | research only | https://github.com/tencent-ailab/songgeneration |
| YuE | song | promising spike | research only | https://github.com/multimodal-art-projection/YuE |
| DiffRhythm 2 | song | promising spike | research only | https://huggingface.co/ASLP-lab/DiffRhythm2 |
| Khala | song | promising spike | research only | https://github.com/Khala-Music-AI/Khala |
| HeartMuLa | song, loop | promising spike | research only | https://github.com/HeartMuLa/heartlib |
| Muse | song | promising spike | research only | https://github.com/yuhui1038/Muse |
| Kokoro 82M | TTS | promising spike | product candidate | https://huggingface.co/hexgrad/Kokoro-82M |
| VibeVoice | TTS | promising spike | research only | https://github.com/microsoft/VibeVoice |
| XTTS-v2 | TTS, voice clone | blocked | blocked | https://huggingface.co/coqui/XTTS-v2 |
| ChatTTS | TTS | blocked | blocked | https://github.com/2noise/ChatTTS |
| Fish Speech | TTS, voice clone | promising spike | needs runtime port | https://github.com/fishaudio/fish-speech |
| Chatterbox | TTS, voice clone, voice conversion | promising spike | needs runtime port | https://github.com/resemble-ai/chatterbox |
| Chatterbox Turbo | TTS | promising spike | needs runtime port | https://www.resemble.ai/learn/models/chatterbox |
| GPT-SoVITS | TTS, voice clone | promising spike | research only | https://github.com/RVC-Boss/GPT-SoVITS |
| F5-TTS | TTS, voice clone | blocked | blocked | https://github.com/SWivid/F5-TTS |
| CosyVoice 2 | TTS, voice clone | promising spike | research only | https://github.com/FunAudioLLM/CosyVoice |
| OpenVoice V2 | TTS, voice clone, voice conversion | promising spike | needs runtime port | https://github.com/myshell-ai/OpenVoice |
| RVC | voice conversion | promising spike | needs runtime port | https://github.com/RVC-Project/Retrieval-based-Voice-Conversion-WebUI |
| Stable Audio Open 1.0 | SFX, ambience, loop | promising spike | needs runtime port | https://huggingface.co/stabilityai/stable-audio-open-1.0 |
| AudioCraft / AudioGen | SFX | promising spike | research only | https://github.com/facebookresearch/audiocraft |
| AudioLDM | SFX, ambience | blocked | blocked | https://github.com/haoheliu/AudioLDM |
| AudioLDM 2 | SFX, ambience | promising spike | research only | https://github.com/haoheliu/AudioLDM2 |
| AudioX | SFX, video-to-audio, ambience | promising spike | research only | https://github.com/zeyuet/AudioX |
| MMAudio | video-to-audio, SFX | promising spike | research only | https://github.com/hkchengrex/MMAudio |
| ThinkSound | video-to-audio, SFX, ambience | promising spike | research only | https://github.com/FunAudioLLM/ThinkSound |
| MOSS-SoundEffect | SFX, ambience | promising spike | product candidate | https://github.com/OpenMOSS/MOSS-TTS |

## Current Recommendations

- TTS: start with Kokoro 82M because it is small, permissively licensed, and has plausible ONNX/native/Rust paths.
- Voice clone: spike Chatterbox first, but do not product-enable until no-Python packaging and watermark/provenance behavior are verified.
- Voice conversion: score RVC as speech-to-speech voice conversion, not TTS.
- SFX: start with MOSS-SoundEffect because current evidence includes an Apache-licensed upstream/MLX path.
- Song: spike ACE-Step 1.5 first, then compare against Stable Audio 3 and the larger research-only full-song models.
- Video-to-audio: use MMAudio as the benchmark, but keep it research-only until license and runtime packaging are resolved.

## Validation

Rust tests verify:

- All 28 named candidates are present.
- Every candidate has source evidence, status, license, smoke tests, and score focus.
- Product candidates do not require Python runtime.
- Blocked candidates carry explicit license or runtime blockers.
- Fixtures and scorecards cover required lanes.
- Scorecard axes sum to 100 per lane.
- Recommendations point to existing non-ready candidates.
- The catalog serializes for Tauri and storage boundaries.
