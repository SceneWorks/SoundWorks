# Product Parity Map

This map translates SceneWorks-style product concepts into SoundWorks concepts. It is not a claim that the implementation should copy SceneWorks line-for-line; it is a guide for preserving familiar durable patterns while making audio-specific behavior first-class.

Confidence: medium. The map uses live Shortcut story coverage plus current high-level SceneWorks architectural orientation from CodeGraph. Exact SceneWorks source contracts must be re-checked before integration work.

## Concept Mapping

| SceneWorks concept | SoundWorks equivalent | Notes |
| --- | --- | --- |
| Project | Project | Local creative container for compositions, generated assets, recipes, and exports. |
| Studio/workflow | Audio studio/workflow | TTS, Voice Lab, SFX/ambience, samples/loops, song generation, video-to-audio, and composition editing. |
| Image/video asset | Audio asset | Source audio, generated take, edited clip, stem, loop, full mix, export, or preview. |
| Generation set | Generation batch/take set | A grouped set of generated audio variants with shared recipe context. |
| Recipe | Generation recipe | Provider, model, prompt/script, source refs, seed/randomness, settings, safety/provenance state. |
| Model/provider selector | Capability-based model/provider manifest | Match workflows to providers by supported audio capabilities and constraints. |
| Worker/job queue | Audio generation/export worker queue | Generation, analysis, waveform preview, export, and SceneWorks handoff jobs. |
| Reference asset | Source reference | Source voice, source audio, image/video reference, script, MIDI, style reference, or composition reference. |
| Preview/review | Waveform and take review | Playback, waveform, version comparison, trim/fade, metadata, and provenance inspection. |
| Asset library | Global audio asset library | Tags, collections, search, reusable voices, loops, samples, stems, and generated takes. |
| Export | Audio export and DAW/SceneWorks handoff | Full mix, stems, sidecars, loop/sample packs, DAW folder layout, SceneWorks video audio track package. |

## Workflow Coverage

| Shortcut story | Capability | Baseline subsystem |
| --- | --- | --- |
| `sc-6149` | Architecture baseline and product parity map | Planning and architecture |
| `sc-6250` | Rust backend, React UI, Tauri app architecture | Desktop shell, UI, core workspace |
| `sc-6150` | Audio domain model, storage, recipe contracts | Core domain, persistence |
| `sc-6151` | Capability-based provider and model manifests | Provider registry, manifest schema |
| `sc-6158` | Worker runtime, packaging, model installation | Worker runtime, installer, packaging, no-Python shipped runtime validation |
| `sc-6157` | Source-backed model evaluation harness and scorecard | Evaluation fixtures, model scorecards |
| `sc-6152` | Text-to-Speech Studio | TTS workflow, voice selection, script generation |
| `sc-6177` | Voice Lab cloning, fine-tuning, conversion | Voice profiles, consent, conversion jobs |
| `sc-6153` | SFX and ambience generation | Effects and ambience workflow |
| `sc-6154` | Instrument sample and loop generation | Samples, loops, tempo/key metadata |
| `sc-6155` | Complete song generation | Song workflow, sections, stems, full mix |
| `sc-6183` | Multimodal SFX and video-to-audio | Visual/video references, sync, semantic matching |
| `sc-6156` | Waveform review, editing, version comparison | Review/edit surface |
| `sc-6190` | Multitrack composition editor and mixer | Composition timeline, mixer, stems |
| `sc-6198` | Project/workspace model with global asset library | Project store and global library |
| `sc-6160` | Audio asset library, tagging, collections, search | Library UI and metadata |
| `sc-6161` | Export presets, stems, sidecars, DAW handoff | Export subsystem |
| `sc-6202` | Export compositions to SceneWorks video audio tracks | SceneWorks integration |
| `sc-6159` | Rights, consent, safety, provenance controls | Policy/provenance domain and UI |
| `sc-6162` | MVP validation matrix, fixtures, demo workflows | Validation and release readiness |

## Parity Principles

- Preserve replayability: every generated asset should have an inspectable recipe.
- Preserve provenance: every generated, edited, and exported asset should have traceable source and model metadata.
- Preserve project portability: local project data should be understandable without a hosted backend.
- Preserve provider flexibility: workflows should ask for capabilities, not a specific model implementation.
- Preserve queued execution: long-running model and export work should be represented as jobs.
- Extend the model for audio: voice identity, consent, waveform metadata, stems, tempo, key, duration, channels, sample rate, loudness, and sync should not be bolted on as generic tags.

## Integration Notes

- SceneWorks handoff should be implemented only after checking current SceneWorks media import/export contracts.
- A SoundWorks export package should include audio media plus sidecar metadata sufficient for SceneWorks to attach it to a video project without losing provenance.
- Story `sc-6202` should own any exact SceneWorks contract work rather than letting earlier foundation stories silently choose an integration format.
