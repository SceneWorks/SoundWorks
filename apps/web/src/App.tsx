import {
  Activity,
  Boxes,
  CircleAlert,
  CircleCheck,
  ClipboardCheck,
  Cpu,
  Disc3,
  Gauge,
  HardDrive,
  Library,
  Mic2,
  Music2,
  PackageCheck,
  Play,
  Radio,
  Save,
  ShieldCheck,
  SlidersHorizontal,
  Sparkles,
  Waves,
} from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import {
  fallbackOverview,
  fallbackReviewWorkspace,
  fallbackRuntime,
  fallbackSamplesStudio,
  fallbackSongStudio,
  fallbackSfxStudio,
  fallbackTtsStudio,
  fallbackVoiceLab,
} from "./appData";
import {
  loadAppOverview,
  loadReviewWorkspaceOverview,
  loadRuntimeOverview,
  loadSamplesStudioOverview,
  loadSongStudioOverview,
  loadSfxStudioOverview,
  loadTtsStudioOverview,
  loadVoiceLabOverview,
} from "./tauri";
import type {
  AppOverview,
  ReviewWorkspaceOverview,
  RuntimeOverview,
  SamplesStudioOverview,
  SongStudioOverview,
  SfxStudioOverview,
  TtsStudioOverview,
  VoiceLabOverview,
} from "./types";

const navItems = [
  { label: "Studios", icon: Sparkles },
  { label: "Library", icon: Library },
  { label: "Mixer", icon: SlidersHorizontal },
  { label: "Jobs", icon: Activity },
];

const studioIcons = [Mic2, Radio, Waves, Boxes, Music2, ClipboardCheck, Sparkles];

function workflowLabel(workflow: string) {
  return workflow
    .split("-")
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
}

function statusLabel(status: string) {
  return status
    .split("-")
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
}

function formatMb(value?: number | null) {
  if (!value) {
    return "n/a";
  }

  return value >= 1024 ? `${Math.round(value / 1024)} GB` : `${value} MB`;
}

function formatDuration(ms: number) {
  const seconds = Math.round(ms / 100) / 10;

  return `${seconds}s`;
}

function countFor(counts: Record<string, number>, key: string) {
  return counts[key] ?? 0;
}

