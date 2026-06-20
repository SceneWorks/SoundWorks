// DR-02 + UX-05: SFX + Ambience studio. UX-05 adds an editable generation draft
// (prompt, category preset, duration, intensity, realism, loopable — all consumed
// by the native procedural SFX adapter), a live progress panel driven by the
// UX-F1 job poll, and inline playback of the saved result. The variant grid,
// provider panels, and gating are preserved.
import { useEffect, useRef, useState } from "react";
import { CircleCheck, Play, SlidersHorizontal } from "lucide-react";
import {
  GenerationPanel,
  HeroStat,
  MainSurface,
  ModelAvailabilityGate,
  PlaybackControl,
  SectionHeading,
  SurfaceHeader,
} from "../components";
import { formatDuration, statusLabel, workflowLabel } from "../viewModel";
import { useAppContext } from "./context";

export function SfxScreen() {
  const {
    sfxStudio,
    sfxRuntimeModel,
    runRuntimeJob,
    runtimeOperation,
    cancelRuntimeOperation,
    retryRuntimeOperation,
    assetLibrary,
    libraryPlayback,
    previewLibraryItem,
    overview,
    sfxCandidateFocus,
    setActiveView,
  } = useAppContext();

  const editedRef = useRef(false);
  const [prompt, setPrompt] = useState(sfxStudio.prompt.text);
  const [category, setCategory] = useState<string>(sfxStudio.prompt.category);
  const [durationMs, setDurationMs] = useState(sfxStudio.controls.durationMs);
  const [intensity, setIntensity] = useState(sfxStudio.controls.intensity);
  const [realism, setRealism] = useState(sfxStudio.controls.realism);
  const [loopable, setLoopable] = useState(sfxStudio.controls.loopable);

  useEffect(() => {
    if (!editedRef.current) {
      setPrompt(sfxStudio.prompt.text);
      setCategory(sfxStudio.prompt.category);
      setDurationMs(sfxStudio.controls.durationMs);
      setIntensity(sfxStudio.controls.intensity);
      setRealism(sfxStudio.controls.realism);
      setLoopable(sfxStudio.controls.loopable);
    }
  }, [
    sfxStudio.prompt.text,
    sfxStudio.prompt.category,
    sfxStudio.controls.durationMs,
    sfxStudio.controls.intensity,
    sfxStudio.controls.realism,
    sfxStudio.controls.loopable,
  ]);

  const trimmed = prompt.trim();
  const blockReason = !sfxRuntimeModel
    ? "Install an SFX model to generate."
    : trimmed.length === 0
      ? "Enter a sound prompt to generate."
      : null;

  function generate() {
    runRuntimeJob(loopable ? "ambience" : "sfx", prompt, {
      category,
      negativePrompt: sfxStudio.prompt.negativePrompt,
      tags: sfxStudio.prompt.tags,
      durationMs,
      loopable,
      intensity,
      realism,
    });
  }

  const selectedItemId = assetLibrary.selectedItem?.item.id;

  return (
    <section className="sfx-studio-panel" aria-label="SFX and Ambience">
      <SurfaceHeader
        eyebrow="SFX + Ambience"
        title={statusLabel(sfxStudio.prompt.category)}
        actions={
          <button
            className="primary-action sfx-action"
            disabled={Boolean(blockReason)}
            onClick={generate}
            type="button"
            title={blockReason ?? "Queue SFX generation"}
          >
            <Play aria-hidden="true" size={18} />
            <span>{sfxRuntimeModel ? "Generate" : "Blocked"}</span>
          </button>
        }
        stats={
          <>
            <HeroStat label="variants" value={overview.sfxStudio.variantCount} />
            <HeroStat label="saved" value={overview.sfxStudio.savedOutputCount} />
            <HeroStat
              label="scorecards"
              value={overview.sfxStudio.scorecardCount}
            />
            <HeroStat
              label={loopable ? "loopable" : "one-shot"}
              value={formatDuration(durationMs)}
            />
          </>
        }
      />

      <ModelAvailabilityGate
        installed={Boolean(sfxRuntimeModel)}
        label="SFX"
        onOpenModelManager={() => setActiveView("models")}
      />

      <MainSurface className="studio-compose" ariaLabel="Compose sound effect">
        <SectionHeading title="Compose" eyebrow="prompt + shape" />
        <label className="field">
          <span>Prompt</span>
          <textarea
            className="field-input"
            rows={3}
            value={prompt}
            onChange={(event) => {
              editedRef.current = true;
              setPrompt(event.target.value);
            }}
            placeholder="Describe the sound effect or ambience…"
          />
          <small className="field-hint">{trimmed.length} characters</small>
        </label>
        <div className="field-row">
          <label className="field">
            <span>Category</span>
            <select
              className="field-input"
              value={category}
              onChange={(event) => {
                editedRef.current = true;
                setCategory(event.target.value);
                const preset = sfxStudio.categoryPresets.find(
                  (item) => item.category === event.target.value,
                );
                if (preset) {
                  setDurationMs(preset.defaultDurationMs);
                  setLoopable(preset.loopableDefault);
                }
              }}
            >
              {sfxStudio.categoryPresets.map((preset) => (
                <option key={preset.category} value={preset.category}>
                  {preset.label}
                </option>
              ))}
            </select>
          </label>
          <label className="field">
            <span>Duration {formatDuration(durationMs)}</span>
            <input
              className="field-input"
              type="range"
              min={250}
              max={30000}
              step={250}
              value={durationMs}
              onChange={(event) => {
                editedRef.current = true;
                setDurationMs(Number(event.target.value));
              }}
            />
          </label>
        </div>
        <div className="field-row">
          <label className="field">
            <span>Intensity {intensity}</span>
            <input
              className="field-input"
              type="range"
              min={1}
              max={100}
              value={intensity}
              onChange={(event) => {
                editedRef.current = true;
                setIntensity(Number(event.target.value));
              }}
            />
          </label>
          <label className="field">
            <span>Realism {realism}</span>
            <input
              className="field-input"
              type="range"
              min={1}
              max={100}
              value={realism}
              onChange={(event) => {
                editedRef.current = true;
                setRealism(Number(event.target.value));
              }}
            />
          </label>
          <label className="field field-check">
            <input
              type="checkbox"
              checked={loopable}
              onChange={(event) => {
                editedRef.current = true;
                setLoopable(event.target.checked);
              }}
            />
            <span>Loopable</span>
          </label>
        </div>
      </MainSurface>

      <GenerationPanel
        job={runtimeOperation}
        workflows={["sfx", "ambience"]}
        typeLabel="SFX"
        onCancel={cancelRuntimeOperation}
        onRetry={retryRuntimeOperation}
      >
        <button
          type="button"
          className="secondary-action"
          disabled={!selectedItemId}
          onClick={() => selectedItemId && previewLibraryItem(selectedItemId)}
          title="Play the saved clip"
        >
          <Play aria-hidden="true" size={16} />
          <span>Play latest</span>
        </button>
        <PlaybackControl playback={libraryPlayback} />
      </GenerationPanel>

      <div className="sfx-layout">
        <div className="sfx-main">
          <div className="sfx-variant-grid" aria-label="Generated variants">
            {sfxStudio.variants.map((variant) => (
              <article
                className={
                  variant.selectedForSave
                    ? "sfx-variant selected"
                    : "sfx-variant"
                }
                key={variant.id}
              >
                <div className="sfx-variant-title">
                  <strong>{variant.label}</strong>
                  <span>{statusLabel(variant.assetKind)}</span>
                </div>
                <small>
                  {formatDuration(variant.durationMs)} / {variant.loudnessLufs}{" "}
                  LUFS / {variant.truePeakDbfs} dBTP
                </small>
                <p>
                  {variant.loopPoints
                    ? `loop ${variant.loopPoints.startSample}-${variant.loopPoints.endSample}`
                    : "one-shot preview"}
                </p>
                <div className="candidate-strip">
                  {variant.tags.slice(0, 4).map((tag, index) => (
                    <span key={index}>{tag}</span>
                  ))}
                </div>
              </article>
            ))}
          </div>
        </div>

        <div className="sfx-side">
          <MainSurface ariaLabel="SFX provider options">
            <SectionHeading
              title="Providers"
              eyebrow={`${sfxStudio.providerOptions.length}`}
            />
            {sfxStudio.providerOptions.map((provider) => (
              <article
                className="sfx-provider-option"
                key={`${provider.workflow}-${provider.modelId}`}
              >
                <strong>{workflowLabel(provider.workflow)}</strong>
                <small>
                  {statusLabel(provider.installStatus)} /{" "}
                  {statusLabel(provider.outputAssetKind)} /{" "}
                  {provider.sampleRateHz} Hz
                </small>
                <p>{provider.supportedControls.map(statusLabel).join(" / ")}</p>
              </article>
            ))}
          </MainSurface>

          <MainSurface ariaLabel="SFX provider scorecards">
            <SectionHeading
              title="Scorecards"
              eyebrow={`${sfxStudio.providerScorecards.length}`}
            />
            <div className="voice-provider-list">
              {sfxCandidateFocus.map((scorecard) => (
                <article
                  className={`voice-provider ${scorecard.readiness}`}
                  key={scorecard.candidateId}
                >
                  <div>
                    <strong>{scorecard.name}</strong>
                    <small>
                      {statusLabel(scorecard.readiness)} /{" "}
                      {scorecard.lanes.map(workflowLabel).join(" / ")}
                    </small>
                    <p>{scorecard.notes}</p>
                    {scorecard.blockers[0] ? <p>{scorecard.blockers[0]}</p> : null}
                  </div>
                  {scorecard.recommended ? <span>pick</span> : null}
                </article>
              ))}
            </div>
          </MainSurface>

          <MainSurface ariaLabel="Saved SFX outputs">
            <SectionHeading
              title="Outputs"
              eyebrow={sfxStudio.submission.job.status}
            />
            {sfxStudio.savedOutputs.map((output) => (
              <div className="output-card" key={output.variantId}>
                <strong>{output.asset.name}</strong>
                <small>
                  {output.asset.kind} / {output.asset.currentVersionId}
                </small>
                <p>{output.version.file.storagePath}</p>
              </div>
            ))}
          </MainSurface>
        </div>
      </div>

      <div className="sfx-review-grid">
        <ol className="voice-checks" aria-label="SFX post-processing">
          {sfxStudio.postProcessingActions.map((action) => (
            <li className={action.enabled ? "ready" : "warning"} key={action.id}>
              <SlidersHorizontal aria-hidden="true" size={16} />
              <span>{action.summary}</span>
            </li>
          ))}
        </ol>
        <ol className="voice-checks" aria-label="SFX validation checks">
          {sfxStudio.validationChecks.map((check) => (
            <li className={check.status} key={check.id}>
              <CircleCheck aria-hidden="true" size={16} />
              <span>{check.summary}</span>
            </li>
          ))}
        </ol>
      </div>
    </section>
  );
}
