# MVP Validation Matrix

SoundWorks cannot be marked MVP-ready until this matrix passes. The current reference matrix defines coverage for every capability lane, but fixture/demo data is not runtime evidence. The release gate remains blocked until verified model cache/package evidence, real provider audio, persisted runtime jobs, playback/edit/export artifacts, manual QA, stress runs, and release-hardware evidence are attached.

## Release Gate

- Status: blocked.
- Workflow coverage: 12 of 12 capability workflows have both a golden demo and a regression fixture.
- Automated checks: 4 of 10 required checks currently pass in the reference contract.
- Runtime evidence: 0 of 5 required runtime evidence checks are satisfied; all are fixture-only until follow-up recovery stories attach real artifacts.
- Manual audio QA: 0 of 9 required scorecards have real generated-audio evidence.
- Stress cases: 3 of 8 required cases pass from current contract coverage.
- Blocking limitations: manifest-only model install state, fixture-only job/output/playback/edit/export evidence, real provider audio evidence, video-to-audio prototype evidence, and Mac/Windows release hardware runs.

## Golden Demo Workflows

| Workflow | Demo | Required outcome |
| --- | --- | --- |
| TTS | Narrate short script | Voice clip, recipe, consent record, and provenance sidecar are reusable from the library. |
| Voice clone | Multi-speaker podcast segment | Speaker map and consent audit block rejected voices before export. |
| Voice conversion | Consented voice conversion | Source and target IDs stay attached to the generated voice clip. |
| SFX | Generate game UI SFX | Variants are auditionable, taggable, saveable, and accepted by the game export preset. |
| Ambience | Create loopable ambience | Loop markers, waveform preview, loudness, and loopability are inspectable. |
| Instrument sample | Generate instrument sample pack | Sample metadata and pack membership survive sample-pack export. |
| Loop | Generate musical loop pack | BPM, key, bar count, loop points, and DAW handoff metadata survive export. |
| Song | Generate complete song from lyrics and structure | Section map, lyrics, stems, and disclosure data survive song export. |
| Stem separation | Prepare stem bundle | Stem kinds, source asset links, and sidecar metadata are explicit. |
| Video to audio | Prototype silent video Foley | Source media, target ranges, sync points, and provenance are tracked until sc-6183 ships. |
| Edit | Edit, trim, and normalize | Original version remains intact and the edit chain is inspectable. |
| Composition render | Export composition with provenance | Mixdown, stems, DAW bundle, SceneWorks handoff, and provenance manifest are produced. |

## Automated Checks

The required automated categories are job contracts, recipe persistence, metadata extraction, provider manifest validation, asset lifecycle, export sidecars, safety gates, runtime evidence, documentation, and release-run artifacts. Recipe persistence, provider manifests, safety gates, and documentation pass at the current contract level. Job contracts, metadata extraction, asset lifecycle, export sidecars, runtime evidence, and release-run artifacts remain pending because fixtures do not prove product-runtime behavior.

## Runtime Evidence

The MVP gate requires real evidence for verified model cache/package files, persisted generation jobs, generated audio files, playback/edit behavior over real media, and exported audio files plus sidecars. Until those artifacts exist, static manifests, fixture paths, and reference snapshots must remain labeled as fixture/demo evidence and cannot satisfy the release gate.

## Manual QA Scorecards

Manual scorecards are required for TTS, dialogue or voice clone, voice conversion, SFX, ambience loops, sample packs, loop packs, complete songs, and video Foley. Each scorecard must attach reviewer notes and generated audio artifacts before the MVP gate can pass.

## Stress Cases

Required stress coverage includes long scripts, long songs, cancellation, failed model download, missing GPU, unsupported language, rejected voice consent, and noncommercial model use in a commercial project. Current contract coverage passes cancellation, rejected consent, and noncommercial commercial-export blocking; the rest require provider/runtime release runs.

## Epic Requirement Mapping

The matrix maps back to all eight epic requirements: TTS, SFX/ambience/Foley, samples/loops, complete songs, recipe and provenance persistence, safety/licensing, capability manifests, and audio-native review/export tools. Each requirement has at least one demo workflow, one regression fixture, and one validation check.
