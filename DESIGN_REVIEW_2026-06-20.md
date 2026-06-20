# SoundWorks Design Review — SceneWorks Parity — 2026-06-20

Reference app: `/Users/michael/Repos/SceneWorks/apps/web/src` · Subject: `/Users/michael/Repos/SoundWorks/apps/web/src`

## Overall impression

SoundWorks looks *adjacent* to SceneWorks but is not built from the same system. It copied a **subset of the design tokens** (36 of SceneWorks' 52 root vars, same names) and the **outer shell class names** (`.app`/`.sidebar`/`.topbar`/`.workspace`), then **discarded SceneWorks' entire reusable component grammar and re-invented every screen with bespoke, single-use classes** (`tts-subpanel` ×55, `subpanel-heading` ×61, `voice-checks`, `candidate-strip`, `samples-*`…). It has **none** of SceneWorks' shared components — `.main-surface`, `.surface-header`, `.nav-item`, `.toolbar`, `.segmented-control`, `.status-badge`, `.model-card`, `.empty-panel`, `.accent-picker`, `.icon-btn`, `.asset-tile` all return **0** matches. The single biggest gap is that **the theme system is gone entirely**: SceneWorks has a light/dark toggle + 7 switchable accents persisted to localStorage and the server; SoundWorks has no theme controls, no dark mode, no accent picker, and a hardcoded single light palette.

**Confidence: High** on all structural claims (grep-verified across both codebases). The biggest opportunity is not a reskin — it is adopting SceneWorks' component grammar and theme system so every screen inherits the look instead of re-inventing it.

A note on your three specific reports, corrected against the code:
- **"None of the pages are wired."** Partly true. In the **browser preview** (`npm run dev`) *nothing* is live — every backend call falls back to fixtures, so the whole app is a mock (see code review F-008). In the **desktop shell**, ~7 of 16 screens have working primary actions; but **6 marquee buttons are genuinely dead** (no handler at all): Multitrack **Render**, Voice Lab **Convert**, Video **Generate**, Song **Generate**, Rights **Export**, Validation **Ready**.
- **"Models page is complete chaos."** Confirmed — one scrolling surface stacks **8 unrelated sections** with **5 competing status vocabularies** over **28 candidates**. Detail in §4.
- **"Download controls throw errors."** Confirmed, and it is **by design, not a crash**: there is no downloader — `ModelManagerOperation::install` is hardcoded to return `Failed` for every input, and the UI faithfully renders that as a red failure banner on every click. Detail in §4.

---

## 1. Theme & accent controls — the headline parity gap 🔴

SceneWorks ships a two-axis theme system; SoundWorks ships none.

| Capability | SceneWorks | SoundWorks |
|---|---|---|
| Light/dark toggle | Moon/Sun `.icon-btn` in topbar (`App.jsx:1988-1995`) → flips `data-theme` on `<html>` | **Absent** |
| Accent picker | 7-swatch `.accent-picker` in topbar (`App.jsx:1974-1987`) → flips `data-accent` | **Absent** |
| CSS dark overrides | `[data-theme="dark"]` block (`styles.css:78-108`) + `color-scheme` | **Absent** (single `:root`, light only) |
| Accent palettes | 7× `[data-accent="…"]` hue blocks (`styles.css:113-123`) | **Absent**; accent hue hardcoded `oklch(0.66 0.11 178)` |
| Persistence | `localStorage` (`sceneworks-theme`/`sceneworks-accent`) + server `/api/v1/ui-preferences` | **None** |
| OS preference | `color-scheme` set per theme | No `prefers-color-scheme` fallback |

| Finding | Severity | Recommendation |
|---|---|---|
| No theme control anywhere in the UI; light-only | 🔴 Critical | Port SceneWorks' theme axis verbatim: add `[data-theme="dark"]` overrides to `styles.css`, set `data-theme` on `document.documentElement` from React state, add the Moon/Sun `.icon-btn` to the topbar, persist to a `soundworks-theme` localStorage key (and the Tauri settings store for cross-launch durability — origin changes per launch, so localStorage alone is unreliable). |
| No accent picker; accent hue hardcoded | 🔴 Critical | Re-introduce the **`--accent-h`** indirection (SoundWorks currently inlines hue 178 — `--accent: oklch(0.66 0.11 178)` — which makes accents un-switchable). Add the 7 `[data-accent]` hue blocks and a SoundWorks `accents.js`, render the `.accent-picker` swatch row in the topbar, persist `soundworks-accent`. |
| Token set is a drifting partial copy (36 vs 52 vars) | 🟡 Moderate | Re-sync the token block to SceneWorks so future SceneWorks token changes can be lifted across. Missing tokens include the dark ramp, `--warm*`, `--side-w`, several `--shadow-*`/`--gap-*`, and brand/logo tokens. |

Because every SceneWorks component reads `--accent`/`--bg`/`--surface`/`--text`, **once the `data-theme`/`data-accent` attributes and override blocks exist, the whole app recolors with zero per-component work** — this is the highest-leverage fix in the review.

---

## 2. Look & feel — component grammar, not paint 🔴

The reason it "doesn't feel like SceneWorks" is structural: SoundWorks reinvented the furniture.

**What draws the eye / coherence problem:** each screen uses its own one-off class names, so spacing, headers, badges, and cards differ subtly screen-to-screen. SceneWorks gets consistency *for free* because every screen is assembled from the same ~15 shared classes.

| Element | SceneWorks (the grammar to adopt) | SoundWorks today | Recommendation |
|---|---|---|---|
| Screen container | `.main-surface` + `<screen>-surface` modifier | bespoke `panel`/`tts-subpanel`/`samples-studio-panel` | Standardize every screen on `.main-surface` |
| Screen header | `.surface-header.hero` → `.section-heading` (`.eyebrow` + `<h2>`) + `.hero-blurb` + `.hero-stats` | `subpanel-heading`/`samples-header`/`eyebrow` ad hoc | Adopt `.surface-header.hero` + `.section-heading` everywhere |
| Toolbars | `.toolbar` (flex-wrap actions/filters) | per-screen action rows | Use `.toolbar` |
| Tabs/modes | `.segmented-control` / `.compact-segment` | none | Use `.segmented-control` for the mode switches |
| Status chips | `.status-badge` (`.installed`/`.warning`/`.failed`) | raw colored dots + free text | Use `.status-badge` with the 3 states |
| Cards/grids | `.asset-grid`/`.asset-tile`, `.model-grid`/`.model-card`, `.review-card` | `output-card`/`candidate-strip`/`samples-variant-grid` | Adopt the shared card/grid classes |
| Empty states | `.empty-panel` / `.compact-panel` | inconsistent inline text | Use `.empty-panel` |
| Side nav items | `.nav-item`(`.active`) + `.nav-label` + `.nav-pulse` + accent left-bar | custom nav markup (no `.nav-item`) | Adopt `.nav-item` so the active-state accent bar matches |
| Topbar status | `.status-pill`, `.queue-chip`, `.icon-btn` | custom `queue-chip` only | Adopt `.status-pill`/`.icon-btn` for parity |
| Progress/cancel/retry/errors | shared **`<WorkerProgressCard>`** used by every studio + the model manager | hand-rolled per screen | Build one `WorkerProgressCard` equivalent and reuse it (also fixes §4 download UX) |

| Finding | Severity | Recommendation |
|---|---|---|
| No shared component layer; every screen rolls bespoke classes | 🔴 Critical | Extract a SceneWorks-parallel component set (`SurfaceHeader`, `Toolbar`, `SegmentedControl`, `StatusBadge`, `ModelCard`, `EmptyPanel`, `WorkerProgressCard`) and rebuild screens against it. This is the bulk of the "make it feel like SceneWorks" work. |
| `App.tsx` is one 4,540-line component (vs SceneWorks' 2,137-line shell + ~30 screen files + ~40 components) | 🟡 Moderate | Split per `ActiveView` screen (tracked as code-review F-010 / sc-6648). Decomposition is a prerequisite for reusing shared components cleanly. |

---

## 3. Page coherence & wiring 🟡

**Visual hierarchy is off because two screens are developer dumps, not product screens:**
- **Settings** (`App.tsx:4108-4144`) renders a read-only list of architecture layers and Tauri command names — there is no settings *control* (and, tellingly, this is where SceneWorks puts theme/accent/preferences). It reads as debug output.
- **Models** (§4) reads as a QA console.

**Wiring map** (desktop shell; in the browser preview all of these are fixtures):

| Screen | Primary control | State |
|---|---|---|
| Project, Library, TTS, SFX, Samples, Review, Export, Jobs | create/open, preview/lifecycle, Queue, Generate, Save, Export, cancel/retry | ✅ wired |
| Models | Install / Revalidate | ⚠️ wired but backend always fails (§4) |
| **Multitrack** Render, **Voice Lab** Convert, **Video** Generate, **Song** Generate, **Rights** Export, **Validation** Ready | — | 🔴 **dead — no `onClick`** |
| Settings | — | 🔴 no controls (read-only dump) |

| Finding | Severity | Recommendation |
|---|---|---|
| 6 marquee action buttons have no handler (look primary, do nothing) | 🔴 Critical | Wire Voice/Video/Song **Generate** + Multitrack **Render** to the existing `runRuntimeJob`/runtime path (the backend command exists); make Rights/Validation gates either functional or non-button display (code-review F-015 / sc-6654). |
| Settings screen has no actual settings | 🟡 Moderate | Make Settings the home of theme/accent/preferences (matching SceneWorks); move the architecture/command dump out of the product UI. |
| No "web preview = simulated" signal | 🟡 Moderate | Add a banner when not running under Tauri so the mock state is obvious (code-review F-008 / sc-6646). This is why the app "feels" entirely unwired. |

---

## 4. Models page — chaos + download errors 🔴

### Why it's chaotic
The Models view stacks **8 heterogeneous blocks on one scroll** (`App.tsx:4146-4392`): (1) provider-coverage stats, (2) evaluation scorecard, (3) manager summary stat-grid, (4) cache-root + policy line, (5) lane-readiness list, (6) a 28-candidate cache list (sliced to 10, each with 2 buttons + raw "N of M files"/missing-file/evidence strings), (7) a red operation banner, (8) a validation-checks list. Each candidate carries **5 unreconciled status vocabularies** (`installState`, `evaluationStatus`, `productEligibility`, `evidenceLevel`, `runtimePath`), and **19 of 28** candidates are `blocked`/`research-only` with disabled controls. It reads as an internal QA console.

### Why downloads "throw errors"
Not a crash — a designed dead-end. Path: button `onClick` → `runModelManagerAction(id,"install")` (`App.tsx:4332`) → `installModelCandidate` (`tauri.ts:172`) → `install_model_candidate` (`lib.rs:122`) → **`ModelManagerOperation::install` (`model_manager.rs:416-462`), which has no downloader and returns `Self::failed(...)` on every branch**. The `Succeeded` status variant exists but is never constructed; a desktop test even asserts install of `kokoro-82m` returns `Failed`. So every enabled Download click produces a red failure banner.

### How SceneWorks does it (the target)
SceneWorks' `ModelManagerScreen.jsx` groups models into collapsible `<details class="model-type-group">` (Image/Video/Utility), splits **Recommended** vs **Additional** within each, renders each model as a `.model-card` with one `.status-badge` and a single Download/Resume/Ready button, and delegates **all** progress/cancel/retry/error UI to the shared `<WorkerProgressCard>`. A failed download becomes a **"Resume Download"** affordance, not a dead red banner. Studios that lack a model render `ModelAvailabilityGate` (offer + inline download) instead of their body.

| Finding | Severity | Recommendation |
|---|---|---|
| Install always fails — no downloader | 🔴 Critical | Either implement a real downloader (with verified-cache evidence) or, until then, **don't present a Download button that always errors** — show install instructions / "manual cache required" state. Tracked as code-review F-014 (honest gates) and the model-manager stories. |
| 8 sections + 5 status vocabularies on one surface | 🔴 Critical | Rebuild on SceneWorks' grammar: group by capability lane (TTS/SFX/Samples/Song/Voice) into `model-type-group`, one `.model-card` per model with a single `.status-badge`, move provider-coverage/evaluation-scorecard/validation-checks off this screen (or behind a tab/disclosure). |
| Raw blocker strings shown as primary content | 🟡 Moderate | Demote "missing model.safetensors", "requires a Python runtime" to secondary/expandable detail; lead with a human status. |
| No shared download-progress component | 🟡 Moderate | Build a `WorkerProgressCard` equivalent and use it here and in every studio (§2). |
| Per-studio availability not gated | 🟡 Moderate | Add a `ModelAvailabilityGate` equivalent so a studio with no installed model shows an offer, not broken controls. |

---

## Accessibility (quick pass)
- **Color contrast:** light-only palette is the SceneWorks light ramp, generally adequate; **but no dark mode** means OS-dark users get a bright UI with no escape (🟡).
- **Touch/click targets:** SceneWorks standardizes `--row-h: 40px` / 34px icon buttons; SoundWorks' bespoke controls aren't on that scale — verify against 40px when adopting `.nav-item`/`.icon-btn`.
- **Disabled affordances:** ~22 disabled Install buttons and 6 inert "Generate"/"Render" buttons read as enabled/primary — an affordance-honesty problem more than a contrast one.
- **State semantics:** SceneWorks uses `aria-pressed`/`role="group"` on the accent picker and `StatusDot` semantics; port these with the controls.

## What works well
- The **token foundation is the right one** — SoundWorks already uses SceneWorks' token *names* and OKLCH model, so wiring up `data-theme`/`data-accent` is additive, not a rewrite.
- The **outer shell** (`.app` grid + `.sidebar` + `.topbar` + `.workspace`) already matches SceneWorks' frame, and nav routing now actually switches views.
- Underneath the UI, the **real backend exists** (synthesis, persistence) — so wiring the dead buttons connects to something real, not vapor.

## Priority recommendations (roadmap)

1. **Port the theme system (theme + accent + persistence).** 🔴 Highest leverage, well-bounded: add `[data-theme="dark"]` + 7 `[data-accent]` blocks, restore `--accent-h`, add the Moon/Sun + swatch controls to the topbar, persist via Tauri settings. Recolors the whole app at once and directly closes "same theme controls."
2. **Adopt SceneWorks' shared component grammar.** 🔴 Build `SurfaceHeader`/`Toolbar`/`SegmentedControl`/`StatusBadge`/`ModelCard`/`EmptyPanel`/`WorkerProgressCard`, decompose `App.tsx` per screen (code-review F-010), and rebuild screens against them so they cohere.
3. **Rebuild the Models page on `model-type-group` + `model-card` + `WorkerProgressCard`, and stop showing a Download button that always errors.** 🔴 Move provider-coverage/evaluation/validation off the product surface.
4. **Wire the 6 dead buttons** (Voice/Video/Song Generate, Render, Rights/Validation gates) and **make Settings hold real preferences** (theme/accent live here). 🟡
5. **Add a "web preview — simulated data" banner** so the mock state is legible. 🟡

These are design/UX items; several overlap existing code-review stories under epic [6635 — SoundWorks Remediation](https://app.shortcut.com/trefry/epic/6635) (F-008/sc-6646, F-010/sc-6648, F-014/sc-6653, F-015/sc-6654). The theme-system and component-grammar work is **new** and not yet tracked.
