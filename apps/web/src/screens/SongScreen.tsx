// DR-02: Song studio. Extracted from App.tsx and rebuilt on the shared grammar
// (SurfaceHeader hero + HeroStat for the header/metrics, MainSurface +
// SectionHeading for the side/prompt panels) in place of the bespoke
// samples-header / samples-metrics / sfx-prompt-panel / tts-subpanel /
// subpanel-heading classes. Follows the TtsScreen template. All Generate
// wiring, gating, and data bindings are preserved verbatim.
import { CircleCheck, Music2, Save } from "lucide-react";
import {
  HeroStat,
  MainSurface,
  SectionHeading,
  SurfaceHeader,
  ModelAvailabilityGate,
} from "../components";
import { formatDuration, statusLabel, workflowLabel } from "../viewModel";
import { useAppContext } from "./context";

export function SongScreen() {
  const {
    songStudio,
    songRuntimeModel,
    songCandidateFocus,
    runRuntimeJob,
    overview,
    setActiveView,
  } = useAppContext();

  return (
    <section
      className="samples-studio-panel song-studio-panel"
      aria-label="Song Studio"
    >
      <SurfaceHeader
        eyebrow="Song Studio"
        title={songStudio.draft.title}
        actions={
          <button
            className="primary-action samples-action"
            disabled={!songRuntimeModel}
            onClick={() =>
              runRuntimeJob("song", songStudio.draft.prompt, {
                title: songStudio.draft.title,
                lyrics: songStudio.draft.lyrics,
                styleTags: songStudio.draft.styleTags,
                bpm: songStudio.controls.bpm,
                musicalKey: songStudio.controls.musicalKey,
                sectionLabels: songStudio.draft.sections.map(
                  (section) => section.label,
                ),
              })
            }
            type="button"
            title="Queue song generation"
          >
            <Music2 aria-hidden="true" size={18} />
            <span>{songRuntimeModel ? "Generate" : "Blocked"}</span>
          </button>
        }
        stats={
          <>
            <HeroStat
              label="sections"
              value={overview.songStudio.sectionCount}
            />
            <HeroStat
              label="saved"
              value={overview.songStudio.savedOutputCount}
            />
            <HeroStat
              label={songStudio.controls.musicalKey}
              value={songStudio.controls.bpm}
            />
            <HeroStat
              label="scorecards"
              value={overview.songStudio.scorecardCount}
            />
          </>
        }
      />

      <ModelAvailabilityGate
        installed={Boolean(songRuntimeModel)}
        label="song"
        onOpenModelManager={() => setActiveView("models")}
      />

      <div className="samples-layout">
        <div className="samples-main">
          <MainSurface ariaLabel="Song prompt">
            <SectionHeading
              title={songStudio.draft.language}
              eyebrow={`${songStudio.draft.styleTags.length}`}
            />
            <p>{songStudio.draft.prompt}</p>
            <small>{songStudio.draft.singerHint}</small>
            <div className="candidate-strip">
              {songStudio.draft.styleTags.map((tag, index) => (
                <span key={index}>{tag}</span>
              ))}
            </div>
          </MainSurface>

          <div className="samples-control-grid" aria-label="Song controls">
            <div>
              <strong>{songStudio.arrangement.totalBars} bars</strong>
              <span>{songStudio.controls.timeSignature}</span>
            </div>
            <div>
              <strong>
                {formatDuration(songStudio.arrangement.estimatedDurationMs)}
              </strong>
              <span>arranged</span>
            </div>
            <div>
              <strong>{songStudio.controls.variationCount}</strong>
              <span>variants</span>
            </div>
            <div>
              <strong>{songStudio.controls.requestedStems.length}</strong>
              <span>stems</span>
            </div>
          </div>

          <div className="samples-variant-grid" aria-label="Song sections">
            {songStudio.arrangement.sections.map((section) => (
              <article className="samples-variant selected" key={section.id}>
                <div className="sfx-variant-title">
                  <strong>{section.label}</strong>
                  <span>{section.bars} bars</span>
                </div>
                <small>
                  starts bar {section.startBar + 1} /{" "}
                  {section.hasLyrics ? "lyrics" : "instrumental"}
                </small>
                <p>{section.locked ? "locked" : "regeneratable"}</p>
              </article>
            ))}
          </div>

          <div className="samples-variant-grid" aria-label="Song variants">
            {songStudio.variants.map((variant) => (
              <article
                className={
                  variant.selectedForSave
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
                  {formatDuration(variant.durationMs)} / {variant.bpm} BPM /{" "}
                  {variant.musicalKey}
                </small>
                <p>
                  lyric {variant.lyricAlignmentScore} / structure{" "}
                  {variant.structureMatchScore}
                </p>
                <div className="candidate-strip">
                  {variant.stemKinds.slice(0, 5).map((stem, index) => (
                    <span key={index}>{statusLabel(stem)}</span>
                  ))}
                </div>
              </article>
            ))}
          </div>
        </div>

        <div className="samples-side">
          <MainSurface ariaLabel="Song providers">
            <SectionHeading
              title="Providers"
              eyebrow={`${songStudio.providerOptions.length}`}
            />
            {songStudio.providerOptions.map((provider) => (
              <article
                className="sfx-provider-option"
                key={`${provider.workflow}-${provider.modelId}`}
              >
                <strong>{provider.modelId}</strong>
                <small>
                  {statusLabel(provider.installStatus)} /{" "}
                  {provider.supportsStems ? "stems" : "mixdown"} /{" "}
                  {provider.sampleRateHz} Hz
                </small>
                <p>
                  {provider.supportedControls.map(statusLabel).join(" / ")}
                </p>
              </article>
            ))}
          </MainSurface>

          <MainSurface ariaLabel="Song provider scorecards">
            <SectionHeading
              title="Scorecards"
              eyebrow={`${songStudio.providerScorecards.length}`}
            />
            <div className="voice-provider-list">
              {songCandidateFocus.map((scorecard) => (
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

          <MainSurface ariaLabel="Song outputs">
            <SectionHeading
              title="Outputs"
              eyebrow={songStudio.submission.job.status}
            />
            {songStudio.savedOutputs.map((output) => (
              <div className="output-card" key={output.variantId}>
                <strong>{output.asset.name}</strong>
                <small>
                  {output.asset.kind} /{" "}
                  {output.version.technical.bpm
                    ? `${output.version.technical.bpm} BPM`
                    : output.asset.currentVersionId}
                </small>
                <p>{output.version.file.storagePath}</p>
              </div>
            ))}
          </MainSurface>
        </div>
      </div>

      <div className="samples-review-grid">
        <ol className="voice-checks" aria-label="Song export targets">
          {songStudio.exportTargets.map((target) => (
            <li className="ready" key={target.id}>
              <Save aria-hidden="true" size={16} />
              <span>{target.summary}</span>
            </li>
          ))}
        </ol>
        <ol className="voice-checks" aria-label="Song QA checks">
          {songStudio.qaChecks.map((check) => (
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
