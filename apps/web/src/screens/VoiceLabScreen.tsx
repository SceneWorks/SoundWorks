// DR-02: Voice Lab studio. Extracted from App.tsx and rebuilt on the shared
// grammar (SurfaceHeader hero + HeroStat for the header/metrics, MainSurface +
// SectionHeading for the side panels) in place of the bespoke
// voice-lab-header / voice-lab-metrics / tts-subpanel / subpanel-heading
// classes, following the TtsScreen template. All Convert wiring, gating, and
// data bindings are preserved verbatim.
import { ClipboardCheck, Radio, ShieldCheck } from "lucide-react";
import {
  HeroStat,
  MainSurface,
  SectionHeading,
  SurfaceHeader,
  ModelAvailabilityGate,
} from "../components";
import { formatDuration, statusLabel, workflowLabel } from "../viewModel";
import { useAppContext } from "./context";

export function VoiceLabScreen() {
  const {
    voiceLab,
    voiceRuntimeModel,
    voiceCandidateFocus,
    runRuntimeJob,
    overview,
    setActiveView,
  } = useAppContext();

  return (
    <section className="voice-lab-panel" aria-label="Voice Lab">
      <SurfaceHeader
        eyebrow="Voice Lab"
        title="Consented voice workflows"
        actions={
          <button
            className="primary-action voice-action"
            disabled={!voiceRuntimeModel}
            onClick={() => {
              const conversion = voiceLab.selectedConversion.recipe.request;
              runRuntimeJob(
                "voice-conversion",
                `Voice conversion ${conversion.sourceAudioAssetId ?? "source"} -> ${conversion.targetVoiceProfileId ?? "target"}`,
                {
                  sourceAudioAssetId: conversion.sourceAudioAssetId ?? null,
                  targetVoiceProfileId:
                    conversion.targetVoiceProfileId ?? null,
                  preserveTiming: conversion.preserveTiming ?? true,
                },
              );
            }}
            type="button"
            title="Queue voice conversion"
          >
            <Radio aria-hidden="true" size={18} />
            <span>{voiceRuntimeModel ? "Convert" : "Blocked"}</span>
          </button>
        }
        stats={
          <>
            <HeroStat label="modes" value={overview.voiceLab.modeCount} />
            <HeroStat
              label="profiles"
              value={overview.voiceLab.profileCount}
            />
            <HeroStat
              label="scorecards"
              value={overview.voiceLab.providerCount}
            />
            <HeroStat
              label={overview.voiceLab.selectedConversionCandidateId}
              value={overview.voiceLab.savedAssetKind}
            />
          </>
        }
      />

      <ModelAvailabilityGate
        installed={Boolean(voiceRuntimeModel)}
        label="voice conversion"
        onOpenModelManager={() => setActiveView("models")}
      />

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
            <article
              className="voice-profile-card"
              key={profile.profile.id}
            >
              <div className="voice-profile-topline">
                <div>
                  <strong>{profile.profile.displayName}</strong>
                  <small>
                    {profile.language} /{" "}
                    {statusLabel(profile.profile.consent)}
                  </small>
                </div>
                <span
                  className={
                    profile.commercialUseAllowed
                      ? "voice-approval approved"
                      : "voice-approval review"
                  }
                >
                  {profile.commercialUseAllowed ? "licensed" : "review"}
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
          <MainSurface ariaLabel="Conversion source">
            <SectionHeading
              title="Conversion"
              eyebrow={voiceLab.selectedConversion.job.status}
            />
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
          </MainSurface>

          <MainSurface ariaLabel="Voice providers">
            <SectionHeading
              title="Providers"
              eyebrow={voiceLab.providerScorecards.length}
            />
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
                    {scorecard.blockers[0] ? (
                      <p>{scorecard.blockers[0]}</p>
                    ) : null}
                  </div>
                  {scorecard.recommended ? <span>pick</span> : null}
                </article>
              ))}
            </div>
          </MainSurface>
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
  );
}
