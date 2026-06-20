// DR-02: Multitrack composition editor. Extracted from App.tsx and rebuilt on
// the shared grammar (SurfaceHeader hero + HeroStat for the header/metrics,
// MainSurface + SectionHeading for the sub-panels) in place of the bespoke
// composition-header / composition-metrics / tts-subpanel / subpanel-heading
// classes. The F-015 inert demotions (render action, tools, clips) stay
// non-button elements and all data bindings are preserved verbatim.
import { CircleCheck, ClipboardCheck, Disc3 } from "lucide-react";
import {
  HeroStat,
  MainSurface,
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
  const { compositionEditor, overview } = useAppContext();

  return (
    <section
      className="composition-editor-panel"
      aria-label="Multitrack Composition Editor"
    >
      <SurfaceHeader
        eyebrow="Multitrack editor"
        title={compositionEditor.composition.name}
        actions={
          <div
            className={
              compositionEditor.exportPlan.canRenderMixdown
                ? "primary-action composition-action is-inert"
                : "primary-action composition-action is-inert is-blocked"
            }
            title="Composition mixdown render is not available in this build yet"
            aria-disabled="true"
          >
            <Disc3 aria-hidden="true" size={18} />
            <span>
              {compositionEditor.exportPlan.canRenderMixdown
                ? "Render (preview)"
                : "Blocked"}
            </span>
          </div>
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

      <div className="composition-layout">
        <div className="composition-main">
          <section className="composition-toolbar" aria-label="Editor tools">
            {compositionEditor.tools.map((tool) => (
              <div
                className={
                  tool.id === compositionEditor.timeline.selectedTool
                    ? "tool-button selected is-inert"
                    : "tool-button is-inert"
                }
                key={tool.id}
                title={tool.label}
                aria-disabled="true"
              >
                <span>{tool.label}</span>
              </div>
            ))}
          </section>

          <section className="timeline-board" aria-label="Timeline tracks">
            <div className="timeline-selection" aria-label="Timeline selection">
              <span>{compositionEditor.timeline.selectedClipId}</span>
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
                  <span>
                    {track.muted ? "Muted" : "Live"} /{" "}
                    {track.soloed ? "Solo" : "Mix"}
                  </span>
                </div>
                <div className="clip-lane">
                  {track.clips.map((clip) => (
                    <div
                      className={
                        clip.clipId ===
                        compositionEditor.timeline.selectedClipId
                          ? "timeline-clip selected is-inert"
                          : "timeline-clip is-inert"
                      }
                      key={clip.clipId}
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
                      aria-disabled="true"
                    >
                      <strong>{clip.assetName}</strong>
                      <span>{statusLabel(clip.assetKind)}</span>
                    </div>
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
