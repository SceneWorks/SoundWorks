// DR-02 + UX-07: Song studio. UX-07 adds an editable authoring draft (title,
// prompt, lyrics, bpm, key, time signature, and an add/remove section list), a
// live progress panel driven by the UX-F1 poll, and inline playback of the saved
// result. No song-capable model ships today (the native music model declares only
// sample/loop), so generation is honestly gated by ModelAvailabilityGate — the
// authoring draft is real and feeds runRuntimeJob once a song model is installed.
import { useEffect, useRef, useState } from "react";
import { CircleCheck, Music2, Play, Save } from "lucide-react";
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

export function SongScreen() {
  const {
    songStudio,
    songRuntimeModel,
    songCandidateFocus,
    runRuntimeJob,
    runtimeOperation,
    cancelRuntimeOperation,
    retryRuntimeOperation,
    assetLibrary,
    libraryPlayback,
    previewLibraryItem,
    overview,
    openModelsFor,
  } = useAppContext();

  const editedRef = useRef(false);
  const [title, setTitle] = useState(songStudio.draft.title);
  const [prompt, setPrompt] = useState(songStudio.draft.prompt);
  const [lyrics, setLyrics] = useState(songStudio.draft.lyrics);
  const [bpm, setBpm] = useState(songStudio.controls.bpm);
  const [musicalKey, setMusicalKey] = useState(songStudio.controls.musicalKey);
  const [timeSignature, setTimeSignature] = useState(
    songStudio.controls.timeSignature,
  );
  const [sections, setSections] = useState<string[]>(
    songStudio.draft.sections.map((section) => section.label),
  );

  useEffect(() => {
    if (!editedRef.current) {
      setTitle(songStudio.draft.title);
      setPrompt(songStudio.draft.prompt);
      setLyrics(songStudio.draft.lyrics);
      setBpm(songStudio.controls.bpm);
      setMusicalKey(songStudio.controls.musicalKey);
      setTimeSignature(songStudio.controls.timeSignature);
      setSections(songStudio.draft.sections.map((section) => section.label));
    }
  }, [
    songStudio.draft.title,
    songStudio.draft.prompt,
    songStudio.draft.lyrics,
    songStudio.controls.bpm,
    songStudio.controls.musicalKey,
    songStudio.controls.timeSignature,
    songStudio.draft.sections,
  ]);

  const edit = () => {
    editedRef.current = true;
  };
  const trimmed = prompt.trim();
  const blockReason = !songRuntimeModel
    ? "Install a song model to generate."
    : trimmed.length === 0
      ? "Enter a song prompt to generate."
      : null;

  function generate() {
    runRuntimeJob("song", prompt, {
      title,
      lyrics,
      styleTags: songStudio.draft.styleTags,
      bpm,
      musicalKey,
      timeSignature,
      sectionLabels: sections.filter((label) => label.trim().length > 0),
    });
  }

  const selectedItemId = assetLibrary.selectedItem?.item.id;

  return (
    <section
      className="samples-studio-panel song-studio-panel"
      aria-label="Song Studio"
    >
      <SurfaceHeader
        eyebrow="Song Studio"
        title={title}
        actions={
          <button
            className="primary-action samples-action"
            disabled={Boolean(blockReason)}
            onClick={generate}
            type="button"
            title={blockReason ?? "Queue song generation"}
          >
            <Music2 aria-hidden="true" size={18} />
            <span>{songRuntimeModel ? "Generate" : "Blocked"}</span>
          </button>
        }
        stats={
          <>
            <HeroStat label="sections" value={sections.length} />
            <HeroStat
              label="saved"
              value={overview.songStudio.savedOutputCount}
            />
            <HeroStat label={musicalKey} value={bpm} />
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
        onOpenModelManager={() => openModelsFor("song")}
      />

      <MainSurface className="studio-compose" ariaLabel="Compose song">
        <SectionHeading title="Compose" eyebrow="structure + lyrics" />
        <label className="field">
          <span>Title</span>
          <input
            className="field-input"
            type="text"
            value={title}
            onChange={(event) => {
              edit();
              setTitle(event.target.value);
            }}
          />
        </label>
        <label className="field">
          <span>Prompt</span>
          <textarea
            className="field-input"
            rows={2}
            value={prompt}
            onChange={(event) => {
              edit();
              setPrompt(event.target.value);
            }}
            placeholder="Describe the song…"
          />
          <small className="field-hint">{trimmed.length} characters</small>
        </label>
        <label className="field">
          <span>Lyrics</span>
          <textarea
            className="field-input"
            rows={4}
            value={lyrics}
            onChange={(event) => {
              edit();
              setLyrics(event.target.value);
            }}
            placeholder="One line per lyric…"
          />
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
                edit();
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
                edit();
                setMusicalKey(event.target.value);
              }}
            />
          </label>
          <label className="field">
            <span>Time signature</span>
            <input
              className="field-input"
              type="text"
              value={timeSignature}
              onChange={(event) => {
                edit();
                setTimeSignature(event.target.value);
              }}
            />
          </label>
        </div>
        <div className="field">
          <span>Sections</span>
          <div className="editable-list">
            {sections.map((label, index) => (
              <div className="editable-row" key={index}>
                <input
                  className="field-input"
                  type="text"
                  value={label}
                  onChange={(event) => {
                    edit();
                    setSections((current) =>
                      current.map((value, position) =>
                        position === index ? event.target.value : value,
                      ),
                    );
                  }}
                />
                <button
                  type="button"
                  className="secondary-action"
                  onClick={() => {
                    edit();
                    setSections((current) =>
                      current.filter((_, position) => position !== index),
                    );
                  }}
                  title="Remove section"
                >
                  Remove
                </button>
              </div>
            ))}
            <button
              type="button"
              className="secondary-action"
              onClick={() => {
                edit();
                setSections((current) => [...current, "New section"]);
              }}
            >
              Add section
            </button>
          </div>
        </div>
      </MainSurface>

      <GenerationPanel
        job={runtimeOperation}
        workflows={["song"]}
        typeLabel="Song"
        onCancel={cancelRuntimeOperation}
        onRetry={retryRuntimeOperation}
      >
        <button
          type="button"
          className="secondary-action"
          disabled={!selectedItemId}
          onClick={() => selectedItemId && previewLibraryItem(selectedItemId)}
          title="Play the saved song"
        >
          <Play aria-hidden="true" size={16} />
          <span>Play latest</span>
        </button>
        <PlaybackControl playback={libraryPlayback} />
      </GenerationPanel>

      <div className="samples-layout">
        <div className="samples-main">
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
                <p>{provider.supportedControls.map(statusLabel).join(" / ")}</p>
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
