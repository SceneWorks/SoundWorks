// DR-02 + UX-08: Voice Lab studio. UX-08 adds consent capture (the F-003 write
// path via recordVoiceProfileConsent), editable source/target pickers feeding the
// conversion, a live progress panel driven by the UX-F1 poll, and inline playback
// of the result. Conversion has no executable adapter today, so Convert is
// honestly gated by ModelAvailabilityGate; the consent write path is real and
// unblocks F-003 admission once a conversion model is installed.
import { useEffect, useRef, useState } from "react";
import { ClipboardCheck, Play, Radio, ShieldCheck } from "lucide-react";
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

export function VoiceLabScreen() {
  const {
    voiceLab,
    voiceRuntimeModel,
    voiceCandidateFocus,
    runRuntimeJob,
    runtimeOperation,
    cancelRuntimeOperation,
    retryRuntimeOperation,
    recordVoiceProfileConsent,
    assetLibrary,
    libraryPlayback,
    previewLibraryItem,
    overview,
    openModelsFor,
  } = useAppContext();

  const request = voiceLab.selectedConversion.recipe.request;
  const editedRef = useRef(false);
  const [targetProfileId, setTargetProfileId] = useState(
    request.targetVoiceProfileId ?? voiceLab.voiceProfiles[0]?.profile.id ?? "",
  );
  const [sourceAssetId, setSourceAssetId] = useState(
    request.sourceAudioAssetId ?? "",
  );

  useEffect(() => {
    if (!editedRef.current) {
      setTargetProfileId(
        request.targetVoiceProfileId ??
          voiceLab.voiceProfiles[0]?.profile.id ??
          "",
      );
      setSourceAssetId(request.sourceAudioAssetId ?? "");
    }
  }, [
    request.targetVoiceProfileId,
    request.sourceAudioAssetId,
    voiceLab.voiceProfiles,
  ]);

  const targetProfile = voiceLab.voiceProfiles.find(
    (profile) => profile.profile.id === targetProfileId,
  );
  const targetConsented =
    targetProfile?.profile.consent === "explicit-consent-recorded";
  const blockReason = !voiceRuntimeModel
    ? "Install a voice-conversion model to generate."
    : !targetProfileId
      ? "Select a target voice profile."
      : !targetConsented
        ? "Record explicit consent for the target voice profile."
        : null;

  function convert() {
    runRuntimeJob(
      "voice-conversion",
      `Voice conversion ${sourceAssetId || "source"} -> ${targetProfileId || "target"}`,
      {
        sourceAudioAssetId: sourceAssetId || null,
        targetVoiceProfileId: targetProfileId || null,
        preserveTiming: request.preserveTiming ?? true,
      },
    );
  }

  const selectedItemId = assetLibrary.selectedItem?.item.id;

  return (
    <section className="voice-lab-panel" aria-label="Voice Lab">
      <SurfaceHeader
        eyebrow="Voice Lab"
        title="Consented voice workflows"
        actions={
          <button
            className="primary-action voice-action"
            disabled={Boolean(blockReason)}
            onClick={convert}
            type="button"
            title={blockReason ?? "Queue voice conversion"}
          >
            <Radio aria-hidden="true" size={18} />
            <span>{voiceRuntimeModel ? "Convert" : "Blocked"}</span>
          </button>
        }
        stats={
          <>
            <HeroStat label="modes" value={overview.voiceLab.modeCount} />
            <HeroStat label="profiles" value={overview.voiceLab.profileCount} />
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
        onOpenModelManager={() => openModelsFor("voice-conversion")}
      />

      <MainSurface className="studio-compose" ariaLabel="Conversion setup">
        <SectionHeading title="Convert" eyebrow="source -> target" />
        <div className="field-row">
          <label className="field">
            <span>Source audio asset id</span>
            <input
              className="field-input"
              type="text"
              value={sourceAssetId}
              onChange={(event) => {
                editedRef.current = true;
                setSourceAssetId(event.target.value);
              }}
              placeholder="Paste a source audio asset id…"
            />
          </label>
          <label className="field">
            <span>Target voice profile</span>
            <select
              className="field-input"
              value={targetProfileId}
              onChange={(event) => {
                editedRef.current = true;
                setTargetProfileId(event.target.value);
              }}
            >
              {voiceLab.voiceProfiles.map((profile) => (
                <option key={profile.profile.id} value={profile.profile.id}>
                  {profile.profile.displayName} (
                  {statusLabel(profile.profile.consent)})
                </option>
              ))}
            </select>
          </label>
        </div>
        {targetProfile && !targetConsented ? (
          <p className="field-hint">
            {targetProfile.profile.displayName} has no recorded consent —
            conversion is blocked until consent is recorded below.
          </p>
        ) : null}
      </MainSurface>

      <GenerationPanel
        job={runtimeOperation}
        workflows={["voice-conversion"]}
        typeLabel="Voice conversion"
        onCancel={cancelRuntimeOperation}
        onRetry={retryRuntimeOperation}
      >
        <button
          type="button"
          className="secondary-action"
          disabled={!selectedItemId}
          onClick={() => selectedItemId && previewLibraryItem(selectedItemId)}
          title="Play the converted clip"
        >
          <Play aria-hidden="true" size={16} />
          <span>Play latest</span>
        </button>
        <PlaybackControl playback={libraryPlayback} />
      </GenerationPanel>

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
          {voiceLab.voiceProfiles.map((profile) => {
            const consented =
              profile.profile.consent === "explicit-consent-recorded";
            return (
              <article className="voice-profile-card" key={profile.profile.id}>
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
                <div className="voice-consent-actions">
                  <button
                    type="button"
                    className="secondary-action"
                    disabled={consented}
                    onClick={() =>
                      recordVoiceProfileConsent(
                        profile.profile.id,
                        "explicit-consent-recorded",
                      )
                    }
                    title="Record explicit consent for this voice profile"
                  >
                    {consented ? "Consent recorded" : "Record consent"}
                  </button>
                  <button
                    type="button"
                    className="secondary-action"
                    onClick={() =>
                      recordVoiceProfileConsent(
                        profile.profile.id,
                        "prohibited",
                      )
                    }
                    title="Mark this voice profile as prohibited"
                  >
                    Prohibit
                  </button>
                </div>
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
            );
          })}
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
                    {scorecard.blockers[0] ? <p>{scorecard.blockers[0]}</p> : null}
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
