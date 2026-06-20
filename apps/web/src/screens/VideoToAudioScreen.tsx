// DR-02 + UX-09: Video-to-Audio studio. UX-09 adds an editable direction draft
// (source asset id, prompt, sync mode, duration, negative prompt) and an
// add/remove timed-event editor, a live progress panel driven by the UX-F1 poll,
// and inline playback of the saved result. No video-to-audio model ships today,
// so generation is honestly gated by ModelAvailabilityGate — the draft feeds
// runRuntimeJob once a model is installed.
import { useEffect, useRef, useState } from "react";
import { CircleCheck, FileVideo, Play, ShieldCheck } from "lucide-react";
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

interface TimedEvent {
  label: string;
  atMs: number;
  requestedSound: string;
}

export function VideoToAudioScreen() {
  const {
    videoToAudio,
    videoRuntimeModel,
    videoCandidateFocus,
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

  const editedRef = useRef(false);
  const [videoAssetId, setVideoAssetId] = useState(
    videoToAudio.source.videoAssetId,
  );
  const [prompt, setPrompt] = useState(videoToAudio.direction.prompt);
  const [syncMode, setSyncMode] = useState(videoToAudio.direction.syncMode);
  const [durationMs, setDurationMs] = useState(
    videoToAudio.direction.durationMs,
  );
  const [events, setEvents] = useState<TimedEvent[]>(
    videoToAudio.detectedEvents.map((event) => ({
      label: event.label,
      atMs: event.atMs,
      requestedSound: event.requestedSound,
    })),
  );

  useEffect(() => {
    if (!editedRef.current) {
      setVideoAssetId(videoToAudio.source.videoAssetId);
      setPrompt(videoToAudio.direction.prompt);
      setSyncMode(videoToAudio.direction.syncMode);
      setDurationMs(videoToAudio.direction.durationMs);
      setEvents(
        videoToAudio.detectedEvents.map((event) => ({
          label: event.label,
          atMs: event.atMs,
          requestedSound: event.requestedSound,
        })),
      );
    }
  }, [
    videoToAudio.source.videoAssetId,
    videoToAudio.direction.prompt,
    videoToAudio.direction.syncMode,
    videoToAudio.direction.durationMs,
    videoToAudio.detectedEvents,
  ]);

  const edit = () => {
    editedRef.current = true;
  };
  const trimmed = prompt.trim();
  const blockReason = !videoRuntimeModel
    ? "Install a video-to-audio model to generate."
    : videoAssetId.trim().length === 0
      ? "Select a video source to generate."
      : trimmed.length === 0
        ? "Describe the audio to generate."
        : null;

  function generate() {
    runRuntimeJob("video-to-audio", prompt, {
      videoAssetId,
      syncMode,
      negativePrompt: videoToAudio.direction.negativePrompt,
      durationMs,
      events: events.map((event) => ({
        label: event.label,
        atMs: event.atMs,
        requestedSound: event.requestedSound,
      })),
    });
  }

  const selectedItemId = assetLibrary.selectedItem?.item.id;

  return (
    <section className="video-to-audio-panel" aria-label="Video to Audio">
      <SurfaceHeader
        eyebrow="Video to Audio"
        title={videoToAudio.source.filename}
        actions={
          <button
            className="primary-action video-action"
            disabled={Boolean(blockReason)}
            onClick={generate}
            type="button"
            title={blockReason ?? "Queue video-to-audio generation"}
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
            <HeroStat label="events" value={events.length} />
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

      <MainSurface className="studio-compose" ariaLabel="Direct video audio">
        <SectionHeading title="Direction" eyebrow="source + events" />
        <div className="field-row">
          <label className="field">
            <span>Video source asset id</span>
            <input
              className="field-input"
              type="text"
              value={videoAssetId}
              onChange={(event) => {
                edit();
                setVideoAssetId(event.target.value);
              }}
              placeholder="Paste a project video asset id…"
            />
          </label>
          <label className="field">
            <span>Sync mode</span>
            <input
              className="field-input"
              type="text"
              value={syncMode}
              onChange={(event) => {
                edit();
                setSyncMode(event.target.value);
              }}
            />
          </label>
          <label className="field">
            <span>Duration {formatDuration(durationMs)}</span>
            <input
              className="field-input"
              type="range"
              min={1000}
              max={Math.max(60000, videoToAudio.source.durationMs)}
              step={500}
              value={durationMs}
              onChange={(event) => {
                edit();
                setDurationMs(Number(event.target.value));
              }}
            />
          </label>
        </div>
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
            placeholder="Describe the audio to generate for this video…"
          />
          <small className="field-hint">{trimmed.length} characters</small>
        </label>
        <div className="field">
          <span>Timed events</span>
          <div className="editable-list">
            {events.map((event, index) => (
              <div className="editable-row" key={index}>
                <input
                  className="field-input"
                  type="text"
                  value={event.label}
                  aria-label="Event label"
                  onChange={(change) => {
                    edit();
                    setEvents((current) =>
                      current.map((value, position) =>
                        position === index
                          ? { ...value, label: change.target.value }
                          : value,
                      ),
                    );
                  }}
                />
                <input
                  className="field-input field-input-narrow"
                  type="number"
                  min={0}
                  value={event.atMs}
                  aria-label="Event time (ms)"
                  onChange={(change) => {
                    edit();
                    setEvents((current) =>
                      current.map((value, position) =>
                        position === index
                          ? { ...value, atMs: Number(change.target.value) }
                          : value,
                      ),
                    );
                  }}
                />
                <button
                  type="button"
                  className="secondary-action"
                  onClick={() => {
                    edit();
                    setEvents((current) =>
                      current.filter((_, position) => position !== index),
                    );
                  }}
                  title="Remove event"
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
                setEvents((current) => [
                  ...current,
                  { label: "New event", atMs: 0, requestedSound: "" },
                ]);
              }}
            >
              Add event
            </button>
          </div>
        </div>
      </MainSurface>

      <GenerationPanel
        job={runtimeOperation}
        workflows={["video-to-audio"]}
        typeLabel="Video to Audio"
        onCancel={cancelRuntimeOperation}
        onRetry={retryRuntimeOperation}
      >
        <button
          type="button"
          className="secondary-action"
          disabled={!selectedItemId}
          onClick={() => selectedItemId && previewLibraryItem(selectedItemId)}
          title="Play the saved audio"
        >
          <Play aria-hidden="true" size={16} />
          <span>Play latest</span>
        </button>
        <PlaybackControl playback={libraryPlayback} />
      </GenerationPanel>

      <div className="video-layout">
        <div className="video-main">
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
                  {statusLabel(provider.installStatus)} / {provider.sampleRateHz}{" "}
                  Hz / {statusLabel(provider.channelLayout)}
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
                    {scorecard.blockers[0] ? <p>{scorecard.blockers[0]}</p> : null}
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
            <span>{videoToAudio.exportPackage.requiredFields.length}</span>
          </div>
          <p>{videoToAudio.exportPackage.sidecarPath}</p>
          <div className="candidate-strip">
            {videoToAudio.exportPackage.destinationTargets.map((target) => (
              <span key={target}>{target}</span>
            ))}
          </div>
        </section>
        <ol className="voice-checks" aria-label="Video-to-audio validation">
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
