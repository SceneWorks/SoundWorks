// DR-02: Waveform Review studio. Extracted from App.tsx and rebuilt on the
// shared grammar (SurfaceHeader hero + HeroStat for the header/metrics,
// MainSurface + SectionHeading for the side panels) in place of the bespoke
// samples-header / samples-metrics / tts-subpanel / subpanel-heading classes.
// All save wiring, gating, preview wiring, and data bindings are preserved
// verbatim, including the F-015 is-inert edit-action demotions.
import {
  CircleCheck,
  ClipboardCheck,
  Play,
  Save,
  SlidersHorizontal,
} from "lucide-react";
import {
  FeedbackLine,
  HeroStat,
  MainSurface,
  PlaybackControl,
  SectionHeading,
  SurfaceHeader,
} from "../components";
import { formatDuration, statusLabel } from "../viewModel";
import { useAppContext } from "./context";

export function ReviewScreen() {
  const {
    reviewWorkspace,
    overview,
    assetLibrary,
    libraryPlayback,
    reviewActionStatus,
    saveSelectedReviewEdit,
    previewLibraryItem,
  } = useAppContext();

  return (
    <section className="review-workspace-panel" aria-label="Waveform Review">
      <SurfaceHeader
        eyebrow="Waveform Review"
        title={reviewWorkspace.selectedAsset.asset.name}
        actions={
          <button
            className="primary-action review-action"
            disabled={!reviewWorkspace.editSubmission.canSave}
            onClick={saveSelectedReviewEdit}
            type="button"
            title="Save edited audio version"
          >
            <Save aria-hidden="true" size={18} />
            <span>
              {reviewWorkspace.editSubmission.canSave
                ? "Save version"
                : "Blocked"}
            </span>
          </button>
        }
        stats={
          <>
            <HeroStat
              label="Assets"
              value={overview.reviewWorkspace.assetCount}
            />
            <HeroStat
              label="Previewable"
              value={overview.reviewWorkspace.previewableAssetCount}
            />
            <HeroStat
              label="Edit actions"
              value={overview.reviewWorkspace.editActionCount}
            />
            <HeroStat
              label="Comparison"
              value={overview.reviewWorkspace.comparisonCount}
            />
          </>
        }
      />
      <FeedbackLine feedback={reviewActionStatus} />
      <PlaybackControl
        playback={libraryPlayback}
        className="review-audio-preview"
      />

      <div className="review-layout">
        <div className="review-main">
          <section className="review-transport" aria-label="Waveform transport">
            <div className="review-transport-topline">
              <button
                className="icon-control"
                disabled={!assetLibrary.selectedItem}
                onClick={() => {
                  if (assetLibrary.selectedItem) {
                    previewLibraryItem(assetLibrary.selectedItem.item.id);
                  }
                }}
                type="button"
                title="Play or pause preview"
              >
                <Play aria-hidden="true" size={18} />
              </button>
              <strong>
                {formatDuration(reviewWorkspace.transport.positionMs)} /{" "}
                {formatDuration(reviewWorkspace.transport.durationMs)}
              </strong>
              <span>{reviewWorkspace.transport.zoomPixelsPerSecond}px/s</span>
            </div>
            <div
              className="waveform-strip"
              aria-label="Cached waveform preview"
            >
              {reviewWorkspace.waveform.peaks.map((peak, index) => (
                <span
                  aria-hidden="true"
                  className="waveform-bar"
                  key={index}
                  style={{ height: `${Math.max(20, peak.max * 86)}%` }}
                />
              ))}
            </div>
            <div className="transport-meta">
              <span>
                selection{" "}
                {formatDuration(
                  reviewWorkspace.transport.selection?.startMs ?? 0,
                )}
                -
                {formatDuration(
                  reviewWorkspace.transport.selection?.endMs ?? 0,
                )}
              </span>
              <span>
                loop{" "}
                {formatDuration(
                  reviewWorkspace.transport.loopRegion?.startMs ?? 0,
                )}
                -
                {formatDuration(
                  reviewWorkspace.transport.loopRegion?.endMs ?? 0,
                )}
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
                  {statusLabel(asset.sourceWorkflow)} / {asset.versions.length}{" "}
                  version
                </small>
                <p>
                  {asset.canPreview
                    ? "waveform and spectrogram cached"
                    : "preview pending"}
                </p>
              </article>
            ))}
          </div>

          <div
            className="edit-action-grid"
            aria-label="Lightweight edit actions"
          >
            {reviewWorkspace.editActions.map((action) => (
              <div
                className={
                  action.enabled
                    ? "edit-action enabled is-inert"
                    : "edit-action is-inert"
                }
                key={action.id}
                title={action.label}
                aria-disabled="true"
              >
                <SlidersHorizontal aria-hidden="true" size={16} />
                <span>{action.label}</span>
              </div>
            ))}
          </div>
        </div>

        <div className="review-side">
          <MainSurface ariaLabel="Version comparison">
            <SectionHeading
              title="Version comparison"
              eyebrow={reviewWorkspace.versionComparison.mode}
            />
            <div className="comparison-grid">
              {[
                reviewWorkspace.versionComparison.left,
                reviewWorkspace.versionComparison.right,
              ].map((side) => (
                <article key={side.versionId}>
                  <strong>{side.label}</strong>
                  <small>{side.versionId}</small>
                  <p>
                    {formatDuration(side.durationMs)} / {side.loudnessLufs} LUFS
                    / {side.truePeakDbfs} dBTP
                  </p>
                </article>
              ))}
            </div>
            <div className="comparison-metrics">
              <span>
                {reviewWorkspace.versionComparison.metrics.durationDeltaMs}
                ms
              </span>
              <span>
                {reviewWorkspace.versionComparison.metrics.loudnessDeltaLufs}{" "}
                LUFS
              </span>
              <span>
                diff{" "}
                {
                  reviewWorkspace.versionComparison.metrics
                    .waveformDifferenceScore
                }
              </span>
            </div>
          </MainSurface>

          <MainSurface ariaLabel="Edited version">
            <SectionHeading
              title="Edited version"
              eyebrow={reviewWorkspace.editSubmission.job.status}
            />
            <div className="output-card">
              <strong>{reviewWorkspace.editSubmission.savedVersion.id}</strong>
              <small>
                v{reviewWorkspace.editSubmission.savedVersion.versionIndex} /{" "}
                {reviewWorkspace.editSubmission.savedVersion.file.format}
              </small>
              <p>
                {reviewWorkspace.editSubmission.savedVersion.file.storagePath}
              </p>
            </div>
          </MainSurface>

          <MainSurface ariaLabel="Recipe provenance">
            <SectionHeading
              title="Provenance"
              eyebrow={
                reviewWorkspace.provenance.inspectable
                  ? "inspectable"
                  : "blocked"
              }
            />
            <div className="output-card">
              <strong>{reviewWorkspace.provenance.editRecipe.id}</strong>
              <small>
                {statusLabel(
                  reviewWorkspace.provenance.originalRecipe.workflow,
                )}{" "}
                to {statusLabel(reviewWorkspace.provenance.editRecipe.workflow)}
              </small>
              <p>{reviewWorkspace.provenance.sidecarPath}</p>
            </div>
          </MainSurface>
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
  );
}
