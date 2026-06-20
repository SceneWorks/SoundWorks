// DR-02: Video-to-Audio studio. Extracted from App.tsx and rebuilt on the shared
// grammar (SurfaceHeader hero + HeroStat for the header/metrics, MainSurface +
// SectionHeading for the side panels) in place of the bespoke video-header /
// video-metrics / tts-subpanel / subpanel-heading classes, following the
// TtsScreen template. All Generate wiring, gating, and data bindings are
// preserved verbatim.
import { CircleCheck, FileVideo, ShieldCheck } from "lucide-react";
import {
  HeroStat,
  MainSurface,
  SectionHeading,
  SurfaceHeader,
  ModelAvailabilityGate,
} from "../components";
import { formatDuration, statusLabel } from "../viewModel";
import { useAppContext } from "./context";

export function VideoToAudioScreen() {
  const {
    videoToAudio,
    videoRuntimeModel,
    videoCandidateFocus,
    runRuntimeJob,
    overview,
    setActiveView,
  } = useAppContext();

  return (
    <section className="video-to-audio-panel" aria-label="Video to Audio">
      <SurfaceHeader
        eyebrow="Video to Audio"
        title={videoToAudio.source.filename}
        actions={
          <button
            className="primary-action video-action"
            disabled={!videoRuntimeModel}
            onClick={() =>
              runRuntimeJob(
                "video-to-audio",
                videoToAudio.direction.prompt,
                {
                  videoAssetId: videoToAudio.source.videoAssetId,
                  syncMode: videoToAudio.direction.syncMode,
                  negativePrompt: videoToAudio.direction.negativePrompt,
                  durationMs: videoToAudio.direction.durationMs,
                  targetRangeIds: videoToAudio.targetRanges.map(
                    (target) => target.id,
                  ),
                },
              )
            }
            type="button"
            title="Queue video-to-audio generation"
          >
            <FileVideo aria-hidden="true" size={18} />
            <span>{videoRuntimeModel ? "Generate" : "Blocked"}</span>
          </button>
        }
        stats={
          <>
            <HeroStat
              label={videoToAudio.source.frameRate}
              value={formatDuration(videoToAudio.source.durationMs)}
            />
            <HeroStat
              label="ranges"
              value={overview.videoToAudio.targetRangeCount}
            />
            <HeroStat
              label="sync points"
              value={overview.videoToAudio.syncPointCount}
            />
            <HeroStat
              label="scorecards"
              value={overview.videoToAudio.scorecardCount}
            />
          </>
        }
      />

      <ModelAvailabilityGate
        installed={Boolean(videoRuntimeModel)}
        label="video-to-audio"
        onOpenModelManager={() => setActiveView("models")}
      />

      <div className="video-layout">
        <div className="video-main">
          <section
            className="video-source-panel"
            aria-label="Video source and direction"
          >
            <div className="subpanel-heading">
              <h3>Source and direction</h3>
              <span>{videoToAudio.source.resolution}</span>
            </div>
            <p>{videoToAudio.direction.prompt}</p>
            <small>{videoToAudio.direction.negativePrompt}</small>
            <div className="candidate-strip">
              <span>{statusLabel(videoToAudio.direction.syncMode)}</span>
              <span>
                {videoToAudio.source.hasSourceAudio
                  ? "source audio"
                  : "silent video"}
              </span>
              <span>
                {videoToAudio.source.imageReferenceIds.length} keyframe
              </span>
              <span>
                {videoToAudio.source.referenceAudioAssetIds.length}{" "}
                reference audio
              </span>
            </div>
          </section>

          <div className="video-range-list" aria-label="Target ranges">
            {videoToAudio.targetRanges.map((range) => (
              <article className="video-range" key={range.id}>
                <div>
                  <strong>{range.label}</strong>
                  <small>
                    {formatDuration(range.range.startMs)}-
                    {formatDuration(range.range.endMs)}
                  </small>
                </div>
                <p>{range.requestedAction}</p>
                <span>{range.objectLabel ?? "full frame"}</span>
              </article>
            ))}
          </div>

          <div className="sync-timeline" aria-label="Sync preview">
            {videoToAudio.syncPreview.segments.map((segment) => (
              <article
                className="sync-segment"
                key={segment.id}
                style={{
                  marginLeft: `${(segment.range.startMs / videoToAudio.syncPreview.durationMs) * 100}%`,
                  width: `${Math.max(
                    8,
                    ((segment.range.endMs - segment.range.startMs) /
                      videoToAudio.syncPreview.durationMs) *
                      100,
                  )}%`,
                }}
              >
                <strong>{segment.label}</strong>
                <span>{Math.round(segment.syncConfidence * 100)}%</span>
              </article>
            ))}
          </div>

          <div className="video-event-grid" aria-label="Detected events">
            {videoToAudio.detectedEvents.map((event) => (
              <div className="video-event" key={event.id}>
                <strong>{event.label}</strong>
                <span>{formatDuration(event.atMs)}</span>
                <small>{event.requestedSound}</small>
              </div>
            ))}
          </div>
        </div>

        <div className="video-side">
          <MainSurface ariaLabel="Video-to-audio provider options">
            <SectionHeading
              title="Provider"
              eyebrow={videoToAudio.providerOptions.length}
            />
            {videoToAudio.providerOptions.map((provider) => (
              <article
                className="sfx-provider-option"
                key={`${provider.workflow}-${provider.modelId}`}
              >
                <strong>{provider.displayName}</strong>
                <small>
                  {statusLabel(provider.installStatus)} /{" "}
                  {provider.sampleRateHz} Hz /{" "}
                  {statusLabel(provider.channelLayout)}
                </small>
                <p>
                  {[
                    provider.supportsVideo ? "video" : null,
                    provider.supportsText ? "text" : null,
                    provider.supportsRangeRefinement ? "ranges" : null,
                    provider.supportsObjectRegions ? "regions" : null,
                  ]
                    .filter(Boolean)
                    .join(" / ")}
                </p>
              </article>
            ))}
          </MainSurface>

          <MainSurface ariaLabel="Video-to-audio scorecards">
            <SectionHeading
              title="Scorecards"
              eyebrow={videoToAudio.providerScorecards.length}
            />
            <div className="voice-provider-list">
              {videoCandidateFocus.map((scorecard) => (
                <article
                  className={`voice-provider ${scorecard.readiness}`}
                  key={scorecard.candidateId}
                >
                  <div>
                    <strong>{scorecard.name}</strong>
                    <small>
                      {statusLabel(scorecard.readiness)} /{" "}
                      {scorecard.supports.map(statusLabel).join(" / ")}
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

          <MainSurface ariaLabel="Video output">
            <SectionHeading
              title="Output"
              eyebrow={videoToAudio.submission.job.status}
            />
            <div className="output-card">
              <strong>{videoToAudio.savedOutput.asset.name}</strong>
              <small>
                {videoToAudio.savedOutput.asset.kind} /{" "}
                {videoToAudio.savedOutput.asset.currentVersionId}
              </small>
              <p>{videoToAudio.savedOutput.version.file.storagePath}</p>
            </div>
          </MainSurface>
        </div>
      </div>

      <div className="video-review-grid">
        <ol className="voice-checks" aria-label="Video-to-audio gates">
          {videoToAudio.safetyGates.map((gate) => (
            <li className={gate.status} key={gate.id}>
              <ShieldCheck aria-hidden="true" size={16} />
              <span>{gate.summary}</span>
            </li>
          ))}
        </ol>
        <section className="video-sidecar" aria-label="Video sidecar">
          <div className="subpanel-heading">
            <h3>Sidecar</h3>
            <span>
              {videoToAudio.exportPackage.requiredFields.length}
            </span>
          </div>
          <p>{videoToAudio.exportPackage.sidecarPath}</p>
          <div className="candidate-strip">
            {videoToAudio.exportPackage.destinationTargets.map(
              (target) => (
                <span key={target}>{target}</span>
              ),
            )}
          </div>
        </section>
        <ol
          className="voice-checks"
          aria-label="Video-to-audio validation"
        >
          {videoToAudio.validationChecks.map((check) => (
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
