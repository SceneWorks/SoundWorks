import { useState } from "react";
import {
  CircleCheck,
  ClipboardCheck,
  Disc3,
  Plus,
  Scissors,
  Trash2,
  MoveHorizontal,
  Play,
} from "lucide-react";
import {
  GenerationPanel,
  HeroStat,
  MainSurface,
  PlaybackControl,
  SectionHeading,
  SurfaceHeader,
} from "../components";
import {
  formatDuration,
  isHttpUrl,
  scopeLabel,
  statusLabel,
} from "../viewModel";
import { useAppContext } from "./context";

export function MultitrackScreen() {
  const {
    compositionEditor,
    overview,
    renderComposition,
    addCompositionClipToTrack,
    moveCompositionClip,
    trimCompositionClip,
    deleteCompositionClip,
    addCompositionTrack,
    updateCompositionTrack,
    runtimeOperation,
    cancelRuntimeOperation,
    retryRuntimeOperation,
    assetLibrary,
    libraryPlayback,
    previewLibraryItem,
  } = useAppContext();

  const [selectedClipId, setSelectedClipId] = useState(
    compositionEditor.timeline.selectedClipId,
  );
  const canRender = compositionEditor.exportPlan.canRenderMixdown;
  const selectedItemId = assetLibrary.selectedItem?.item.id;
  const trackWithSelectedClip = compositionEditor.tracks.find((track) =>
    track.clips.some((clip) => clip.clipId === selectedClipId),
  );
  const selectedClip =
    trackWithSelectedClip?.clips.find((clip) => clip.clipId === selectedClipId) ??
    compositionEditor.tracks.flatMap((track) => track.clips)[0];
  const selectedTrackId =
    trackWithSelectedClip?.trackId ?? compositionEditor.tracks[0]?.trackId;
  const defaultAsset = compositionEditor.assetBin[0];
  const snapMs = compositionEditor.timeline.snapGridMs;

  const moveSelectedClip = (direction: -1 | 1) => {
    if (!selectedClip) {
      return;
    }
    moveCompositionClip(
      selectedClip.clipId,
      Math.max(0, selectedClip.timelineStartMs + direction * snapMs),
    );
  };

  const trimSelectedClip = () => {
    if (!selectedClip) {
      return;
    }
    const startMs = Math.min(
      selectedClip.sourceRange.startMs + snapMs,
      selectedClip.sourceRange.endMs - 250,
    );
    trimCompositionClip(
      selectedClip.clipId,
      {
        startMs: Math.max(0, startMs),
        endMs: selectedClip.sourceRange.endMs,
      },
      selectedClip.fadeInMs,
      selectedClip.fadeOutMs,
    );
  };

  return (
    <section
      className="composition-editor-panel"
      aria-label="Multitrack Composition Editor"
    >
      <SurfaceHeader
        eyebrow="Multitrack editor"
        title={compositionEditor.composition.name}
        actions={
          <button
            className="primary-action composition-action"
            disabled={!canRender}
            onClick={() => renderComposition()}
            type="button"
            title={
              canRender
                ? "Render the composition into a mixed-down asset"
                : "Render is blocked — see the render plan below"
            }
          >
            <Disc3 aria-hidden="true" size={18} />
            <span>{canRender ? "Render Mixdown" : "Blocked"}</span>
          </button>
        }
        stats={
          <>
            <HeroStat
              label="tracks"
              value={overview.compositionEditor.trackCount}
            />
            <HeroStat
              label="clips"
              value={overview.compositionEditor.clipCount}
            />
            <HeroStat
              label="assets"
              value={overview.compositionEditor.assetBinCount}
            />
            <HeroStat
              label={`${compositionEditor.timeline.snapGridMs}ms grid`}
              value={`${compositionEditor.timeline.zoomPercent}%`}
            />
          </>
        }
      />

      <GenerationPanel
        job={runtimeOperation}
        workflows={["composition-render"]}
        typeLabel="Mixdown"
        onCancel={cancelRuntimeOperation}
        onRetry={retryRuntimeOperation}
      >
        <button
          type="button"
          className="secondary-action"
          disabled={!selectedItemId}
          onClick={() => selectedItemId && previewLibraryItem(selectedItemId)}
          title="Play the rendered mixdown"
        >
          <Play aria-hidden="true" size={16} />
          <span>Play latest</span>
        </button>
        <PlaybackControl playback={libraryPlayback} />
      </GenerationPanel>

      <div className="composition-layout">
        <div className="composition-main">
          <section className="composition-toolbar" aria-label="Editor tools">
            <button
              className="tool-button"
              type="button"
              disabled={!selectedTrackId || !defaultAsset}
              onClick={() =>
                selectedTrackId &&
                defaultAsset &&
                addCompositionClipToTrack(selectedTrackId, defaultAsset)
              }
              title="Add the first bin asset to the selected track"
            >
              <Plus aria-hidden="true" size={16} />
              <span>Add clip</span>
            </button>
            <button
              className="tool-button"
              type="button"
              disabled={!selectedClip}
              onClick={() => moveSelectedClip(-1)}
              title="Move selected clip earlier"
            >
              <MoveHorizontal aria-hidden="true" size={16} />
              <span>Earlier</span>
            </button>
            <button
              className="tool-button"
              type="button"
              disabled={!selectedClip}
              onClick={() => moveSelectedClip(1)}
              title="Move selected clip later"
            >
              <MoveHorizontal aria-hidden="true" size={16} />
              <span>Later</span>
            </button>
            <button
              className="tool-button"
              type="button"
              disabled={!selectedClip}
              onClick={trimSelectedClip}
              title="Trim selected clip by one grid step"
            >
              <Scissors aria-hidden="true" size={16} />
              <span>Trim</span>
            </button>
            <button
              className="tool-button"
              type="button"
              disabled={!selectedClip}
              onClick={() => selectedClip && deleteCompositionClip(selectedClip.clipId)}
              title="Delete selected clip"
            >
              <Trash2 aria-hidden="true" size={16} />
              <span>Delete</span>
            </button>
            <button
              className="tool-button"
              type="button"
              onClick={() => addCompositionTrack("sfx")}
              title="Add a new SFX track"
            >
              <Plus aria-hidden="true" size={16} />
              <span>Add track</span>
            </button>
            {compositionEditor.tools.slice(0, 3).map((tool) => (
              <button
                className={
                  tool.id === compositionEditor.timeline.selectedTool
                    ? "tool-button selected"
                    : "tool-button"
                }
                key={tool.id}
                title={tool.label}
                type="button"
              >
                <span>{tool.label}</span>
              </button>
            ))}
          </section>

          <section className="timeline-board" aria-label="Timeline tracks">
            <div className="timeline-selection" aria-label="Timeline selection">
              <span>{selectedClip?.clipId ?? "no clip selected"}</span>
              <span>
                cursor{" "}
                {formatDuration(compositionEditor.timeline.playbackCursorMs)}
              </span>
              <span>
                loop{" "}
                {formatDuration(compositionEditor.timeline.loopRange.endMs)}
              </span>
            </div>
            <div className="timeline-ruler" aria-label="Timeline ruler">
              {compositionEditor.timeline.gridLabels.map((label, index) => (
                <span key={index}>{label}</span>
              ))}
            </div>
            {compositionEditor.tracks.map((track) => (
              <article className="timeline-track" key={track.trackId}>
                <div className="track-strip">
                  <strong>{track.name}</strong>
                  <small>
                    {statusLabel(track.role)} / {track.gainDb} dB / pan{" "}
                    {track.pan}
                  </small>
                  <button
                    type="button"
                    className={track.muted ? "secondary-action is-active" : "secondary-action"}
                    onClick={() =>
                      updateCompositionTrack(track.trackId, {
                        muted: !track.muted,
                      })
                    }
                    title="Toggle saved track mute"
                  >
                    {track.muted ? "Muted" : "Mute"}
                  </button>
                  <button
                    type="button"
                    className={track.soloed ? "secondary-action is-active" : "secondary-action"}
                    onClick={() =>
                      updateCompositionTrack(track.trackId, {
                        soloed: !track.soloed,
                      })
                    }
                    title="Toggle saved track solo"
                  >
                    {track.soloed ? "Soloed" : "Solo"}
                  </button>
                  <div className="track-gain-controls" aria-label={`${track.name} gain`}>
                    <button
                      type="button"
                      className="secondary-action"
                      onClick={() =>
                        updateCompositionTrack(track.trackId, {
                          gainDb: Math.max(-60, track.gainDb - 1),
                        })
                      }
                    >
                      -1 dB
                    </button>
                    <button
                      type="button"
                      className="secondary-action"
                      onClick={() =>
                        updateCompositionTrack(track.trackId, {
                          gainDb: Math.min(12, track.gainDb + 1),
                        })
                      }
                    >
                      +1 dB
                    </button>
                  </div>
                </div>
                <div className="clip-lane">
                  {track.clips.map((clip) => (
                    <button
                      className={
                        clip.clipId === selectedClip?.clipId
                          ? "timeline-clip selected"
                          : "timeline-clip"
                      }
                      key={clip.clipId}
                      type="button"
                      onClick={() => setSelectedClipId(clip.clipId)}
                      style={{
                        marginLeft: `${Math.min(
                          68,
                          clip.timelineStartMs / 420,
                        )}%`,
                        width: `${Math.max(
                          16,
                          Math.min(
                            38,
                            (clip.sourceRange.endMs -
                              clip.sourceRange.startMs) /
                              520,
                          ),
                        )}%`,
                      }}
                      title={clip.assetName}
                    >
                      <strong>{clip.assetName}</strong>
                      <span>{statusLabel(clip.assetKind)}</span>
                    </button>
                  ))}
                </div>
              </article>
            ))}
          </section>

          <MainSurface ariaLabel="Editor validation">
            <SectionHeading
              title="Validation"
              eyebrow={compositionEditor.validationChecks.length}
            />
            <ol className="voice-checks">
              {compositionEditor.validationChecks.map((check) => (
                <li
                  className={check.passed ? "passed" : "failed"}
                  key={check.id}
                >
                  <ClipboardCheck aria-hidden="true" size={16} />
                  <span>{check.summary}</span>
                </li>
              ))}
            </ol>
          </MainSurface>
        </div>

        <div className="composition-side">
          <MainSurface ariaLabel="Timeline assets">
            <SectionHeading
              title="Asset bin"
              eyebrow={compositionEditor.assetBin.length}
            />
            <div className="asset-bin-list">
              {compositionEditor.assetBin.map((asset) => (
                <article key={asset.assetId}>
                  <strong>{asset.name}</strong>
                  <small>
                    {statusLabel(asset.kind)} / {scopeLabel(asset.scope)}
                  </small>
                  <div className="asset-tag-row">
                    <span>{formatDuration(asset.durationMs)}</span>
                    <span>{statusLabel(asset.sourceWorkflow)}</span>
                    {asset.draggableToTimeline ? <span>placeable</span> : null}
                  </div>
                  <button
                    type="button"
                    className="secondary-action"
                    disabled={!selectedTrackId || !asset.draggableToTimeline}
                    onClick={() =>
                      selectedTrackId && addCompositionClipToTrack(selectedTrackId, asset)
                    }
                    title="Add this asset to the selected track"
                  >
                    <Plus aria-hidden="true" size={14} />
                    <span>Add</span>
                  </button>
                </article>
              ))}
            </div>
          </MainSurface>

          <MainSurface ariaLabel="Mixer state">
            <SectionHeading
              title="Mixer"
              eyebrow={`${compositionEditor.mixer.targetLufs} LUFS`}
            />
            <p>{compositionEditor.mixer.loudnessCheck}</p>
            <div className="mixer-list">
              {compositionEditor.mixer.trackStates.map((track) => (
                <article key={track.trackId}>
                  <strong>{track.label}</strong>
                  <small>
                    {track.gainDb} dB / pan {track.pan}
                  </small>
                  <div className="asset-tag-row">
                    {track.effectChain.map((effect, index) => (
                      <span key={`${track.trackId}-effect-${index}`}>
                        {effect}
                      </span>
                    ))}
                    {track.sendTargets.map((send, index) => (
                      <span key={`${track.trackId}-send-${index}`}>{send}</span>
                    ))}
                  </div>
                </article>
              ))}
            </div>
          </MainSurface>
        </div>
      </div>

      <div className="composition-bottom-grid">
        <MainSurface ariaLabel="Generated asset flows">
          <SectionHeading
            title="Studio flows"
            eyebrow={compositionEditor.sourceFlows.length}
          />
          <ol className="voice-checks">
            {compositionEditor.sourceFlows.map((flow) => (
              <li
                className={flow.status === "ready" ? "passed" : "failed"}
                key={`${flow.workflow}-${flow.assetKind}`}
              >
                <CircleCheck aria-hidden="true" size={16} />
                <span>
                  <strong>{flow.label}</strong> {statusLabel(flow.assetKind)}
                </span>
              </li>
            ))}
          </ol>
        </MainSurface>

        <MainSurface ariaLabel="Render plan">
          <SectionHeading
            title="Render plan"
            eyebrow={
              compositionEditor.exportPlan.canRenderMixdown
                ? "ready"
                : "blocked"
            }
          />
          <p>{compositionEditor.exportPlan.mixdownPath}</p>
          <div className="asset-tag-row detail-tags">
            {compositionEditor.exportPlan.presetIds.map((preset, index) => (
              <span key={index}>{preset}</span>
            ))}
          </div>
          <small>{compositionEditor.exportPlan.sceneWorksWarning}</small>
        </MainSurface>

        <MainSurface ariaLabel="Editor component decision">
          <SectionHeading
            title="Component decision"
            eyebrow={overview.compositionEditor.recommendedComponentId}
          />
          <div className="component-decision-list">
            {compositionEditor.componentDecisions.map((decision) => (
              <article
                className={
                  decision.fit === "strong-prototype-candidate"
                    ? "recommended"
                    : ""
                }
                key={decision.id}
              >
                <strong>{decision.name}</strong>
                <small>
                  {decision.license} / {statusLabel(decision.fit)}
                </small>
                <p>{decision.prototypeEvidence}</p>
                <p>{decision.decision}</p>
                {isHttpUrl(decision.sourceUrl) ? (
                  <a
                    href={decision.sourceUrl}
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    {decision.sourceUrl}
                  </a>
                ) : (
                  <span>{decision.sourceUrl}</span>
                )}
              </article>
            ))}
          </div>
        </MainSurface>
      </div>
    </section>
  );
}
