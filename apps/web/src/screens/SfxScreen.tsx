// DR-02: SFX + Ambience studio. Extracted from App.tsx and rebuilt on the shared
// grammar (SurfaceHeader hero + HeroStat for the header/metrics, MainSurface +
// SectionHeading for the side panels) in place of the bespoke sfx-header /
// sfx-metrics / sfx-prompt-panel / tts-subpanel / subpanel-heading classes,
// following the TtsScreen template. All Generate wiring, gating, and data
// bindings are preserved verbatim.
import { CircleCheck, Play, SlidersHorizontal } from "lucide-react";
import {
  HeroStat,
  MainSurface,
  SectionHeading,
  SurfaceHeader,
  ModelAvailabilityGate,
} from "../components";
import { formatDuration, statusLabel, workflowLabel } from "../viewModel";
import { useAppContext } from "./context";

export function SfxScreen() {
  const {
    sfxStudio,
    sfxRuntimeModel,
    runRuntimeJob,
    overview,
    sfxCandidateFocus,
    setActiveView,
  } = useAppContext();

  return (
    <section className="sfx-studio-panel" aria-label="SFX and Ambience">
      <SurfaceHeader
        eyebrow="SFX + Ambience"
        title={statusLabel(sfxStudio.prompt.category)}
        actions={
          <button
            className="primary-action sfx-action"
            disabled={!sfxRuntimeModel}
            onClick={() =>
              runRuntimeJob("sfx", sfxStudio.prompt.text, {
                category: sfxStudio.prompt.category,
                negativePrompt: sfxStudio.prompt.negativePrompt,
                tags: sfxStudio.prompt.tags,
                durationMs: sfxStudio.controls.durationMs,
                loopable: sfxStudio.controls.loopable,
                intensity: sfxStudio.controls.intensity,
                realism: sfxStudio.controls.realism,
              })
            }
            type="button"
            title="Queue SFX generation"
          >
            <Play aria-hidden="true" size={18} />
            <span>{sfxRuntimeModel ? "Generate" : "Blocked"}</span>
          </button>
        }
        stats={
          <>
            <HeroStat
              label="variants"
              value={overview.sfxStudio.variantCount}
            />
            <HeroStat
              label="saved"
              value={overview.sfxStudio.savedOutputCount}
            />
            <HeroStat
              label="scorecards"
              value={overview.sfxStudio.scorecardCount}
            />
            <HeroStat
              label={sfxStudio.controls.loopable ? "loopable" : "one-shot"}
              value={formatDuration(sfxStudio.controls.durationMs)}
            />
          </>
        }
      />

      <ModelAvailabilityGate
        installed={Boolean(sfxRuntimeModel)}
        label="SFX"
        onOpenModelManager={() => setActiveView("models")}
      />

      <div className="sfx-layout">
        <div className="sfx-main">
          <section className="sfx-prompt-panel" aria-label="SFX prompt">
            <div className="subpanel-heading">
              <h3>Prompt</h3>
              <span>{sfxStudio.prompt.tags.length}</span>
            </div>
            <p>{sfxStudio.prompt.text}</p>
            <small>{sfxStudio.prompt.negativePrompt}</small>
            <div className="candidate-strip">
              {sfxStudio.prompt.tags.map((tag, index) => (
                <span key={index}>{tag}</span>
              ))}
            </div>
          </section>

          <div className="sfx-control-grid" aria-label="SFX controls">
            <div>
              <strong>{sfxStudio.controls.variationCount}</strong>
              <span>batch</span>
            </div>
            <div>
              <strong>{sfxStudio.controls.intensity}</strong>
              <span>intensity</span>
            </div>
            <div>
              <strong>{sfxStudio.controls.realism}</strong>
              <span>realism</span>
            </div>
            <div>
              <strong>{sfxStudio.controls.loopCrossfadeMs}ms</strong>
              <span>crossfade</span>
            </div>
          </div>

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
                  {formatDuration(variant.durationMs)} /{" "}
                  {variant.loudnessLufs} LUFS / {variant.truePeakDbfs} dBTP
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
                <p>
                  {provider.supportedControls.map(statusLabel).join(" / ")}
                </p>
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
                    {scorecard.blockers[0] ? (
                      <p>{scorecard.blockers[0]}</p>
                    ) : null}
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
            <li
              className={action.enabled ? "ready" : "warning"}
              key={action.id}
            >
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