export function App() {
  const [overview, setOverview] = useState<AppOverview>(fallbackOverview);
  const [runtime, setRuntime] = useState<RuntimeOverview>(fallbackRuntime);
  const [ttsStudio, setTtsStudio] =
    useState<TtsStudioOverview>(fallbackTtsStudio);
  const [voiceLab, setVoiceLab] =
    useState<VoiceLabOverview>(fallbackVoiceLab);
  const [sfxStudio, setSfxStudio] =
    useState<SfxStudioOverview>(fallbackSfxStudio);
  const [samplesStudio, setSamplesStudio] = useState<SamplesStudioOverview>(
    fallbackSamplesStudio,
  );
  const [songStudio, setSongStudio] =
    useState<SongStudioOverview>(fallbackSongStudio);
  const [reviewWorkspace, setReviewWorkspace] =
    useState<ReviewWorkspaceOverview>(fallbackReviewWorkspace);

  useEffect(() => {
    let active = true;

    loadAppOverview().then((nextOverview) => {
      if (active) {
        setOverview(nextOverview);
      }
    });

    loadRuntimeOverview().then((nextRuntime) => {
      if (active) {
        setRuntime(nextRuntime);
      }
    });

    loadTtsStudioOverview().then((nextTtsStudio) => {
      if (active) {
        setTtsStudio(nextTtsStudio);
      }
    });

    loadVoiceLabOverview().then((nextVoiceLab) => {
      if (active) {
        setVoiceLab(nextVoiceLab);
      }
    });

    loadSfxStudioOverview().then((nextSfxStudio) => {
      if (active) {
        setSfxStudio(nextSfxStudio);
      }
    });

    loadSamplesStudioOverview().then((nextSamplesStudio) => {
      if (active) {
        setSamplesStudio(nextSamplesStudio);
      }
    });

    loadSongStudioOverview().then((nextSongStudio) => {
      if (active) {
        setSongStudio(nextSongStudio);
      }
    });

    loadReviewWorkspaceOverview().then((nextReviewWorkspace) => {
      if (active) {
        setReviewWorkspace(nextReviewWorkspace);
      }
    });

    return () => {
      active = false;
    };
  }, []);

  const scaffoldedLayerCount = useMemo(
    () =>
      overview.architecture.layers.filter(
        (layer) => layer.status === "scaffolded",
      ).length,
    [overview.architecture.layers],
  );
  const voiceCandidateFocus = useMemo(
    () =>
      voiceLab.providerScorecards.filter((scorecard) =>
        ["chatterbox", "rvc", "chatterbox-turbo"].includes(
          scorecard.candidateId,
        ),
      ),
    [voiceLab.providerScorecards],
  );
  const sfxCandidateFocus = useMemo(
    () =>
      sfxStudio.providerScorecards.filter((scorecard) =>
        ["moss-soundeffect", "stable-audio-open-1", "mmaudio"].includes(
          scorecard.candidateId,
        ),
      ),
    [sfxStudio.providerScorecards],
  );
  const samplesCandidateFocus = useMemo(
    () =>
      samplesStudio.providerScorecards.filter((scorecard) =>
        ["ace-step-1-5", "stable-audio-open-1", "heartmula"].includes(
          scorecard.candidateId,
        ),
      ),
    [samplesStudio.providerScorecards],
  );
  const songCandidateFocus = useMemo(
    () =>
      songStudio.providerScorecards.filter((scorecard) =>
        ["ace-step-1-5", "stable-audio-3", "diffrhythm-2"].includes(
          scorecard.candidateId,
        ),
      ),
    [songStudio.providerScorecards],
  );

  return (
    <main className="app-shell">
      <aside className="sidebar" aria-label="Primary">
        <div className="brand-mark" aria-label={overview.productName}>
          <Waves aria-hidden="true" size={28} />
          <span>{overview.productName}</span>
        </div>

        <nav className="nav-list">
          {navItems.map((item) => (
            <button
              className="nav-button"
              key={item.label}
              type="button"
              title={item.label}
            >
              <item.icon aria-hidden="true" size={18} />
              <span>{item.label}</span>
            </button>
          ))}
        </nav>
      </aside>

      <section className="workspace" aria-label="Workspace">
        <header className="workspace-header">
          <div>
            <p className="eyebrow">Local workspace</p>
            <h1>{overview.productName}</h1>
          </div>
          <div className="status-strip" aria-label="Scaffold status">
            <strong>{scaffoldedLayerCount}</strong>
            <span>active layers</span>
          </div>
        </header>

        <section className="studio-grid" aria-label="Studios">
          {overview.studios.map((studio, index) => {
            const Icon = studioIcons[index % studioIcons.length];

            return (
              <button className="studio-card" key={studio.id} type="button">
                <span className="icon-badge">
                  <Icon aria-hidden="true" size={22} />
                </span>
                <span className="studio-copy">
                  <strong>{studio.name}</strong>
                  <small>{studio.route}</small>
                </span>
                <span className={`state-pill ${studio.status}`}>
                  {studio.status}
                </span>
              </button>
            );
          })}
        </section>

        <section className="tts-studio-panel" aria-label="TTS Studio">
          <div className="tts-header">
            <div>
              <p className="eyebrow">TTS Studio</p>
              <h2>{ttsStudio.script.title}</h2>
            </div>
            <button
              className="primary-action"
              disabled={!ttsStudio.submission.canSubmit}
              type="button"
              title="Queue TTS generation"
            >
              <Play aria-hidden="true" size={18} />
              <span>
                {ttsStudio.submission.canSubmit ? "Queue" : "Blocked"}
              </span>
            </button>
          </div>

          <div className="tts-metrics" aria-label="TTS workflow status">
            <div>
              <Mic2 aria-hidden="true" size={18} />
              <strong>{overview.ttsStudio.segmentCount}</strong>
              <span>segments</span>
            </div>
            <div>
              <ShieldCheck aria-hidden="true" size={18} />
              <strong>{overview.ttsStudio.speakerCount}</strong>
              <span>voices</span>
            </div>
            <div>
              <Gauge aria-hidden="true" size={18} />
              <strong>
                {formatDuration(
                  ttsStudio.generationPlan.estimatedTotalDurationMs,
                )}
              </strong>
              <span>estimate</span>
            </div>
            <div>
              <Save aria-hidden="true" size={18} />
              <strong>{ttsStudio.savedOutput.version.file.format}</strong>
              <span>{ttsStudio.savedOutput.asset.kind}</span>
            </div>
          </div>

          <div className="tts-layout">
            <div className="tts-script" aria-label="Script segments">
              {ttsStudio.script.segments.map((segment) => (
                <article className="segment-row" key={segment.id}>
                  <span className="segment-index">{segment.position}</span>
                  <div>
                    <div className="segment-meta">
                      <strong>{segment.speakerLabel}</strong>
                      <span>{segment.sceneLabel}</span>
                      <small>
                        {formatDuration(segment.targetDurationMs ?? 0)}
                      </small>
                    </div>
                    <p>{segment.text}</p>
                  </div>
                </article>
              ))}
            </div>

            <div className="tts-side">
              <section className="tts-subpanel" aria-label="Voice consent">
                <div className="subpanel-heading">
                  <h3>Voices</h3>
                  <span>{ttsStudio.voiceProfiles.length}</span>
                </div>
                <ol className="voice-list">
                  {ttsStudio.speakers.map((speaker) => (
                    <li key={speaker.voiceProfileId}>
                      <ShieldCheck aria-hidden="true" size={16} />
                      <div>
                        <strong>{speaker.label}</strong>
                        <small>
                          {speaker.language} /{" "}
                          {statusLabel(speaker.consentStatus)}
                        </small>
                      </div>
                    </li>
                  ))}
                </ol>
              </section>

              <section className="tts-subpanel" aria-label="Provider limits">
                <div className="subpanel-heading">
                  <h3>Provider</h3>
                  <span>{ttsStudio.providerOptions.length}</span>
                </div>
                {ttsStudio.providerOptions.map((provider) => (
                  <article className="provider-option" key={provider.modelId}>
                    <strong>{provider.modelId}</strong>
                    <small>
                      {statusLabel(provider.installStatus)} /{" "}
                      {statusLabel(provider.runtime)} / {provider.sampleRateHz}{" "}
                      Hz
                    </small>
                    <ul>
                      {provider.limitations.map((limitation) => (
                        <li key={limitation}>{limitation}</li>
                      ))}
                    </ul>
                  </article>
                ))}
              </section>

              <section className="tts-subpanel" aria-label="Saved output">
                <div className="subpanel-heading">
                  <h3>Output</h3>
                  <span>{ttsStudio.submission.job.status}</span>
                </div>
                <div className="output-card">
                  <strong>{ttsStudio.savedOutput.asset.name}</strong>
                  <small>{ttsStudio.savedOutput.asset.currentVersionId}</small>
                  <p>{ttsStudio.savedOutput.version.file.storagePath}</p>
                </div>
              </section>
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

        <section className="voice-lab-panel" aria-label="Voice Lab">
          <div className="voice-lab-header">
            <div>
              <p className="eyebrow">Voice Lab</p>
              <h2>Consented voice workflows</h2>
            </div>
            <button
              className="primary-action voice-action"
              disabled={!voiceLab.selectedConversion.canSubmit}
              type="button"
              title="Queue voice conversion"
            >
              <Radio aria-hidden="true" size={18} />
              <span>
                {voiceLab.selectedConversion.canSubmit ? "Convert" : "Blocked"}
              </span>
            </button>
          </div>

          <div className="voice-lab-metrics" aria-label="Voice Lab status">
            <div>
              <Mic2 aria-hidden="true" size={18} />
              <strong>{overview.voiceLab.modeCount}</strong>
              <span>modes</span>
            </div>
            <div>
              <ShieldCheck aria-hidden="true" size={18} />
              <strong>{overview.voiceLab.profileCount}</strong>
              <span>profiles</span>
            </div>
            <div>
              <ClipboardCheck aria-hidden="true" size={18} />
              <strong>{overview.voiceLab.providerCount}</strong>
              <span>scorecards</span>
            </div>
            <div>
              <Save aria-hidden="true" size={18} />
              <strong>{overview.voiceLab.savedAssetKind}</strong>
              <span>{overview.voiceLab.selectedConversionCandidateId}</span>
            </div>
          </div>

          <div className="voice-mode-grid" aria-label="Voice modes">
            {voiceLab.modes.map((mode) => (
              <article className="voice-mode-card" key={mode.mode}>
                <div className="voice-mode-title">
                  <strong>{mode.label}</strong>
                  <span>{workflowLabel(mode.workflow)}</span>
                </div>
                <small>
                  {mode.inputAssetKinds.map(statusLabel).join(" / ")} to{" "}
                  {statusLabel(mode.outputAssetKind)}
                </small>
                <div className="candidate-strip">
                  {mode.providerCandidateIds.slice(0, 4).map((candidateId) => (
                    <span key={candidateId}>{candidateId}</span>
                  ))}
                </div>
              </article>
            ))}
          </div>

          <div className="voice-lab-layout">
            <div className="voice-profile-column" aria-label="Voice profiles">
              {voiceLab.voiceProfiles.map((profile) => (
                <article className="voice-profile-card" key={profile.profile.id}>
                  <div className="voice-profile-topline">
                    <div>
                      <strong>{profile.profile.displayName}</strong>
                      <small>
                        {profile.language} / {statusLabel(profile.profile.consent)}
                      </small>
                    </div>
                    <span
                      className={
                        profile.commercialUseAllowed
                          ? "voice-approval approved"
                          : "voice-approval review"
                      }
                    >
                      {profile.commercialUseAllowed ? "commercial" : "review"}
                    </span>
                  </div>
                  <p>{profile.safetySummary}</p>
                  <ol className="mode-readiness">
                    {profile.modeReadiness.map((readiness) => (
                      <li key={readiness.mode}>
                        <span
                          className={
                            readiness.ready
                              ? "readiness-dot ready"
                              : "readiness-dot blocked"
                          }
                        />
                        <span>{statusLabel(readiness.mode)}</span>
                        <small>{readiness.reason ?? "ready"}</small>
                      </li>
                    ))}
                  </ol>
                </article>
              ))}
            </div>

            <div className="voice-side">
              <section className="tts-subpanel" aria-label="Conversion source">
                <div className="subpanel-heading">
                  <h3>Conversion</h3>
                  <span>{voiceLab.selectedConversion.job.status}</span>
                </div>
                <div className="conversion-source">
                  <strong>{voiceLab.conversionSource.name}</strong>
                  <small>
                    {statusLabel(voiceLab.conversionSource.kind)} /{" "}
                    {formatDuration(voiceLab.conversionSource.durationMs)}
                  </small>
                  <p>{voiceLab.conversionSource.assetId}</p>
                </div>
                <div className="output-card">
                  <strong>{voiceLab.savedOutput.asset.name}</strong>
                  <small>{voiceLab.savedOutput.asset.currentVersionId}</small>
                  <p>{voiceLab.savedOutput.version.file.storagePath}</p>
                </div>
              </section>

              <section className="tts-subpanel" aria-label="Voice providers">
                <div className="subpanel-heading">
                  <h3>Providers</h3>
                  <span>{voiceLab.providerScorecards.length}</span>
                </div>
                <div className="voice-provider-list">
                  {voiceCandidateFocus.map((scorecard) => (
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
              </section>
            </div>
          </div>

          <div className="voice-review-grid">
            <ol className="voice-checks" aria-label="Voice safety gates">
              {voiceLab.safetyGates.map((gate) => (
                <li className={gate.status} key={gate.id}>
                  <ShieldCheck aria-hidden="true" size={16} />
                  <span>{gate.summary}</span>
                </li>
              ))}
            </ol>
            <ol className="voice-checks" aria-label="Voice QA checks">
              {voiceLab.qaChecks.map((check) => (
                <li className={check.status} key={check.id}>
                  <ClipboardCheck aria-hidden="true" size={16} />
                  <span>
                    <strong>{check.label}</strong> {check.target}
                  </span>
                </li>
              ))}
            </ol>
          </div>
        </section>

        <section className="sfx-studio-panel" aria-label="SFX and Ambience">
          <div className="sfx-header">
            <div>
              <p className="eyebrow">SFX + Ambience</p>
              <h2>{statusLabel(sfxStudio.prompt.category)}</h2>
            </div>
            <button
              className="primary-action sfx-action"
              disabled={!sfxStudio.submission.canSubmit}
              type="button"
              title="Queue SFX generation"
            >
              <Play aria-hidden="true" size={18} />
              <span>{sfxStudio.submission.canSubmit ? "Generate" : "Blocked"}</span>
            </button>
          </div>

          <div className="sfx-metrics" aria-label="SFX workflow status">
            <div>
              <Waves aria-hidden="true" size={18} />
              <strong>{overview.sfxStudio.variantCount}</strong>
              <span>variants</span>
            </div>
            <div>
              <Save aria-hidden="true" size={18} />
              <strong>{overview.sfxStudio.savedOutputCount}</strong>
              <span>saved</span>
            </div>
            <div>
              <ClipboardCheck aria-hidden="true" size={18} />
              <strong>{overview.sfxStudio.scorecardCount}</strong>
              <span>scorecards</span>
            </div>
            <div>
              <Gauge aria-hidden="true" size={18} />
              <strong>{formatDuration(sfxStudio.controls.durationMs)}</strong>
              <span>{sfxStudio.controls.loopable ? "loopable" : "one-shot"}</span>
            </div>
          </div>

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
                  {sfxStudio.prompt.tags.map((tag) => (
                    <span key={tag}>{tag}</span>
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
                      {formatDuration(variant.durationMs)} / {variant.loudnessLufs} LUFS /{" "}
                      {variant.truePeakDbfs} dBTP
                    </small>
                    <p>
                      {variant.loopPoints
                        ? `loop ${variant.loopPoints.startSample}-${variant.loopPoints.endSample}`
                        : "one-shot preview"}
                    </p>
                    <div className="candidate-strip">
                      {variant.tags.slice(0, 4).map((tag) => (
                        <span key={tag}>{tag}</span>
                      ))}
                    </div>
                  </article>
                ))}
              </div>
            </div>

            <div className="sfx-side">
              <section className="tts-subpanel" aria-label="SFX provider options">
                <div className="subpanel-heading">
                  <h3>Providers</h3>
                  <span>{sfxStudio.providerOptions.length}</span>
                </div>
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
              </section>

              <section className="tts-subpanel" aria-label="SFX provider scorecards">
                <div className="subpanel-heading">
                  <h3>Scorecards</h3>
                  <span>{sfxStudio.providerScorecards.length}</span>
                </div>
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
                      </div>
                      {scorecard.recommended ? <span>pick</span> : null}
                    </article>
                  ))}
                </div>
              </section>

              <section className="tts-subpanel" aria-label="Saved SFX outputs">
                <div className="subpanel-heading">
                  <h3>Outputs</h3>
                  <span>{sfxStudio.submission.job.status}</span>
                </div>
                {sfxStudio.savedOutputs.map((output) => (
                  <div className="output-card" key={output.variantId}>
                    <strong>{output.asset.name}</strong>
                    <small>
                      {output.asset.kind} / {output.asset.currentVersionId}
                    </small>
                    <p>{output.version.file.storagePath}</p>
                  </div>
                ))}
              </section>
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

        <section className="samples-studio-panel" aria-label="Samples and Loops">
          <div className="samples-header">
            <div>
              <p className="eyebrow">Samples + Loops</p>
              <h2>{samplesStudio.pack.name}</h2>
            </div>
            <button
              className="primary-action samples-action"
              disabled={!samplesStudio.submission.canSubmit}
              type="button"
              title="Queue sample and loop generation"
            >
              <Disc3 aria-hidden="true" size={18} />
              <span>
                {samplesStudio.submission.canSubmit ? "Generate" : "Blocked"}
              </span>
            </button>
          </div>

          <div className="samples-metrics" aria-label="Samples workflow status">
            <div>
              <Boxes aria-hidden="true" size={18} />
              <strong>{overview.samplesStudio.variantCount}</strong>
              <span>variants</span>
            </div>
            <div>
              <Save aria-hidden="true" size={18} />
              <strong>{overview.samplesStudio.savedOutputCount}</strong>
              <span>saved</span>
            </div>
            <div>
              <Gauge aria-hidden="true" size={18} />
              <strong>{samplesStudio.controls.bpm}</strong>
              <span>{samplesStudio.controls.musicalKey}</span>
            </div>
            <div>
              <ClipboardCheck aria-hidden="true" size={18} />
              <strong>{overview.samplesStudio.scorecardCount}</strong>
              <span>scorecards</span>
            </div>
          </div>

          <div className="samples-layout">
            <div className="samples-main">
              <section className="sfx-prompt-panel" aria-label="Sample prompt">
                <div className="subpanel-heading">
                  <h3>{statusLabel(samplesStudio.prompt.instrumentFamily)}</h3>
                  <span>{samplesStudio.prompt.genreTags.length}</span>
                </div>
                <p>{samplesStudio.prompt.text}</p>
                <small>{samplesStudio.prompt.negativePrompt}</small>
                <div className="candidate-strip">
                  {samplesStudio.prompt.genreTags.map((tag) => (
                    <span key={tag}>{tag}</span>
                  ))}
                </div>
              </section>

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
                      {variant.tags.slice(0, 4).map((tag) => (
                        <span key={tag}>{tag}</span>
                      ))}
                    </div>
                  </article>
                ))}
              </div>
            </div>

            <div className="samples-side">
              <section className="tts-subpanel" aria-label="Sample providers">
                <div className="subpanel-heading">
                  <h3>Providers</h3>
                  <span>{samplesStudio.providerOptions.length}</span>
                </div>
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
              </section>

              <section className="tts-subpanel" aria-label="Sample provider scorecards">
                <div className="subpanel-heading">
                  <h3>Scorecards</h3>
                  <span>{samplesStudio.providerScorecards.length}</span>
                </div>
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
              </section>

              <section className="tts-subpanel" aria-label="Sample pack outputs">
                <div className="subpanel-heading">
                  <h3>Pack</h3>
                  <span>{samplesStudio.pack.exportFormats.join(" / ")}</span>
                </div>
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
              </section>
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

        <section className="samples-studio-panel song-studio-panel" aria-label="Song Studio">
          <div className="samples-header">
            <div>
              <p className="eyebrow">Song Studio</p>
              <h2>{songStudio.draft.title}</h2>
            </div>
            <button
              className="primary-action samples-action"
              disabled={!songStudio.submission.canSubmit}
              type="button"
              title="Queue song generation"
            >
              <Music2 aria-hidden="true" size={18} />
              <span>
                {songStudio.submission.canSubmit ? "Generate" : "Blocked"}
              </span>
            </button>
          </div>

          <div className="samples-metrics" aria-label="Song workflow status">
            <div>
              <Music2 aria-hidden="true" size={18} />
              <strong>{overview.songStudio.sectionCount}</strong>
              <span>sections</span>
            </div>
            <div>
              <Save aria-hidden="true" size={18} />
              <strong>{overview.songStudio.savedOutputCount}</strong>
              <span>saved</span>
            </div>
            <div>
              <Gauge aria-hidden="true" size={18} />
              <strong>{songStudio.controls.bpm}</strong>
              <span>{songStudio.controls.musicalKey}</span>
            </div>
            <div>
              <ClipboardCheck aria-hidden="true" size={18} />
              <strong>{overview.songStudio.scorecardCount}</strong>
              <span>scorecards</span>
            </div>
          </div>

          <div className="samples-layout">
            <div className="samples-main">
              <section className="sfx-prompt-panel" aria-label="Song prompt">
                <div className="subpanel-heading">
                  <h3>{songStudio.draft.language}</h3>
                  <span>{songStudio.draft.styleTags.length}</span>
                </div>
                <p>{songStudio.draft.prompt}</p>
                <small>{songStudio.draft.singerHint}</small>
                <div className="candidate-strip">
                  {songStudio.draft.styleTags.map((tag) => (
                    <span key={tag}>{tag}</span>
                  ))}
                </div>
              </section>

              <div className="samples-control-grid" aria-label="Song controls">
                <div>
                  <strong>{songStudio.arrangement.totalBars} bars</strong>
                  <span>{songStudio.controls.timeSignature}</span>
                </div>
                <div>
                  <strong>{formatDuration(songStudio.arrangement.estimatedDurationMs)}</strong>
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
                      {variant.stemKinds.slice(0, 5).map((stem) => (
                        <span key={stem}>{statusLabel(stem)}</span>
                      ))}
                    </div>
                  </article>
                ))}
              </div>
            </div>

            <div className="samples-side">
              <section className="tts-subpanel" aria-label="Song providers">
                <div className="subpanel-heading">
                  <h3>Providers</h3>
                  <span>{songStudio.providerOptions.length}</span>
                </div>
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
              </section>

              <section className="tts-subpanel" aria-label="Song provider scorecards">
                <div className="subpanel-heading">
                  <h3>Scorecards</h3>
                  <span>{songStudio.providerScorecards.length}</span>
                </div>
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
              </section>

              <section className="tts-subpanel" aria-label="Song outputs">
                <div className="subpanel-heading">
                  <h3>Outputs</h3>
                  <span>{songStudio.submission.job.status}</span>
                </div>
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
              </section>
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

        <section className="review-workspace-panel" aria-label="Waveform Review">
          <div className="samples-header">
            <div>
              <p className="eyebrow">Waveform Review</p>
              <h2>{reviewWorkspace.selectedAsset.asset.name}</h2>
            </div>
            <button
              className="primary-action review-action"
              disabled={!reviewWorkspace.editSubmission.canSave}
              type="button"
              title="Save edited audio version"
            >
              <Save aria-hidden="true" size={18} />
              <span>
                {reviewWorkspace.editSubmission.canSave ? "Save version" : "Blocked"}
              </span>
            </button>
          </div>

          <div className="samples-metrics" aria-label="Waveform review status">
            <div>
              <Library aria-hidden="true" size={18} />
              <strong>{overview.reviewWorkspace.assetCount}</strong>
              <span>assets</span>
            </div>
            <div>
              <Waves aria-hidden="true" size={18} />
              <strong>{overview.reviewWorkspace.previewableAssetCount}</strong>
              <span>previewable</span>
            </div>
            <div>
              <SlidersHorizontal aria-hidden="true" size={18} />
              <strong>{overview.reviewWorkspace.editActionCount}</strong>
              <span>edit actions</span>
            </div>
            <div>
              <ClipboardCheck aria-hidden="true" size={18} />
              <strong>{overview.reviewWorkspace.comparisonCount}</strong>
              <span>comparison</span>
            </div>
          </div>

          <div className="review-layout">
            <div className="review-main">
              <section className="review-transport" aria-label="Waveform transport">
                <div className="review-transport-topline">
                  <button className="icon-control" type="button" title="Play or pause preview">
                    <Play aria-hidden="true" size={18} />
                  </button>
                  <strong>
                    {formatDuration(reviewWorkspace.transport.positionMs)} /{" "}
                    {formatDuration(reviewWorkspace.transport.durationMs)}
                  </strong>
                  <span>{reviewWorkspace.transport.zoomPixelsPerSecond}px/s</span>
                </div>
                <div className="waveform-strip" aria-label="Cached waveform preview">
                  {reviewWorkspace.waveform.peaks.map((peak, index) => (
                    <span
                      aria-hidden="true"
                      className="waveform-bar"
                      key={`${peak.min}-${peak.max}-${index}`}
                      style={{ height: `${Math.max(20, peak.max * 86)}%` }}
                    />
                  ))}
                </div>
                <div className="transport-meta">
                  <span>
                    selection{" "}
                    {formatDuration(reviewWorkspace.transport.selection?.startMs ?? 0)}-
                    {formatDuration(reviewWorkspace.transport.selection?.endMs ?? 0)}
                  </span>
                  <span>
                    loop{" "}
                    {formatDuration(reviewWorkspace.transport.loopRegion?.startMs ?? 0)}-
                    {formatDuration(reviewWorkspace.transport.loopRegion?.endMs ?? 0)}
                  </span>
                  <span>{reviewWorkspace.waveform.cachePath}</span>
                </div>
              </section>

              <div className="review-asset-grid" aria-label="Reviewable assets">
                {reviewWorkspace.assets.map((asset) => (
                  <article
                    className={
                      asset.asset.id === reviewWorkspace.selectedAsset.asset.id
                        ? "review-asset selected"
                        : "review-asset"
                    }
                    key={asset.asset.id}
                  >
                    <div className="sfx-variant-title">
                      <strong>{asset.asset.name}</strong>
                      <span>{statusLabel(asset.asset.kind)}</span>
                    </div>
                    <small>
                      {statusLabel(asset.sourceWorkflow)} / {asset.versions.length} version
                    </small>
                    <p>{asset.canPreview ? "waveform and spectrogram cached" : "preview pending"}</p>
                  </article>
                ))}
              </div>

              <div className="edit-action-grid" aria-label="Lightweight edit actions">
                {reviewWorkspace.editActions.map((action) => (
                  <button
                    className={action.enabled ? "edit-action enabled" : "edit-action"}
                    key={action.id}
                    type="button"
                    title={action.label}
                  >
                    <SlidersHorizontal aria-hidden="true" size={16} />
                    <span>{action.label}</span>
                  </button>
                ))}
              </div>
            </div>

            <div className="review-side">
              <section className="tts-subpanel" aria-label="Version comparison">
                <div className="subpanel-heading">
                  <h3>Version comparison</h3>
                  <span>{reviewWorkspace.versionComparison.mode}</span>
                </div>
                <div className="comparison-grid">
                  {[reviewWorkspace.versionComparison.left, reviewWorkspace.versionComparison.right].map(
                    (side) => (
                      <article key={side.versionId}>
                        <strong>{side.label}</strong>
                        <small>{side.versionId}</small>
                        <p>
                          {formatDuration(side.durationMs)} / {side.loudnessLufs} LUFS /{" "}
                          {side.truePeakDbfs} dBTP
                        </p>
                      </article>
                    ),
                  )}
                </div>
                <div className="comparison-metrics">
                  <span>{reviewWorkspace.versionComparison.metrics.durationDeltaMs}ms</span>
                  <span>
                    {reviewWorkspace.versionComparison.metrics.loudnessDeltaLufs} LUFS
                  </span>
                  <span>
                    diff {reviewWorkspace.versionComparison.metrics.waveformDifferenceScore}
                  </span>
                </div>
              </section>

              <section className="tts-subpanel" aria-label="Edited version">
                <div className="subpanel-heading">
                  <h3>Edited version</h3>
                  <span>{reviewWorkspace.editSubmission.job.status}</span>
                </div>
                <div className="output-card">
                  <strong>{reviewWorkspace.editSubmission.savedVersion.id}</strong>
                  <small>
                    v{reviewWorkspace.editSubmission.savedVersion.versionIndex} /{" "}
                    {reviewWorkspace.editSubmission.savedVersion.file.format}
                  </small>
                  <p>{reviewWorkspace.editSubmission.savedVersion.file.storagePath}</p>
                </div>
              </section>

              <section className="tts-subpanel" aria-label="Recipe provenance">
                <div className="subpanel-heading">
                  <h3>Provenance</h3>
                  <span>{reviewWorkspace.provenance.inspectable ? "inspectable" : "blocked"}</span>
                </div>
                <div className="output-card">
                  <strong>{reviewWorkspace.provenance.editRecipe.id}</strong>
                  <small>
                    {statusLabel(reviewWorkspace.provenance.originalRecipe.workflow)} to{" "}
                    {statusLabel(reviewWorkspace.provenance.editRecipe.workflow)}
                  </small>
                  <p>{reviewWorkspace.provenance.sidecarPath}</p>
                </div>
              </section>
            </div>
          </div>

          <div className="samples-review-grid">
            <ol className="voice-checks" aria-label="Review validation checks">
              {reviewWorkspace.validationChecks.map((check) => (
                <li className={check.status} key={check.id}>
                  <CircleCheck aria-hidden="true" size={16} />
                  <span>{check.summary}</span>
                </li>
              ))}
            </ol>
            <ol className="voice-checks" aria-label="Review shortcuts">
              {reviewWorkspace.transport.keyboardShortcuts.map((shortcut) => (
                <li className="ready" key={shortcut.id}>
                  <ClipboardCheck aria-hidden="true" size={16} />
                  <span>
                    <strong>{shortcut.keys}</strong> {shortcut.action}
                  </span>
                </li>
              ))}
            </ol>
          </div>
        </section>

        <section className="system-grid" aria-label="Architecture">
          <div className="panel">
            <div className="panel-heading">
              <h2>Runtime Layers</h2>
              <span>{overview.architecture.layers.length}</span>
            </div>
            <ol className="layer-list">
              {overview.architecture.layers.map((layer) => (
                <li key={layer.id}>
                  <span className={`layer-dot ${layer.status}`} />
                  <div>
                    <strong>{layer.name}</strong>
                    <p>{layer.responsibility}</p>
                  </div>
                </li>
              ))}
            </ol>
          </div>

          <div className="panel">
            <div className="panel-heading">
              <h2>Command Boundary</h2>
              <span>{overview.commands.length}</span>
            </div>
            <div className="command-list">
              {overview.commands.map((command) => (
                <article className="command-row" key={command.name}>
                  <strong>{command.name}</strong>
                  <span>{command.direction}</span>
                  <p>{command.purpose}</p>
                </article>
              ))}
            </div>
          </div>

          <div className="panel provider-panel">
            <div className="panel-heading">
              <h2>Provider Coverage</h2>
              <span>{overview.providerCatalog.capabilityCount}</span>
            </div>
            <div className="provider-metrics" aria-label="Provider catalog">
              <div>
                <strong>{overview.providerCatalog.providerCount}</strong>
                <span>providers</span>
              </div>
              <div>
                <strong>{overview.providerCatalog.modelCount}</strong>
                <span>models</span>
              </div>
            </div>
            <ol className="workflow-list">
              {overview.providerCatalog.workflows.map((workflow) => (
                <li key={workflow.workflow}>
                  <Cpu aria-hidden="true" size={16} />
                  <span>{workflowLabel(workflow.workflow)}</span>
                  <small>{workflow.defaultModelId}</small>
                </li>
              ))}
            </ol>
          </div>

          <div className="panel evaluation-panel">
            <div className="panel-heading">
              <h2>Evaluation Scorecard</h2>
              <span>{overview.modelEvaluation.candidateCount}</span>
            </div>
            <div className="evaluation-summary" aria-label="Model evaluation">
              <div>
                <ClipboardCheck aria-hidden="true" size={18} />
                <strong>{overview.modelEvaluation.sourceCount}</strong>
                <span>sources</span>
              </div>
              <div>
                <Boxes aria-hidden="true" size={18} />
                <strong>{overview.modelEvaluation.fixtureCount}</strong>
                <span>fixtures</span>
              </div>
              <div>
                <CircleCheck aria-hidden="true" size={18} />
                <strong>
                  {countFor(
                    overview.modelEvaluation.productEligibilityCounts,
                    "product-candidate",
                  )}
                </strong>
                <span>product candidates</span>
              </div>
              <div>
                <CircleAlert aria-hidden="true" size={18} />
                <strong>
                  {countFor(overview.modelEvaluation.statusCounts, "blocked")}
                </strong>
                <span>blocked</span>
              </div>
            </div>
            <ol className="recommendation-list" aria-label="Recommended spikes">
              {overview.modelEvaluation.recommendedCandidateIds.map(
                (candidateId) => (
                  <li key={candidateId}>
                    <span>{candidateId}</span>
                  </li>
                ),
              )}
            </ol>
          </div>

          <div className="panel runtime-panel">
            <div className="panel-heading">
              <h2>Worker Runtime</h2>
              <span>{runtime.schemaVersion}</span>
            </div>
            <div className="runtime-summary" aria-label="Runtime status">
              <div>
                <PackageCheck aria-hidden="true" size={18} />
                <strong>{runtime.statusCounts.installed}</strong>
                <span>installed</span>
              </div>
              <div>
                <HardDrive aria-hidden="true" size={18} />
                <strong>{runtime.statusCounts.available}</strong>
                <span>available</span>
              </div>
              <div>
                <CircleAlert aria-hidden="true" size={18} />
                <strong>{runtime.statusCounts.unavailable}</strong>
                <span>unavailable</span>
              </div>
              <div>
                <Cpu aria-hidden="true" size={18} />
                <strong>{runtime.devices.length}</strong>
                <span>devices</span>
              </div>
            </div>

            <div className="runtime-policy">
              <strong>{runtime.packagingPolicy.name}</strong>
              <span>
                Python runtime:{" "}
                {runtime.packagingPolicy.productRuntimeAllowsPython
                  ? "allowed"
                  : "blocked"}
              </span>
              <small>
                {runtime.packagingPolicy.shippedPlatforms
                  .map(statusLabel)
                  .join(" / ")}
              </small>
            </div>

            <div className="runtime-columns">
              <div className="runtime-stack">
                <h3>Models</h3>
                <ol className="runtime-list">
                  {runtime.modelStates.map((model) => (
                    <li key={`${model.providerId}-${model.modelId}`}>
                      <span className={`runtime-dot ${model.availability}`} />
                      <div>
                        <strong>{model.modelName}</strong>
                        <small>
                          {statusLabel(model.availability)} /{" "}
                          {statusLabel(model.installStatus)} /{" "}
                          {formatMb(model.cache.diskUsageMb)}
                        </small>
                      </div>
                    </li>
                  ))}
                </ol>
              </div>

              <div className="runtime-stack">
                <h3>Jobs</h3>
                <ol className="runtime-list">
                  {runtime.jobs.map((job) => (
                    <li key={job.id}>
                      <span className={`runtime-dot ${job.status}`} />
                      <div>
                        <strong>{statusLabel(job.kind)}</strong>
                        <small>
                          {statusLabel(job.status)} /{" "}
                          {Math.round(job.progress?.percent ?? 0)}% /{" "}
                          {statusLabel(job.cancellation)}
                        </small>
                        {job.actionableError ? (
                          <em>{job.actionableError.recovery}</em>
                        ) : null}
                      </div>
                    </li>
                  ))}
                </ol>
              </div>
            </div>

            <ol className="validation-list" aria-label="Runtime checks">
              {runtime.validationChecks.map((check) => (
                <li key={check.id}>
                  <CircleCheck aria-hidden="true" size={16} />
                  <span>{check.summary}</span>
                </li>
              ))}
            </ol>
          </div>
        </section>
      </section>
    </main>
  );
}
