// DR-02: Samples + Loops studio. Extracted from App.tsx and rebuilt on the
// shared grammar (SurfaceHeader hero + HeroStat for the header/metrics,
// MainSurface + SectionHeading for the sub-panels) in place of the bespoke
// samples-header / samples-metrics / subpanel-heading classes, following the
// TtsScreen template. All Generate wiring, gating, and data bindings are
// preserved verbatim.
import { CircleCheck, Disc3, SlidersHorizontal } from "lucide-react";
import {
  HeroStat,
  MainSurface,
  SectionHeading,
  SurfaceHeader,
} from "../components";
import { formatDuration, statusLabel, workflowLabel } from "../viewModel";
import { useAppContext } from "./context";

export function SamplesScreen() {
  const {
    samplesStudio,
    samplesCandidateFocus,
    runRuntimeJob,
    overview,
  } = useAppContext();

  return (
    <section className="samples-studio-panel" aria-label="Samples and Loops">
      <SurfaceHeader
        eyebrow="Samples + Loops"
        title={samplesStudio.pack.name}
        actions={
          <button
            className="primary-action samples-action"
            disabled={!samplesStudio.submission.canSubmit}
            onClick={() =>
              runRuntimeJob(
                samplesStudio.selectedProvider.workflow,
                samplesStudio.prompt.text,
                {
                  negativePrompt: samplesStudio.prompt.negativePrompt,
                  instrumentFamily: samplesStudio.prompt.instrumentFamily,
                  articulation: samplesStudio.prompt.articulation,
                  tags: samplesStudio.prompt.genreTags,
                  musicalKey: samplesStudio.controls.musicalKey,
                  scale: samplesStudio.controls.scale,
                  bpm: samplesStudio.controls.bpm,
                  bars: samplesStudio.controls.bars,
                  beats: samplesStudio.controls.beats,
                  loopable: samplesStudio.controls.loopable,
                  velocityEnergy: samplesStudio.controls.velocityEnergy,
                  dryWetAmbience: samplesStudio.controls.dryWetAmbience,
                },
              )
            }
            type="button"
            title="Queue sample and loop generation"
          >
            <Disc3 aria-hidden="true" size={18} />
            <span>
              {samplesStudio.submission.canSubmit ? "Generate" : "Blocked"}
            </span>
          </button>
        }
        stats={
          <>
            <HeroStat
              label="variants"
              value={overview.samplesStudio.variantCount}
            />
            <HeroStat
              label="saved"
              value={overview.samplesStudio.savedOutputCount}
            />
            <HeroStat
              label={samplesStudio.controls.musicalKey}
              value={samplesStudio.controls.bpm}
            />
            <HeroStat
              label="scorecards"
              value={overview.samplesStudio.scorecardCount}
            />
          </>
        }
      />

      <div className="samples-layout">
        <div className="samples-main">
          <MainSurface className="sfx-prompt-panel" ariaLabel="Sample prompt">
            <SectionHeading
              title={statusLabel(samplesStudio.prompt.instrumentFamily)}
              eyebrow={samplesStudio.prompt.genreTags.length}
            />
            <p>{samplesStudio.prompt.text}</p>
            <small>{samplesStudio.prompt.negativePrompt}</small>
            <div className="candidate-strip">
              {samplesStudio.prompt.genreTags.map((tag, index) => (
                <span key={index}>{tag}</span>
              ))}
            </div>
          </MainSurface>

          <div className="samples-control-grid" aria-label="Sample controls">
            <div>
              <strong>{samplesStudio.controls.bars} bars</strong>
              <span>{samplesStudio.controls.beats}/4 grid</span>
            </div>
            <div>
              <strong>{samplesStudio.controls.batchSize}</strong>
              <span>batch size</span>
            </div>
            <div>
              <strong>{samplesStudio.controls.velocityEnergy}</strong>
              <span>velocity</span>
            </div>
            <div>
              <strong>{samplesStudio.controls.dryWetAmbience}</strong>
              <span>ambience</span>
            </div>
          </div>

          <div className="samples-variant-grid" aria-label="Sample variants">
            {samplesStudio.variants.map((variant) => (
              <article
                className={
                  variant.selectedForPack
                    ? "samples-variant selected"
                    : "samples-variant"
                }
                key={variant.id}
              >
                <div className="sfx-variant-title">
                  <strong>{variant.label}</strong>
                  <span>{statusLabel(variant.assetKind)}</span>
                </div>
                <small>
                  {formatDuration(variant.durationMs)} /{" "}
                  {variant.bpm ? `${variant.bpm} BPM` : "one-shot"} /{" "}
                  {variant.musicalKey ?? "unpitched"}
                </small>
                <p>
                  {variant.loopPoints
                    ? `loop ${variant.loopPoints.startSample}-${variant.loopPoints.endSample}`
                    : variant.articulation}
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

        <div className="samples-side">
          <MainSurface ariaLabel="Sample providers">
            <SectionHeading
              title="Providers"
              eyebrow={samplesStudio.providerOptions.length}
            />
            {samplesStudio.providerOptions.map((provider) => (
              <article
                className="sfx-provider-option"
                key={`${provider.workflow}-${provider.modelId}`}
              >
                <strong>{workflowLabel(provider.workflow)}</strong>
                <small>
                  {statusLabel(provider.installStatus)} /{" "}
                  {provider.sampleRateHz} Hz /{" "}
                  {provider.supportsLoopPoints ? "loop points" : "metadata"}
                </small>
                <p>{provider.supportedControls.map(statusLabel).join(" / ")}</p>
              </article>
            ))}
          </MainSurface>

          <MainSurface ariaLabel="Sample provider scorecards">
            <SectionHeading
              title="Scorecards"
              eyebrow={samplesStudio.providerScorecards.length}
            />
            <div className="voice-provider-list">
              {samplesCandidateFocus.map((scorecard) => (
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
                  </div>
                  {scorecard.recommended ? <span>pick</span> : null}
                </article>
              ))}
            </div>
          </MainSurface>

          <MainSurface ariaLabel="Sample pack outputs">
            <SectionHeading
              title="Pack"
              eyebrow={samplesStudio.pack.exportFormats.join(" / ")}
            />
            {samplesStudio.savedOutputs.map((output) => (
              <div className="output-card" key={output.variantId}>
                <strong>{output.asset.name}</strong>
                <small>
                  {output.asset.kind} /{" "}
                  {output.version.technical.bpm
                    ? `${output.version.technical.bpm} BPM`
                    : output.version.technical.musicalKey}
                </small>
                <p>{output.version.file.storagePath}</p>
              </div>
            ))}
          </MainSurface>
        </div>
      </div>

      <div className="samples-review-grid">
        <ol className="voice-checks" aria-label="Sample post-processing">
          {samplesStudio.postProcessingActions.map((action) => (
            <li
              className={action.enabled ? "ready" : "warning"}
              key={action.id}
            >
              <SlidersHorizontal aria-hidden="true" size={16} />
              <span>{action.summary}</span>
            </li>
          ))}
        </ol>
        <ol className="voice-checks" aria-label="Sample QA checks">
          {samplesStudio.qaChecks.map((check) => (
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
