# Song Studio Workflow Contract

`sc-6155` establishes the complete-song generation workflow contract across Rust core, Tauri, and React.

Confidence: medium-high. The product contract covers the full story surface, but real model adapters still need runnable Mac/Windows smoke evidence before any provider is marked product-ready.

## Implementation Surface

- `crates/soundworks-core/src/songs.rs` defines the Song Studio overview contract.
- `apps/desktop/src/lib.rs` exposes `get_song_studio_overview` through the Tauri command boundary.
- The React workspace renders a Song Studio panel with lyrics/structure state, style controls, provider options, scorecards, variants, saved outputs, export targets, and QA checks.
- `AppOverview::baseline()` marks Song Studio as scaffolded and includes dashboard summary metadata for routing and command discovery.

## Studio State

The contract represents:

- Song drafts with title, prompt, lyrics, style tags, language, vocal mode, singer hint, reference audio, and ordered sections.
- Controls for BPM, key, time signature, duration, section length, variant count, stem generation, requested stems, reference-audio use, and project-library promotion.
- Provider options derived from `CapabilityWorkflow::Song`, including output kinds, format, sample rate, channel layout, duration limits, lyric/style/reference/stem support, commercial-use state, watermark state, and limitations.
- Provider scorecards for Stable Audio 3, ACE-Step 1.5, LeVo 2, YuE, DiffRhythm 2, Khala, HeartMuLa, and Muse.
- Arrangement previews with section start bars, bar counts, lyric presence, and lock state.
- Song variants with duration, BPM, key, vocal mode, stems, loudness, true peak, lyric-alignment score, structure-match score, tags, and selected-for-save state.
- Submission preview with a queued generation job, replayable song recipe, blocking reasons, and warnings.
- Saved song/music-clip outputs with versioned storage paths, waveform/spectrogram preview paths, sidecar paths, technical metadata, rights metadata, and provenance IDs.
- Export targets for mastered song files, stem bundles, and promotion into the multitrack editor.

## Provider Readiness

Song Studio scorecards are derived from the source-backed model evaluation catalog:

- ACE-Step 1.5 is the current recommended first song spike because it is permissively licensed and local execution is documented, but it still needs a no-Python product runtime wrapper or external executable packaging proof.
- Stable Audio 3 is a strong broad-audio candidate, but SoundWorks must enforce provider terms and validate local Mac/Windows packaging before product enablement.
- LeVo 2, YuE, DiffRhythm 2, Khala, HeartMuLa, and Muse remain research-only until license, runtime, and packaging evidence are stronger.
- SC-6472 keeps complete-song generation blocked in the shipped app because no full-song candidate currently has verified SoundWorks cache evidence plus a product-safe Rust/native/external-executable adapter. This is intentional recovery behavior, not a completed song-generation claim.

Provider-specific advanced controls stay behind capability manifests. Unsupported capabilities are represented as warnings or blocked states rather than silently appearing as available UI.

## Persistence

Storage migration `song_studio_workflow` adds tables for:

- `song_studio_drafts`
- `song_studio_sections`
- `song_studio_variants`
- `song_studio_provider_scorecards`
- `song_studio_submissions`
- `song_studio_saved_outputs`
- `song_studio_export_targets`

The shared generation recipe stores lyrics, style tags, section structure, requested stems, reference audio IDs, seed, source references, post-processing, provider descriptor, and output asset IDs.

## Validation

Rust tests verify:

- Song Studio exposes complete-song controls and provider-derived capabilities.
- Submission preserves lyrics, section structure, requested stems, recipe outputs, and queued job output versions.
- Scorecards keep research-only models behind runtime/product gates.
- Saved song outputs are versioned and export-ready.
- QA checks and export targets cover story acceptance criteria.

Frontend tests verify the Song Studio panel renders the generation action, sections, variants, Stable Audio 3 / ACE-Step / DiffRhythm scorecard visibility, saved outputs, and provider-gate warning.
