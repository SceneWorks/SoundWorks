// DR-02 + UX-06: Samples + Loops studio. UX-06 adds an editable generation draft
// (prompt, bpm, key, bars/beats, velocity, loopable — all consumed by the native
// procedural music adapter), a live progress panel driven by the UX-F1 job poll,
// and inline playback of the saved loop. The variant grid, provider panels, and
// canSubmit gating are preserved.
import { useEffect, useRef, useState } from "react";
import { CircleCheck, Disc3, Play, SlidersHorizontal } from "lucide-react";
import {
  GenerationPanel,
  HeroStat,
  MainSurface,
  PlaybackControl,
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
    runtimeOperation,
    cancelRuntimeOperation,
    retryRuntimeOperation,
    assetLibrary,
    libraryPlayback,
    previewLibraryItem,
    overview,
  } = useAppContext();

  const editedRef = useRef(false);
  const [prompt, setPrompt] = useState(samplesStudio.prompt.text);
  const [bpm, setBpm] = useState(samplesStudio.controls.bpm);
  const [musicalKey, setMusicalKey] = useState(samplesStudio.controls.musicalKey);
  const [bars, setBars] = useState(samplesStudio.controls.bars);
  const [beats, setBeats] = useState(samplesStudio.controls.beats);
  const [loopable, setLoopable] = useState(samplesStudio.controls.loopable);
  const [velocity, setVelocity] = useState(samplesStudio.controls.velocityEnergy);

  useEffect(() => {
    if (!editedRef.current) {
      setPrompt(samplesStudio.prompt.text);
      setBpm(samplesStudio.controls.bpm);
      setMusicalKey(samplesStudio.controls.musicalKey);
      setBars(samplesStudio.controls.bars);
      setBeats(samplesStudio.controls.beats);
      setLoopable(samplesStudio.controls.loopable);
      setVelocity(samplesStudio.controls.velocityEnergy);
    }
  }, [
    samplesStudio.prompt.text,
    samplesStudio.controls.bpm,
    samplesStudio.controls.musicalKey,
    samplesStudio.controls.bars,
    samplesStudio.controls.beats,
    samplesStudio.controls.loopable,
    samplesStudio.controls.velocityEnergy,
  ]);

  const trimmed = prompt.trim();
  const canSubmit = samplesStudio.submission.canSubmit && trimmed.length > 0;
  const blockReason = !samplesStudio.submission.canSubmit
    ? "Submission is blocked — see the QA checks below."
    : trimmed.length === 0
      ? "Enter a prompt to generate."
      : null;

  function generate() {
    runRuntimeJob(samplesStudio.selectedProvider.workflow, prompt, {
      negativePrompt: samplesStudio.prompt.negativePrompt,
      instrumentFamily: samplesStudio.prompt.instrumentFamily,
      articulation: samplesStudio.prompt.articulation,
      tags: samplesStudio.prompt.genreTags,
      musicalKey,
      scale: samplesStudio.controls.scale,
      bpm,
      bars,
      beats,
      loopable,
      velocityEnergy: velocity,
      dryWetAmbience: samplesStudio.controls.dryWetAmbience,
    });
  }

  const selectedItemId = assetLibrary.selectedItem?.item.id;

  return (
    <section className="samples-studio-panel" aria-label="Samples and Loops">
      <SurfaceHeader
        eyebrow="Samples + Loops"
        title={samplesStudio.pack.name}
        actions={
          <button
            className="primary-action samples-action"
            disabled={!canSubmit}
            onClick={generate}
            type="button"
            title={blockReason ?? "Queue sample and loop generation"}
          >
            <Disc3 aria-hidden="true" size={18} />
            <span>{canSubmit ? "Generate" : "Blocked"}</span>
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
            <HeroStat label={musicalKey} value={bpm} />
            <HeroStat
              label="scorecards"
              value={overview.samplesStudio.scorecardCount}
            />
          </>
        }
      />

      <MainSurface className="studio-compose" ariaLabel="Compose loop">
        <SectionHeading title="Compose" eyebrow="prompt + groove" />
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
            placeholder="Describe the instrument, groove, or loop…"
          />
          <small className="field-hint">{trimmed.length} characters</small>
        </label>
        <div className="field-row">
          <label className="field">
            <span>BPM {bpm}</span>
            <input
              className="field-input"
              type="range"
              min={40}
              max={240}
              value={bpm}
              onChange={(event) => {
                editedRef.current = true;
                setBpm(Number(event.target.value));
              }}
            />
          </label>
          <label className="field">
            <span>Key</span>
            <input
              className="field-input"
              type="text"
              value={musicalKey}
              onChange={(event) => {
                editedRef.current = true;
                setMusicalKey(event.target.value);
              }}
            />
          </label>
        </div>
        <div className="field-row">
          <label className="field">
            <span>Bars</span>
            <input
              className="field-input"
              type="number"
              min={1}
              max={16}
              value={bars}
              onChange={(event) => {
                editedRef.current = true;
                setBars(Number(event.target.value));
              }}
            />
          </label>
          <label className="field">
            <span>Beats / bar</span>
            <input
              className="field-input"
              type="number"
              min={1}
              max={12}
              value={beats}
              onChange={(event) => {
                editedRef.current = true;
                setBeats(Number(event.target.value));
              }}
            />
          </label>
          <label className="field">
            <span>Velocity {velocity}</span>
            <input
              className="field-input"
              type="range"
              min={1}
              max={100}
              value={velocity}
              onChange={(event) => {
                editedRef.current = true;
                setVelocity(Number(event.target.value));
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
        workflows={["instrument-sample", "loop"]}
        typeLabel="Sample"
        onCancel={cancelRuntimeOperation}
        onRetry={retryRuntimeOperation}
      >
        <button
          type="button"
          className="secondary-action"
          disabled={!selectedItemId}
          onClick={() => selectedItemId && previewLibraryItem(selectedItemId)}
          title="Play the saved loop"
        >
          <Play aria-hidden="true" size={16} />
          <span>Play latest</span>
        </button>
        <PlaybackControl playback={libraryPlayback} />
      </GenerationPanel>

      <div className="samples-layout">
        <div className="samples-main">
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
                  {statusLabel(provider.installStatus)} / {provider.sampleRateHz}{" "}
                  Hz /{" "}
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
            <li className={action.enabled ? "ready" : "warning"} key={action.id}>
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
