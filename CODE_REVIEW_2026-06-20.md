# Full Codebase Review — SoundWorks — 2026-06-20

## Executive summary

- **Repository at a glance:** Local-first audio-generation desktop app. Rust backend (`crates/soundworks-core`, ~24k LOC, 18 modules) + Tauri 2 desktop shell (`apps/desktop`) + React/Vite/TypeScript UI (`apps/web`, ~19k LOC). ~30 source files, ~43k LOC reviewed. Built autonomously by Codex against Shortcut epic [6148](https://app.shortcut.com/trefry/epic/6148).
- **Coverage:** All Rust modules read in full (runtime, persistence, six generation studios, manifests, model-manager, evaluation, MVP-validation, rights, review, export, composition, fixtures, lib). All web entry points read (`App.tsx`, `tauri.ts`, `types.ts`, `styles.css`, tests, config); `appData.ts` (6,871 lines) characterized but not line-by-line. Excluded: `Cargo.lock`, `package-lock.json`, `node_modules`, `target`, icons. The two highest-severity security findings were verified by direct read, not just delegated.
- **Headline:** This is **no longer the "static contract/demo shell" the 2026-06-19 recovery audit described** — the recovery commits (SC-6468…SC-6482) built a genuine thin spine: real WAV/PCM synthesis, real Kokoro ONNX text-to-speech, durable on-disk persistence, and a working SceneWorks-style navigation shell. But that real spine is wrapped in a very large volume of fixture/contract scaffolding: six near-identical display-only "studio" modules, a 6,871-line TypeScript clone of the Rust fixtures, a capability-manifest catalog that is disconnected from what actually runs, and a rights/safety surface that is displayed but not enforced. The single most serious issue is a **Critical path-traversal / arbitrary-file-write vulnerability** in the real persistence write paths: user-controlled `item_id`/`project_id` are joined straight into filesystem paths while the one sanitizer that exists guards only fixtures. The codebase is also honest about its own immaturity in most places (`mvp_validation.ready_for_mvp = false`, runtime evidence `Pending`), with two notable exceptions where release gates are hardcoded `Passed`.
- **Counts:** Critical: 1 | High: 8 | Medium: 14 | Low: 11 | Info: 3.

---

## Critical findings

#### [F-001] Path traversal → arbitrary file write/read via unsanitized `item_id`/`project_id` in real persistence paths
- **Category:** security
- **Severity:** Critical
- **Location:** `crates/soundworks-core/src/project_library.rs:1089-1094` (`asset_record_path`), `1096-1111` (`asset_version_root`), `520-595` (`mutate_library_item`), `640-700` (`save_review_edit`); reached from `apps/desktop/src/lib.rs:222-236`
- **Finding:** Every real write path joins caller-supplied identifiers directly into filesystem paths with no validation. `asset_record_path(item_id)` does `self.root.join("assets").join(item_id)` on the raw `item_id` that comes straight from the Tauri requests (`mutate_library_item`, `save_review_edit`, `export_library_item` all take `request.item_id`; `save_review_edit` also derives a `version_id` from it). A sanitizer **does** exist — `clean_path_segment` rejects `..`, empty, and non-`[A-Za-z0-9._-]` segments (`storage.rs:1246-1264`) — but it is only reachable via `StoragePathAllocator`, which is called exclusively by fixture builders in `asset_library.rs`, never by the real store. I verified both the unsanitized join and the unused sanitizer directly.
- **Impact:** A malicious or buggy front-end (or any IPC caller — Tauri renderers, deep links) can send `item_id = "../../../../Users/michael/.zshrc"` (or similar) and cause SoundWorks to read, write, or overwrite files outside the library root with the user's privileges. `mutate_library_item` will even *fabricate* a record for a nonexistent `item_id` (`:539`) and then write to it, turning a traversal id into an arbitrary-write primitive.
- **Suggested fix:** Run every externally supplied `item_id`/`project_id`/`version_id` through `clean_path_segment` (or route all real writes through `StoragePathAllocator`) before any `Path::join`, centralized in `asset_record_path`/`asset_version_root`. Reject on failure; canonicalize the result and assert it remains under `self.root`.
- **Confidence:** High

---

## High findings

#### [F-002] `mutate_library_item` writes to a path string read out of a JSON record (confused-deputy / write amplifier)
- **Category:** security
- **Severity:** High
- **Location:** `crates/soundworks-core/src/project_library.rs:544-555`, `588`
- **Finding:** `mutate_library_item` writes via `write_json(&record.metadata_sidecar_path, &record)` (`:588`), where `metadata_sidecar_path` is a string either computed from the raw `item_id` (`:544`) or, for an existing record, read verbatim from `asset-record.json` on disk. The stored absolute path is trusted for the write with no re-validation that it still lives under `self.root`.
- **Impact:** Amplifies F-001: once any crafted path lands in a record, every subsequent favorite/tag/archive/reject re-triggers a write to that attacker-influenced location. Also leaks absolute home-directory paths into persisted records and API payloads.
- **Suggested fix:** Never trust persisted absolute paths for writes; always recompute the target from the validated `item_id` under `self.root` and assert containment before writing.
- **Confidence:** High

#### [F-003] Voice-consent gate trusts a caller-supplied boolean; rights/safety policy is never enforced in generation
- **Category:** security
- **Severity:** High
- **Location:** `crates/soundworks-core/src/runtime.rs:1911-1930` (`validate_request_gates`); `crates/soundworks-core/src/rights.rs:46-55, 224-365`
- **Finding:** The only safety check in any real generation path gates `VoiceClone`/`VoiceConversion` on `request.parameters["voiceConsentRecorded"] == true` — a value the caller sets freely (verified). It never consults the voice profile's stored consent, the `Blocked` public-figure/celebrity gates, license/commercial-use eligibility, or anything in `rights.rs`. `rights.rs::can_export()` has zero non-test callers. SFX, samples, song, and video workflows have **no** safety gate at all.
- **Impact:** Epic requirement #6 ("voice cloning, celebrity/style imitation, copyrighted-music similarity, and disclosure as first-class safety requirements") is met only as displayed policy fixtures. Any client sending `voiceConsentRecorded: true` bypasses consent entirely; prohibited-identity cloning is unreachable from enforcement.
- **Suggested fix:** Resolve consent from the persisted voice profile (not a request flag), enforce the `Blocked` content-policy gates and `ProductEligibility`/commercial-use before job admission for all relevant workflows, and wire `can_export()` into the export execution path.
- **Confidence:** High

#### [F-004] Path traversal in the runtime job store via attacker-controlled `job_id`
- **Category:** security
- **Severity:** High
- **Location:** `crates/soundworks-core/src/runtime.rs` `read_job` (~L1116), `cancel`/`retry`/`artifacts` (~L1053-1099); reached from `apps/desktop/src/lib.rs:96-108`
- **Finding:** `read_job` builds `self.root.join("jobs").join(job_id).join("job.json")` from an unvalidated `job_id` supplied by the front end via `cancel_runtime_job`/`retry_runtime_job`/`get_runtime_job_artifacts`. `retry` then reads the persisted `recipe_path` from that record and re-deserializes it, so a crafted record can point reads anywhere on disk.
- **Impact:** Arbitrary read of JSON-parseable files (path disclosure) and DoS (reading huge/device files) outside the runtime root, from a single IPC string. Same class as F-001 in a second subsystem.
- **Suggested fix:** Validate `job_id` against a strict allowlist (`^job-[a-z-]+-\d+$`) and reject `/`, `\`, `..`; canonicalize and assert containment under `self.root/jobs`.
- **Confidence:** High

#### [F-005] No concurrency control; non-atomic read-modify-rewrite of shared `workspace.json`
- **Category:** bad-pattern
- **Severity:** High
- **Location:** `apps/desktop/src/lib.rs:176-236` (per-call `ProjectLibraryStore::default()`); `crates/soundworks-core/src/project_library.rs:250-286`, `1048-1059`, `write_json` (~L1297)
- **Finding:** Every Tauri command constructs a fresh store (holding only a `PathBuf`) with no shared lock, no Tauri managed `State`, and no file locking. `create_project`/`open_project`/`add_asset_to_project` do load-mutate-rewrite of `workspace.json` and full-directory scans. Tauri commands can run concurrently, and `write_json` writes in place (not temp+rename).
- **Impact:** Concurrent mutations interleave into a classic lost-update/TOCTOU race on `workspace.json` (a project can vanish from the workspace, `active_project_id` clobbered); a crash mid-write truncates the file → durable corruption / data loss.
- **Suggested fix:** Manage one store (or a per-root `Mutex`) via Tauri `State` and serialize mutations; make `write_json` atomic (write temp, `fsync`, `rename`).
- **Confidence:** High

#### [F-006] Generation runs synchronously on the command thread; the "queue / worker / cancellable" model is fiction
- **Category:** bad-pattern
- **Severity:** High
- **Location:** `crates/soundworks-core/src/runtime.rs` `enqueue` (~L973-1050), `run_adapter` (~L1125); Kokoro builds a per-job Tokio runtime (~L1311-1322); `apps/desktop/src/lib.rs:255-264`
- **Finding:** Despite vocabulary like "persisted in the local runtime queue", "worker boundary", and `CancellationState::Cancellable`, there is no queue and no worker. `enqueue` synchronously generates all PCM and writes the WAV before returning; the job is already `Succeeded` by the time the command returns. The only observable cancellation is a test-only `hold-for-cancel` magic-prompt branch.
- **Impact:** `enqueue_runtime_job` blocks the invoked thread for the full synthesis duration (SFX clamps to 30s, music to 120s) → the UI/command handler freezes. Cancellation shown to the user is never actionable for real jobs.
- **Suggested fix:** Move synthesis to a background task (`tauri::async_runtime::spawn` / thread pool); return the `Queued` snapshot immediately and drive status from the worker; share one Tokio runtime instead of building one per Kokoro job.
- **Confidence:** High

#### [F-007] Capability-manifest architecture is decorative — runtime selects/executes models by hardcoded id strings off a different catalog
- **Category:** bad-pattern
- **Severity:** High
- **Location:** `crates/soundworks-core/src/runtime.rs:1158-1167` (`run_adapter`), `1881-1909` (`adapter_for_model`), `83-98` (native injection); `manifests.rs:410-703` vs `evaluation.rs:320-838`; `apps/desktop/src/lib.rs:255-263`
- **Finding:** The shipped `enqueue_job` path builds its overview from the **evaluation catalog** (28 real candidates), while `manifests.rs::ProviderCatalog` (4 fictional `reference-*` models) is consumed only by the manifest-based `RuntimeOverview::reference()`, whose models always resolve to `Unavailable` and can never run. Actual execution dispatch is `match` on literal `model_id == "kokoro-82m" | "native-procedural-sfx" | "native-procedural-music" | "native-smoke"`, duplicated across `adapter_for_model` and `run_adapter`. Capability defaults/limits/safety from `find_matches` are never threaded into execution.
- **Impact:** Directly violates the epic's #1/#7 architecture requirement ("capability-based provider manifests, not one-off model assumptions"). Adding a real model means editing a Rust `match` arm in 3 places, not registering a manifest. Two parallel model taxonomies must be hand-synced.
- **Suggested fix:** Collapse to one catalog that drives the runtime; add an `execution_strategy`/adapter field to the manifest/capability and dispatch on data, not hardcoded ids. Register kokoro + native adapters in the same catalog the runtime reads.
- **Confidence:** High

#### [F-008] Web-mode experience is silent mock data; `catch {}` also swallows genuine runtime errors
- **Category:** bad-pattern
- **Severity:** High
- **Location:** `apps/web/src/tauri.ts:52-453`; `apps/web/src/App.tsx:401-506`
- **Finding:** Every backend call is `try { invoke } catch { return fallback }`. In a browser (`npm run dev`), `invoke` always rejects, so all 17 read overviews silently resolve to `appData` fixtures and the UI renders a fully populated, interactive-looking app with no indication anything is simulated. The bare `catch {}` also masks real errors in desktop mode (a genuine command failure is indistinguishable from "not in Tauri").
- **Impact:** High "looks done" illusion — the most common dev/preview path is 100% fake with no signal. In desktop mode, real failures are silently replaced with stale fixtures, hiding bugs.
- **Suggested fix:** Detect Tauri once (`'__TAURI_INTERNALS__' in window`) and show a persistent "Web preview — data simulated" banner; only fall back on a genuine "command unavailable" condition, and surface real errors to the user.
- **Confidence:** High

#### [F-009] Real persisted assets are always merged on top of a hardcoded fixture catalog
- **Category:** bad-pattern
- **Severity:** High
- **Location:** `crates/soundworks-core/src/asset_library.rs:30-153` (`reference` / `reference_with_persisted_items`); `project_library.rs:216-228`
- **Finding:** `asset_library_overview` builds the entire fabricated demo library via `reference()` (≈10 synthetic assets whose audio paths point at nonexistent files) and then *appends* real persisted items. `selected_item` after a real mutation can fall back to a fixture (`or_else(|| overview.items.last())`).
- **Impact:** Users see and can select assets that do not exist on disk (`playback_for_item` returns `playable:false` for them); counts/filters/scopes are inflated by fakes; the boundary between real and demo data is blurred in production.
- **Suggested fix:** Gate the fixture catalog behind a dev/demo flag; in production return only persisted records.
- **Confidence:** High

---

## Medium findings

#### [F-010] `App()` is a single ~4,540-line god component holding all 16 screens
- **Category:** readability
- **Severity:** Medium
- **Location:** `apps/web/src/App.tsx:349-4543` (state `350-399`; render `858-4542`)
- **Finding:** One function defines ~25 `useState`, ~15 `useMemo`, ~12 handler closures, and the JSX for every screen inline as `{showX ? (…) : null}`. Navigation routing now genuinely works (`activeView`/`setActiveView`, grouped `navSections`), but the *code* is not decomposed. Every `setState` re-renders the entire tree and recreates all closures.
- **Impact:** Very low maintainability/testability; unreviewable diffs; poor render performance compounded by F-011.
- **Suggested fix:** Extract one component per `ActiveView`; render `<ActiveScreen/>` from a small router map; lift each screen's data load into that screen.
- **Confidence:** High

#### [F-011] All 16 overview loaders fire on mount regardless of active view
- **Category:** efficiency
- **Severity:** Medium
- **Location:** `apps/web/src/App.tsx:401-506`
- **Finding:** The mount `useEffect` invokes all 16 `load*Overview` calls unconditionally — 16 IPC round-trips in desktop mode and up to 16 separate re-renders of the monolith during startup — even though the user lands on one screen.
- **Impact:** Unnecessary startup IPC and render churn; couples first paint to data the user hasn't navigated to.
- **Suggested fix:** Load active-view data lazily on first navigation; keep only `app`/`workspace`/`runtime` summaries eager for the header/queue chip.
- **Confidence:** High

#### [F-012] `appData.ts` (6,871 lines) is a hand-maintained TypeScript clone of the Rust `reference()` fixtures
- **Category:** redundant
- **Severity:** Medium
- **Location:** `apps/web/src/appData.ts:1-6871`; `crates/soundworks-core/src/*.rs` `reference()` fns
- **Finding:** The fallback module reproduces the same demo content the Rust crate already defines (matching literals like `"Demo SoundWorks Project"`, model catalogs, validation checks) in a second language, with no codegen linking them. It is imported by both `App.tsx` and `tauri.ts`, so it ships in every production bundle.
- **Impact:** Two sources of truth for the same fixtures → guaranteed drift; the TS copy can silently misrepresent backend shape. ~6.9k lines of maintenance surface.
- **Suggested fix:** Generate `appData.ts` from the Rust fixtures (serialize `reference()` to JSON at build time), or reduce it to a single "preview mode" notice.
- **Confidence:** High

#### [F-013] Six near-identical generation-studio type hierarchies (~1,500–2,000 duplicated LOC)
- **Category:** redundant
- **Severity:** Medium
- **Location:** `crates/soundworks-core/src/tts.rs`, `sfx.rs`, `samples.rs`, `songs.rs`, `voice_lab.rs`, `video_to_audio.rs` (full files)
- **Finding:** All six follow one template: a `*Overview` struct, `reference()`/`from_catalog()` builder, and a per-studio `*ProviderOption`/`*ProviderSelection`/`*ProviderScorecard`/`SubmissionPreview`/`*SavedOutput` family with near-identical fields. `limitations_for_license` and `to_case_label` are **byte-identical (same md5) across 4 files**; `SubmissionPreview` and `*ProviderScorecard` are repeated 6×; `provider_options`/`scorecard`/`saved_output`/`generation_recipe` repeat 5–6×. There is no shared studio-util module.
- **Impact:** A quarter to a third of ~6,700 LOC is copy-paste; any rule change must be applied in up to six places and is easy to apply inconsistently (already realized — see F-025).
- **Suggested fix:** Extract a `studio_common` module for the shared helpers/types; parameterize `SubmissionPreview`/`ProviderScorecard` over the per-studio readiness enum.
- **Confidence:** High

#### [F-014] MVP release-gate marks capability-manifest and safety requirements `Passed` while they are only fixture-true
- **Category:** bad-pattern
- **Severity:** Medium
- **Location:** `crates/soundworks-core/src/mvp_validation.rs:611, 614, 824`
- **Finding:** `check-provider-manifests`, `check-safety-gates`, `epic-req-6`, and `epic-req-7` are hardcoded `MvpValidationStatus::Passed`, though execution is one-off model-id matching (F-007) and safety is an unenforced boolean (F-003). These statuses are literals, not computed from behavior. (Credit: the matrix is otherwise honest — `ready_for_mvp = false`, all 5 runtime-evidence requirements `Pending`/`fixture_only`.)
- **Impact:** Two epic requirements are reported satisfied that are only satisfied at the fixture layer — false confidence, exactly the "report the work as done" risk.
- **Suggested fix:** Downgrade these gates to `Pending` until runtime actually consumes manifests and enforces rights, or compute them from real wiring.
- **Confidence:** High

#### [F-015] Ten UI controls render as enabled buttons with no behavior (incl. Voice/Video/Song "Generate")
- **Category:** dead-code
- **Severity:** Medium
- **Location:** `apps/web/src/App.tsx:1025, 1732, 1777, 1835, 2032, 2477, 2897, 3397, 3782, 3935`
- **Finding:** These `<button>`s have styling and disabled logic but no `onClick`: Multitrack "Render", timeline tool/clip buttons, MVP release-gate, Voice Lab "Convert", Video "Generate", Song "Generate", Review edit-action grid, Rights "Export" gate, workspace scope buttons. The backend exposes `enqueue_runtime_job` for Voice/Video/Song, so the capability exists but is unwired — a partial-implementation gap, not mere decoration.
- **Impact:** Users can click an enabled "Generate"/"Convert"/"Render" and get nothing — looks broken/fake; advertised Voice/Video/Song generation is unreachable from the UI.
- **Suggested fix:** Wire Voice/Video/Song "Generate" to `runRuntimeJob(...)` like TTS/SFX/Samples; render genuinely inert affordances as non-button elements.
- **Confidence:** High

#### [F-016] `.expect()` panics on data-driven paths (malformed WAV, missing fixture)
- **Category:** bad-pattern
- **Severity:** Medium
- **Location:** `crates/soundworks-core/src/project_library.rs:1357-1424` (WAV parser), `asset_library.rs:42-46, 144-148`; `voice_lab.rs:190-193`; `apps/desktop/src/lib.rs:305-341`
- **Finding:** The WAV reader uses `.try_into().expect("slice length")` with chunk arithmetic driven by untrusted size fields; a crafted WAV fed to `save_review_edit`/`export_library_item` can hit an out-of-range slice and panic instead of returning `InvalidData`. `AssetLibraryOverview::reference()` and `VoiceLabOverview::reference()` `.expect()` specific fixtures/ids exist, and the Tauri getters `.expect()` again — a fixture rename panics the running app.
- **Impact:** A hostile/malformed audio file panics the command thread; benign fixture edits crash Voice Lab / library overviews.
- **Suggested fix:** Replace parser `.expect` with checked `InvalidData` errors and bounds checks; return `Result` from reference selection instead of `.expect`.
- **Confidence:** Medium

#### [F-017] `add_asset_to_project` silently skips the default `project-demo`, orphaning imported assets
- **Category:** bad-pattern
- **Severity:** Medium
- **Location:** `crates/soundworks-core/src/project_library.rs:1048-1059`
- **Finding:** The default workspace's `active_project_id` is the synthetic `"project-demo"`, which has no `project.json` on disk. `add_asset_to_project` guards `if project.id != "project-demo"` and skips `write_project`, so the asset→project link is never persisted before the user explicitly creates a project.
- **Impact:** Out of the box, imported artifacts persist as records but are orphaned from any project's `asset_ids`; project asset counts are wrong — silent linkage loss.
- **Suggested fix:** Seed a real `project-demo/project.json` on first run (or refuse a non-persisted project as active) and remove the special case.
- **Confidence:** High

#### [F-018] Overview + full directory scan rebuilt on every command; library computed twice per mutation
- **Category:** efficiency
- **Severity:** Medium
- **Location:** `crates/soundworks-core/src/project_library.rs:977-991` (`action_result`), `216-228`, `1061-1074`; `asset_library.rs:132-152`; `apps/desktop/src/lib.rs:255-263`
- **Finding:** Every mutation ends in `action_result`, which calls `asset_library_overview` (scans all `assets/`, rebuilds the full fixture catalog) **and** `workspace_overview` (which calls `asset_library_overview` again) **and** `read_projects` — two full scans + two fixture rebuilds per mutation. Each runtime command rebuilds a `RuntimeOverview` by re-reading and sorting every `job.json`.
- **Impact:** O(N) disk I/O and allocation per single-item operation, growing unbounded with library/job history that is never pruned; enqueue/mutate latency degrades linearly over the app's lifetime.
- **Suggested fix:** Build the overview once and reuse it; read only the specific job for single-job ops; add an in-memory index invalidated on write, and bounded retention for `jobs/`.
- **Confidence:** High

#### [F-019] Kokoro builds a new Tokio runtime and reloads the ONNX model on every TTS request
- **Category:** efficiency
- **Severity:** Medium
- **Location:** `crates/soundworks-core/src/runtime.rs:1311-1322`
- **Finding:** Each Kokoro job constructs a fresh `tokio::runtime::Builder::new_current_thread()` to `block_on` `KokoroTts::new` (which reloads the ONNX model + voices from disk) and `synth`. The `WarmupStatus::Cold` field exists but is unused.
- **Impact:** The ONNX model is re-initialized from disk on every TTS request, adding large per-request latency; no warmup/caching despite the field tracking it.
- **Suggested fix:** Hold a long-lived `KokoroTts` (LRU keyed by cache_root+voice) and one shared runtime; honor the warmup state.
- **Confidence:** Medium

#### [F-020] ~1,180 lines of SQL `SCHEMA_MIGRATIONS` are unused by the file-backed store
- **Category:** dead-code
- **Severity:** Medium
- **Location:** `crates/soundworks-core/src/storage.rs:5-1183`
- **Finding:** `storage.rs` defines 19 `SchemaMigration` DDL blocks (projects, audio_assets, dozens of workflow tables), but the live persistence layer is entirely JSON-file-based (`read_json`/`write_json`). No SQLite/connection code executes these migrations anywhere in the crate. The actual logic in the file is ~70 lines (`StoragePathAllocator`, path helpers).
- **Impact:** ~1,180 lines of unmaintained schema that does not reflect the on-disk JSON reality and is free to drift; misleads readers about where state lives.
- **Suggested fix:** Remove it, or move it behind the (currently nonexistent) DB module that would consume it, and document that the live store is JSON sidecars.
- **Confidence:** Medium

#### [F-021] `can_submit` / submission validation across the six studios is display-only theater
- **Category:** dead-code
- **Severity:** Medium
- **Location:** `crates/soundworks-core/src/tts.rs:334-441`, `sfx.rs:362-473`, `samples.rs:378-487`, `songs.rs:420-529`, `voice_lab.rs:184-279`, `video_to_audio.rs:698-774`
- **Finding:** Each studio computes `blocking_reasons`/`warnings`/`can_submit` against fixed reference data and serializes the result; nothing consumes it to permit/deny a job (the real path is `runtime.rs`, which references none of these types). In four studios `can_submit` is always `false` (no runnable provider in the reference catalog — tests assert this); in two it is always `true`. The dozens of validation branches never run against user input.
- **Impact:** Substantial logic reads as real enforcement but is unreachable; future readers may rely on it. Maintenance weight with no behavioral effect.
- **Suggested fix:** Either document these as contract-preview output, or route real `runtime.rs` submission through them so they become authoritative. (Scope decision for the owner.)
- **Confidence:** High

#### [F-022] Loudness/true-peak are toy approximations presented as authoritative measurements
- **Category:** readability
- **Severity:** Medium
- **Location:** `crates/soundworks-core/src/project_library.rs:1493-1523` (`pcm_stats`, `normalize_to_lufs`)
- **Finding:** `loudness_lufs = 20*log10(rms)` is plain sample RMS in dBFS, not gated LUFS (BS.1770); `true_peak_dbfs` is sample peak, not oversampled true peak. These are written into `technical.loudness_lufs`/`true_peak_dbfs`, surfaced in sidecars/manifests, and used to derive `has_clipping` and export gating.
- **Impact:** Misleading metadata; downstream consumers (export gating, SceneWorks handoff) may treat non-standard values as compliant. The epic's "normalize/loudness" review tooling is not actually loudness-correct.
- **Suggested fix:** Rename to "approximate RMS dBFS" or implement BS.1770 gating + oversampled true-peak; document the approximation at call sites.
- **Confidence:** High

#### [F-023] The only path sanitizer (`StoragePathAllocator` / `clean_path_segment`) is dead relative to real persistence
- **Category:** dead-code
- **Severity:** Medium
- **Location:** `crates/soundworks-core/src/storage.rs:1202-1264`; only callers in `asset_library.rs:35, 538-541, 658`
- **Finding:** The one path-validating builder is invoked solely by fixture construction; the real `ProjectLibraryStore` writes build paths by hand and never touch it. The security control exists but guards only fake data. (This is the fix that's "sitting right there" for F-001/F-004.)
- **Impact:** Gives a false impression that path inputs are validated while the durable writes are unguarded.
- **Suggested fix:** Route all real write paths through the allocator/validator so the guard becomes load-bearing.
- **Confidence:** High

---

## Low findings

#### [F-024] Native-model set duplicated as string literals across three sites
- **Category:** redundant
- **Severity:** Low
- **Location:** `crates/soundworks-core/src/runtime.rs:83-98, 1158-1168, 1881-1909`
- **Finding:** The executable model ids (`kokoro-82m`, `native-procedural-sfx`, `native-procedural-music`, `native-smoke`) are listed as literals in native injection, `run_adapter`, and `adapter_for_model`, kept in sync by hand. A typo silently routes a model into the 440 Hz `write_smoke_audio` catch-all instead of its synthesizer.
- **Impact:** Adding/renaming a model needs edits in 3 places; silent mis-dispatch on mismatch.
- **Suggested fix:** A `NativeModel` enum / registry (id → adapter fn) resolved once.
- **Confidence:** High

#### [F-025] `limitations_for_safety` silently diverges across its four copies
- **Category:** bad-pattern
- **Severity:** Low
- **Location:** `crates/soundworks-core/src/tts.rs:553-568`, `sfx.rs:664-679`, `samples.rs:652-667`, `songs.rs:699-718`
- **Finding:** Unlike the byte-identical `limitations_for_license`, the four copies of `limitations_for_safety` differ (TTS checks consent; sfx/samples check commercial-use; songs also checks provenance). The differences are plausibly intentional but invisible because the function is copy-pasted under one name — the drift F-013 predicts, already realized.
- **Impact:** A fix in one copy won't propagate; intended differences look accidental.
- **Suggested fix:** Centralize into one helper taking explicit per-studio flags.
- **Confidence:** High

#### [F-026] `to_case_label` derives kebab labels from `Debug` formatting, duplicated 4×
- **Category:** readability
- **Severity:** Low
- **Location:** `crates/soundworks-core/src/tts.rs:836-851` (and identical in sfx/samples/songs)
- **Finding:** `format!("{:?}", x).to_case_label()` couples display labels to `Debug` output (fragile to derive/variant changes) via a hand-rolled char loop copied four times, though the enums already derive serde `rename_all = "kebab-case"`.
- **Impact:** Fragile and duplicated; display-only.
- **Suggested fix:** Derive the label from the serde representation or a `Display` impl in the shared module (F-013).
- **Confidence:** Medium

#### [F-027] Brittle string-ladder maps action `id` → typed action, defaulting unknowns to `add-tag`
- **Category:** bad-pattern
- **Severity:** Low
- **Location:** `apps/web/src/App.tsx:1399-1411`
- **Finding:** A nested ternary converts `action.id` to a `LibraryMutationAction`; anything unmatched silently becomes `"add-tag"`. New lifecycle actions are silently misrouted to tagging.
- **Impact:** Silent incorrect behavior on config drift.
- **Suggested fix:** Make the action id the typed union (or an explicit lookup) and treat unknown ids as errors.
- **Confidence:** Medium

#### [F-028] Studio cards map to nav views by array index, not id
- **Category:** bad-pattern
- **Severity:** Low
- **Location:** `apps/web/src/App.tsx:1138-1164, 282-290`
- **Finding:** The Studios grid derives both icon and destination view from the loop index over `overview.studios` (`navSections[1].items[index % len].id`). If the studios fixture and nav order/length diverge, cards deep-link to the wrong screen with no error.
- **Impact:** Fragile positional coupling between two independently-defined lists.
- **Suggested fix:** Give each studio an explicit `viewId`/icon and look it up.
- **Confidence:** Medium

#### [F-029] Non-stable React keys built from rendered values
- **Category:** bad-pattern
- **Severity:** Low
- **Location:** `apps/web/src/App.tsx:3716, 1932, 1936, 2107`
- **Finding:** List keys are composed from display values (waveform peak min/max, tags, effect names) rather than stable ids; duplicate values produce duplicate keys and defeat reconciliation.
- **Impact:** Potential duplicate-key warnings and subtle diffing bugs if lists become dynamic; low while data is static.
- **Suggested fix:** Use array index for fixed presentational lists or a stable id field.
- **Confidence:** Medium

#### [F-030] `RuntimeJobSnapshot` is a 20-field god struct that leaks absolute home paths into payloads
- **Category:** readability
- **Severity:** Low
- **Location:** `crates/soundworks-core/src/runtime.rs:849-871`
- **Finding:** One struct serves as both on-disk record and API DTO, carrying identity, status, four derivable absolute path strings (`record_root` + 3 derivable), logs, artifacts, and error. The persisted absolute paths are also what enables F-004's `retry` traversal.
- **Impact:** Hard to evolve; absolute paths leak the user's home directory into every payload; redundant path fields.
- **Suggested fix:** Store only `record_root`, derive the rest; split persisted record from API DTO; store paths relative to the store root.
- **Confidence:** Medium

#### [F-031] `write_pcm_f32_wav_channels` is misnamed (writes i16 PCM) and lacks bounds guards
- **Category:** bad-pattern
- **Severity:** Low
- **Location:** `crates/soundworks-core/src/runtime.rs:2241-2267`
- **Finding:** The function name implies float WAV but down-converts to 16-bit PCM (format tag 1). `data_bytes = samples.len() as u32 * 2` is a lossy cast with no overflow/`len % channels` guard.
- **Impact:** Misleading name; a future caller with larger buffers could write a corrupt header.
- **Suggested fix:** Rename to `write_pcm16_wav_channels`; compute sizes as `u64` with bounds checks; assert `samples.len() % channels == 0`.
- **Confidence:** Medium

#### [F-032] `VoiceConversionPreview::build` panics if the `rvc` scorecard is absent
- **Category:** bad-pattern
- **Severity:** Low
- **Location:** `crates/soundworks-core/src/voice_lab.rs:190-193`
- **Finding:** `.find(|s| s.candidate_id == "rvc").expect(...)` hard-depends on the literal id existing in the evaluation catalog; a rename panics Voice Lab (through two `expect` layers), unlike the graceful "unavailable" fallback the other five studios use.
- **Impact:** A benign catalog rename crashes the running app.
- **Suggested fix:** Return `Option` and produce a blocked preview on `None`, mirroring the other studios.
- **Confidence:** High

#### [F-033] Magic candidate count `== 28` duplicated across modules
- **Category:** readability
- **Severity:** Low
- **Location:** `crates/soundworks-core/src/model_manager.rs:881-888`; `evaluation.rs:1210`; `apps/desktop/src/lib.rs` tests
- **Finding:** A "coverage" validation check passes only when `candidate_count == 28`; the literal is repeated across model_manager, evaluation tests, and lib tests. It validates a count, not coverage of named candidates.
- **Impact:** Adding/removing a candidate fails an unrelated check and requires editing several literals.
- **Suggested fix:** Derive the expected set from the named candidate-id list and check membership, not length.
- **Confidence:** Medium

#### [F-034] Duplicated device-selection and WAV-header boilerplate in the runtime
- **Category:** redundant
- **Severity:** Low
- **Location:** `crates/soundworks-core/src/runtime.rs:619-708` (`native_procedural_sfx`/`_music` near-identical), repeated "first available accelerator" closure (~6×), `write_smoke_wav` (~L2212) vs `write_pcm_f32_wav_channels` (~L2241)
- **Finding:** The two native model-state constructors differ only in id/name/workflows yet duplicate the full `ModelRuntimeState` incl. identical `RuntimeCompatibility`; the accelerator lookup is copy-pasted; `write_smoke_wav` re-implements the RIFF header the other writer already produces.
- **Impact:** ~200 lines of drift-prone duplication.
- **Suggested fix:** Extract `native_state(...)` and `selected_accelerator(inventory)` helpers; have `write_smoke_wav` reuse the PCM writer.
- **Confidence:** High

---

## Informational

#### [F-035] `timestamp_millis` swallows clock errors to 0, risking job-id collisions
- **Category:** bad-pattern
- **Severity:** Info
- **Location:** `crates/soundworks-core/src/runtime.rs:1986-1990`
- **Finding:** A `SystemTime` error maps to `0`, and job ids are `job-<wf>-<millis>`, so a clock fault (or two enqueues in the same millisecond) can collide and overwrite a prior job directory.
- **Impact:** Exotic, but a collision silently clobbers job history.
- **Suggested fix:** Add a monotonic counter or random suffix to job ids.
- **Confidence:** Medium

#### [F-036] External `sourceUrl` rendered without `rel="noopener noreferrer"`
- **Category:** security
- **Severity:** Info
- **Location:** `apps/web/src/App.tsx:2016`
- **Finding:** Component-decision URLs render as plain anchors. Data is trusted fixtures today, and there is **no** `dangerouslySetInnerHTML`/`innerHTML`/`eval` anywhere in the web src (verified) — XSS surface is effectively nil. Noted only for when URLs become backend-driven.
- **Impact:** Negligible today.
- **Suggested fix:** Add `rel="noopener noreferrer"` and validate the scheme if URLs ever become dynamic.
- **Confidence:** High

#### [F-037] "Queue / worker boundary / Cancellable" labels overstate the synchronous implementation
- **Category:** readability
- **Severity:** Info
- **Location:** `crates/soundworks-core/src/runtime.rs` (e.g. ~L187, ~L1013, ~L1223)
- **Finding:** User-facing strings and state names describe an async queue + worker + cancellation boundary that does not exist (see F-006).
- **Impact:** Misleads maintainers and anyone reading job records into assuming a concurrency model.
- **Suggested fix:** Implement the async worker (F-006) or soften the language and mark synchronous jobs `NotCancellable`.
- **Confidence:** High

---

## Themes and systemic observations

1. **The real spine is genuine but thin; the scaffolding is vast.** Verified-real: procedural WAV/PCM synthesis, Kokoro ONNX TTS, durable JSON+media persistence, on-disk model-cache verification (`model_manager.rs` is the most honest module), and a working SceneWorks-style nav shell. Everything else — six studios, manifest catalog, rights policy, export/composition/review overviews, `appData.ts` — is `reference()`/fixture contract. The recovery commits did move SoundWorks past "demo shell," but only a few millimeters of the surface area actually execute.

2. **Unsanitized identifiers joined into filesystem paths is systemic, not a one-off.** F-001 (asset writes), F-004 (job store), and F-023 (the dead sanitizer) are the same root cause in three places: caller-supplied strings reach `Path::join` while the correct validator sits unused. This is the most urgent class to fix and should be fixed centrally.

3. **"Capability-driven" is aspirational.** The epic's #1 architecture requirement is contradicted by hardcoded model-id dispatch off a catalog disjoint from the manifest abstraction (F-007), and the MVP gate hides this by hardcoding the relevant checks to `Passed` (F-014).

4. **Safety/consent is displayed, not enforced.** Against epic requirement #6, the only runtime gate is a self-asserted boolean for two of seven workflows; `rights.rs` is entirely fixtures (F-003).

5. **Massive structural duplication is the dominant maintainability tax.** Six stamped-out studios (F-013), a TS clone of Rust fixtures (F-012), triplicated native-model lists (F-024), and divergent copy-pasted helpers (F-025/F-026). Combined, well over 8,000 lines are duplication or display-only scaffolding.

6. **No async execution model.** Synthesis is synchronous on the command thread (F-006), every command rebuilds state from disk with no caching or locking (F-005/F-018/F-019). It works at demo scale and will degrade and corrupt under real use.

7. **The project is mostly honest about its immaturity** — `mvp_validation.ready_for_mvp = false`, runtime evidence `Pending`, fallbacks labeled "requires the Tauri desktop shell" — which makes the two hardcoded-`Passed` gates (F-014) and the silent web-mode mock (F-008) the more important to correct, since they are where the self-assessment lies.

## Coverage notes

- **Reviewed in full:** all 18 Rust modules in `crates/soundworks-core`, the Tauri boundary (`apps/desktop/src/lib.rs`, `main.rs`, `build.rs`, `tauri.conf.json`), and all web source except a line-by-line pass of `appData.ts` (characterized as pure fixture data, not logic). The two security Criticals/Highs (F-001, F-003, plus F-004/F-023 evidence) were verified by direct read, not delegation.
- **Not deeply audited:** `styles.css` (3,116 lines) was assessed for design-token structure and parity, not unused-selector pruning; `Cargo.lock`/`package-lock.json` dependency CVEs were not scanned (no manifest-visible red flags, but a `cargo audit` / `npm audit` pass is recommended); the Kokoro/`ort` model-loading path was read but not run against a real model cache; no runtime/build was executed (`npm run check` not run) — findings are static.
- **Out of scope:** generated icons, `node_modules`, `target`, and the 22 markdown docs under `docs/` (read for orientation, not reviewed as code).
