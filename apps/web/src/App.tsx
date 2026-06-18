import {
  Activity,
  Boxes,
  CircleAlert,
  CircleCheck,
  ClipboardCheck,
  Cpu,
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
  fallbackRuntime,
  fallbackTtsStudio,
} from "./appData";
import {
  loadAppOverview,
  loadRuntimeOverview,
  loadTtsStudioOverview,
} from "./tauri";
import type { AppOverview, RuntimeOverview, TtsStudioOverview } from "./types";

const navItems = [
  { label: "Studios", icon: Sparkles },
  { label: "Library", icon: Library },
  { label: "Mixer", icon: SlidersHorizontal },
  { label: "Jobs", icon: Activity },
];

const studioIcons = [Mic2, Radio, Waves, Boxes, Music2, Sparkles];

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
