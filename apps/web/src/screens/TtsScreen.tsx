// DR-02: TTS studio. Extracted from App.tsx and rebuilt on the shared grammar
// (SurfaceHeader hero + HeroStat for the header/metrics, MainSurface +
// SectionHeading for the side panels) in place of the bespoke
// tts-studio-panel / tts-header / tts-metrics / tts-subpanel / subpanel-heading
// classes. This is the template the other studios follow. All Generate wiring,
// gating, and data bindings are preserved verbatim.
import { CircleCheck, Play, ShieldCheck } from "lucide-react";
import {
  HeroStat,
  MainSurface,
  ModelAvailabilityGate,
  SectionHeading,
  SurfaceHeader,
} from "../components";
import { formatDuration, statusLabel } from "../viewModel";
import { useAppContext } from "./context";

export function TtsScreen() {
  const {
    ttsStudio,
    ttsRuntimeModel,
    runRuntimeJob,
    runtimeOperation,
    overview,
    setActiveView,
  } = useAppContext();

  return (
    <section className="tts-studio-panel" aria-label="TTS Studio">
      <SurfaceHeader
        eyebrow="TTS Studio"
        title={ttsStudio.script.title}
        actions={
          <button
            className="primary-action"
            disabled={!ttsRuntimeModel}
            onClick={() =>
              runRuntimeJob(
                "tts",
                ttsStudio.script.segments
                  .map((segment) => segment.text)
                  .join(" "),
                {
                  language: ttsStudio.script.language,
                  speakerLabels: Array.from(
                    new Set(
                      ttsStudio.script.segments.map(
                        (segment) => segment.speakerLabel,
                      ),
                    ),
                  ),
                  voiceProfileIds: ttsStudio.speakers.map(
                    (speaker) => speaker.voiceProfileId,
                  ),
                  voice: "af_heart",
                  speed: ttsStudio.controls.speed,
                  seed: null,
                  voiceConsentRecorded: ttsStudio.speakers.every(
                    (speaker) =>
                      speaker.consentStatus === "explicit-consent-recorded",
                  ),
                },
              )
            }
            type="button"
            title="Queue TTS generation"
          >
            <Play aria-hidden="true" size={18} />
            <span>{ttsRuntimeModel ? "Queue" : "Blocked"}</span>
          </button>
        }
        stats={
          <>
            <HeroStat
              label="Segments"
              value={overview.ttsStudio.segmentCount}
            />
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
              eyebrow={
                runtimeOperation?.status ?? ttsStudio.submission.job.status
              }
            />
            <div className="output-card">
              <strong>
                {runtimeOperation?.workflow === "tts"
                  ? runtimeOperation.id
                  : ttsStudio.savedOutput.asset.name}
              </strong>
              <small>
                {runtimeOperation?.workflow === "tts"
                  ? `${statusLabel(runtimeOperation.status)} / ${runtimeOperation.adapter}`
                  : ttsStudio.savedOutput.asset.currentVersionId}
              </small>
              <p>
                {runtimeOperation?.workflow === "tts"
                  ? runtimeOperation.recordRoot
                  : ttsStudio.savedOutput.version.file.storagePath}
              </p>
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
