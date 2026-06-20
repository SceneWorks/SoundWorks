// DR-02 + UX-04: TTS studio. Rebuilt on the shared grammar; UX-04 adds an
// editable generation draft (prompt + voice + speed — the parameters the native
// Kokoro adapter actually consumes), a live progress panel driven by the UX-F1
// job poll, and inline playback of the saved result. Gating + the existing
// script/voice/provider panels are preserved.
import { useEffect, useRef, useState } from "react";
import { CircleCheck, Play, ShieldCheck } from "lucide-react";
import {
  GenerationPanel,
  HeroStat,
  MainSurface,
  ModelAvailabilityGate,
  PlaybackControl,
  SectionHeading,
  SurfaceHeader,
} from "../components";
import { formatDuration, statusLabel } from "../viewModel";
import { useAppContext } from "./context";

// Kokoro v0.19 voices that ship with the verified model cache. Uncached voices
// fail with an honest `tts.kokoro_cache_missing` error surfaced by the panel.
const KOKORO_VOICES = [
  "af_heart",
  "af_bella",
  "af_sarah",
  "am_adam",
  "am_michael",
  "bf_emma",
  "bm_george",
];

export function TtsScreen() {
  const {
    ttsStudio,
    ttsRuntimeModel,
    runRuntimeJob,
    runtimeOperation,
    cancelRuntimeOperation,
    retryRuntimeOperation,
    assetLibrary,
    libraryPlayback,
    previewLibraryItem,
    overview,
    setActiveView,
  } = useAppContext();

  const defaultPrompt = ttsStudio.script.segments
    .map((segment) => segment.text)
    .join(" ");
  const editedRef = useRef(false);
  const [prompt, setPrompt] = useState(defaultPrompt);
  const [voice, setVoice] = useState(KOKORO_VOICES[0]);
  const [speed, setSpeed] = useState(ttsStudio.controls.speed);

  // Re-seed from the overview when it loads after the fallback mount, until the
  // user starts editing (then their draft wins).
  useEffect(() => {
    if (!editedRef.current) {
      setPrompt(defaultPrompt);
      setSpeed(ttsStudio.controls.speed);
    }
  }, [defaultPrompt, ttsStudio.controls.speed]);

  const consentRecorded = ttsStudio.speakers.every(
    (speaker) => speaker.consentStatus === "explicit-consent-recorded",
  );
  const trimmed = prompt.trim();
  const blockReason = !ttsRuntimeModel
    ? "Install a TTS model to generate."
    : trimmed.length === 0
      ? "Enter script text to generate."
      : null;

  function generate() {
    runRuntimeJob("tts", prompt, {
      language: ttsStudio.script.language,
      speakerLabels: Array.from(
        new Set(ttsStudio.script.segments.map((s) => s.speakerLabel)),
      ),
      voiceProfileIds: ttsStudio.speakers.map((s) => s.voiceProfileId),
      voice,
      speed,
      seed: null,
      voiceConsentRecorded: consentRecorded,
    });
  }

  const selectedItemId = assetLibrary.selectedItem?.item.id;

  return (
    <section className="tts-studio-panel" aria-label="TTS Studio">
      <SurfaceHeader
        eyebrow="TTS Studio"
        title={ttsStudio.script.title}
        actions={
          <button
            className="primary-action"
            disabled={Boolean(blockReason)}
            onClick={generate}
            type="button"
            title={blockReason ?? "Queue TTS generation"}
          >
            <Play aria-hidden="true" size={18} />
            <span>{ttsRuntimeModel ? "Queue" : "Blocked"}</span>
          </button>
        }
        stats={
          <>
            <HeroStat label="Segments" value={overview.ttsStudio.segmentCount} />
            <HeroStat label="Voices" value={overview.ttsStudio.speakerCount} />
            <HeroStat
              label="Estimate"
              value={formatDuration(
                ttsStudio.generationPlan.estimatedTotalDurationMs,
              )}
            />
            <HeroStat
              label={ttsStudio.savedOutput.asset.kind}
              value={ttsStudio.savedOutput.version.file.format}
            />
          </>
        }
      />

      <ModelAvailabilityGate
        installed={Boolean(ttsRuntimeModel)}
        label="TTS"
        onOpenModelManager={() => setActiveView("models")}
      />

      <MainSurface className="studio-compose" ariaLabel="Compose speech">
        <SectionHeading title="Compose" eyebrow="script + voice" />
        <label className="field">
          <span>Script</span>
          <textarea
            className="field-input"
            rows={4}
            value={prompt}
            onChange={(event) => {
              editedRef.current = true;
              setPrompt(event.target.value);
            }}
            placeholder="Enter the text to speak…"
          />
          <small className="field-hint">{trimmed.length} characters</small>
        </label>
        <div className="field-row">
          <label className="field">
            <span>Voice</span>
            <select
              className="field-input"
              value={voice}
              onChange={(event) => {
                editedRef.current = true;
                setVoice(event.target.value);
              }}
            >
              {KOKORO_VOICES.map((option) => (
                <option key={option} value={option}>
                  {option}
                </option>
              ))}
            </select>
          </label>
          <label className="field">
            <span>Speed {speed.toFixed(2)}x</span>
            <input
              className="field-input"
              type="range"
              min={0.5}
              max={2}
              step={0.05}
              value={speed}
              onChange={(event) => {
                editedRef.current = true;
                setSpeed(Number(event.target.value));
              }}
            />
          </label>
        </div>
      </MainSurface>

      <GenerationPanel
        job={runtimeOperation}
        workflows={["tts"]}
        typeLabel="TTS"
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

      <div className="tts-layout">
        <div className="tts-script" aria-label="Script segments">
          {ttsStudio.script.segments.map((segment) => (
            <article className="segment-row" key={segment.id}>
              <span className="segment-index">{segment.position}</span>
              <div>
                <div className="segment-meta">
                  <strong>{segment.speakerLabel}</strong>
                  <span>{segment.sceneLabel}</span>
                  <small>{formatDuration(segment.targetDurationMs ?? 0)}</small>
                </div>
                <p>{segment.text}</p>
              </div>
            </article>
          ))}
        </div>

        <div className="tts-side">
          <MainSurface ariaLabel="Voice consent">
            <SectionHeading
              title="Voices"
              eyebrow={`${ttsStudio.voiceProfiles.length} profiles`}
            />
            <ol className="voice-list">
              {ttsStudio.speakers.map((speaker) => (
                <li key={speaker.voiceProfileId}>
                  <ShieldCheck aria-hidden="true" size={16} />
                  <div>
                    <strong>{speaker.label}</strong>
                    <small>
                      {speaker.language} / {statusLabel(speaker.consentStatus)}
                    </small>
                  </div>
                </li>
              ))}
            </ol>
          </MainSurface>

          <MainSurface ariaLabel="Provider limits">
            <SectionHeading
              title="Provider"
              eyebrow={`${ttsStudio.providerOptions.length} options`}
            />
            {ttsStudio.providerOptions.map((provider) => (
              <article className="provider-option" key={provider.modelId}>
                <strong>{provider.modelId}</strong>
                <small>
                  {statusLabel(provider.installStatus)} /{" "}
                  {statusLabel(provider.runtime)} / {provider.sampleRateHz} Hz
                </small>
                <ul>
                  {provider.limitations.map((limitation, index) => (
                    <li key={index}>{limitation}</li>
                  ))}
                </ul>
              </article>
            ))}
          </MainSurface>

          <MainSurface ariaLabel="Saved output">
            <SectionHeading
              title="Output"
              eyebrow={ttsStudio.submission.job.status}
            />
            <div className="output-card">
              <strong>{ttsStudio.savedOutput.asset.name}</strong>
              <small>{ttsStudio.savedOutput.asset.currentVersionId}</small>
              <p>{ttsStudio.savedOutput.version.file.storagePath}</p>
            </div>
          </MainSurface>
        </div>
      </div>

      <ol className="tts-checks" aria-label="TTS checks">
        {ttsStudio.validationChecks.map((check) => (
          <li key={check.id}>
            <CircleCheck aria-hidden="true" size={16} />
            <span>{check.summary}</span>
          </li>
        ))}
      </ol>
    </section>
  );
}
