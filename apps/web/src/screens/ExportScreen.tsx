// DR-02: Export workflow screen. Extracted from App.tsx and rebuilt on the
// shared grammar (SurfaceHeader hero + HeroStat for the header/metrics,
// MainSurface + SectionHeading for the sub-panels) in place of the bespoke
// export-header / export-metrics / tts-subpanel / subpanel-heading classes.
// All Export wiring, gating, and data bindings are preserved verbatim.
import { useState } from "react";
import { CircleCheck, ClipboardCheck, Download, ShieldAlert } from "lucide-react";
import {
  ConfirmDialog,
  FeedbackLine,
  HeroStat,
  MainSurface,
  SectionHeading,
  SurfaceHeader,
} from "../components";
import { formatDuration, statusLabel } from "../viewModel";
import { useAppContext } from "./context";

export function ExportScreen() {
  const {
    exportWorkflow,
    exportSelectedLibraryItem,
    exportActionStatus,
    overview,
    setActiveView,
  } = useAppContext();

  const selected = exportWorkflow.selectedExport;
  const [presetId, setPresetId] = useState(selected.presetId);
  const [confirming, setConfirming] = useState(false);
  const chosenPreset =
    exportWorkflow.presets.find((preset) => preset.preset.id === presetId) ??
    exportWorkflow.presets.find(
      (preset) => preset.preset.id === selected.presetId,
    );
  const blockingReasons = selected.blockingReasons ?? [];

  return (
    <section className="export-workflow-panel" aria-label="Export Workflow">
      <SurfaceHeader
        eyebrow="Export"
        title="Presets, stems, and handoff packages"
        actions={
          <button
            className="primary-action export-action"
            disabled={!selected.canExport}
            onClick={() => setConfirming(true)}
            title={
              selected.canExport
                ? "Review and export selected composition"
                : "Export is blocked — resolve the rights reasons below"
            }
            type="button"
          >
            <Download aria-hidden="true" size={18} />
            <span>{selected.canExport ? "Export…" : "Blocked"}</span>
          </button>
        }
        stats={
          <>
            <HeroStat
              label="presets"
              value={overview.exportWorkflow.presetCount}
            />
            <HeroStat
              label="sidecars"
              value={overview.exportWorkflow.sidecarCount}
            />
            <HeroStat
              label="targets ready"
              value={overview.exportWorkflow.readyTargetCount}
            />
            <HeroStat
              label="formats selected"
              value={overview.exportWorkflow.selectedFormatCount}
            />
          </>
        }
      />
      <FeedbackLine feedback={exportActionStatus} />

      <div className="export-layout">
        <div className="export-preset-grid" aria-label="Export presets">
          {exportWorkflow.presets.map((preset) => (
            <button
              type="button"
              className={
                preset.preset.id === presetId
                  ? "export-preset-card selected"
                  : "export-preset-card"
              }
              key={preset.preset.id}
              onClick={() => setPresetId(preset.preset.id)}
              aria-pressed={preset.preset.id === presetId}
            >
              <div className="export-preset-topline">
                <strong>{preset.preset.name}</strong>
                <span>{statusLabel(preset.preset.target)}</span>
              </div>
              <p>{preset.description}</p>
              <div className="asset-tag-row">
                {preset.formats.map((format, index) => (
                  <span key={`${preset.preset.id}-${index}`}>{format}</span>
                ))}
                {preset.preset.includeStems ? <span>stems</span> : null}
                {preset.writesSidecar ? <span>sidecar</span> : null}
              </div>
            </button>
          ))}
        </div>

        <div className="export-detail" aria-label="Selected export detail">
          <MainSurface className="tts-subpanel">
            <SectionHeading
              title="Selected export"
              eyebrow={statusLabel(exportWorkflow.selectedExport.sourceKind)}
            />
            <p>
              {exportWorkflow.selectedExport.presetId} /{" "}
              {exportWorkflow.selectedExport.sourceId}
            </p>
            <ol className="version-list">
              {exportWorkflow.selectedExport.outputPaths.map((path, index) => (
                <li key={index}>
                  <CircleCheck aria-hidden="true" size={16} />
                  <div>
                    <strong>{path.split("/").pop()}</strong>
                    <small>{path}</small>
                  </div>
                </li>
              ))}
            </ol>
          </MainSurface>

          {blockingReasons.length > 0 ? (
            <MainSurface className="tts-subpanel export-blocked" ariaLabel="Export blocked">
              <SectionHeading
                title="Export blocked"
                eyebrow={`${blockingReasons.length} reason${blockingReasons.length === 1 ? "" : "s"}`}
              />
              <ul className="blocking-reasons">
                {blockingReasons.map((reason, index) => (
                  <li key={index}>
                    <ShieldAlert aria-hidden="true" size={16} />
                    <span>{reason}</span>
                  </li>
                ))}
              </ul>
              <button
                type="button"
                className="secondary-action"
                onClick={() => setActiveView("rights")}
                title="Open Rights & Safety to resolve consent / commercial-use"
              >
                Resolve in Rights & Safety
              </button>
            </MainSurface>
          ) : null}

          <MainSurface className="tts-subpanel">
            <SectionHeading
              title="DAW bundle"
              eyebrow={exportWorkflow.dawHandoff.stemKinds.length}
            />
            <p>{exportWorkflow.dawHandoff.packagePath}</p>
            <div className="asset-tag-row detail-tags">
              <span>zip bundle</span>
              <span>cue markers</span>
              <span>loop markers</span>
              <span>BPM/key</span>
              <span>lyrics</span>
            </div>
          </MainSurface>

          <MainSurface className="tts-subpanel">
            <SectionHeading
              title="SceneWorks handoff"
              eyebrow={formatDuration(
                exportWorkflow.sceneWorksHandoff.durationMs,
              )}
            />
            <p>{exportWorkflow.sceneWorksHandoff.packagePath}</p>
            <small>
              {exportWorkflow.sceneWorksHandoff.sampleRateHz} Hz /{" "}
              {exportWorkflow.sceneWorksHandoff.channels} channels /{" "}
              {exportWorkflow.sceneWorksHandoff.markerCount} marker /{" "}
              {statusLabel(exportWorkflow.sceneWorksHandoff.importStrategy)}
            </small>
            <div className="asset-tag-row detail-tags">
              <span>{exportWorkflow.sceneWorksHandoff.sceneWorksAssetType}</span>
              <span>{exportWorkflow.sceneWorksHandoff.sceneWorksMimeType}</span>
              <span>
                {exportWorkflow.sceneWorksHandoff.replaceExistingAudio
                  ? "replace enabled"
                  : "attach only"}
              </span>
            </div>
            <p>{exportWorkflow.sceneWorksHandoff.packageManifestPath}</p>
            <small>
              {exportWorkflow.sceneWorksHandoff.intendedProjectId} /{" "}
              {exportWorkflow.sceneWorksHandoff.intendedVideoAssetId}
            </small>
          </MainSurface>
        </div>
      </div>

      <div className="export-bottom-grid">
        <MainSurface className="tts-subpanel" ariaLabel="Export targets">
          <SectionHeading
            title="Targets"
            eyebrow={exportWorkflow.targets.length}
          />
          <ol className="voice-checks">
            {exportWorkflow.targets.map((target) => (
              <li
                className={target.ready ? "passed" : "failed"}
                key={target.target}
              >
                <CircleCheck aria-hidden="true" size={16} />
                <span>
                  <strong>{target.label}</strong> {target.notes[0]}
                </span>
              </li>
            ))}
          </ol>
        </MainSurface>

        <MainSurface className="tts-subpanel" ariaLabel="Export sidecars">
          <SectionHeading
            title="Sidecars"
            eyebrow={exportWorkflow.sidecars.length}
          />
          <div className="sidecar-list">
            {exportWorkflow.sidecars.map((sidecar) => (
              <article key={sidecar.id}>
                <strong>{sidecar.assetId}</strong>
                <small>
                  {statusLabel(sidecar.target)} / {sidecar.eventCount} events
                </small>
                <p>{sidecar.path}</p>
              </article>
            ))}
          </div>
        </MainSurface>

        <MainSurface className="tts-subpanel" ariaLabel="Export validation">
          <SectionHeading
            title="Validation"
            eyebrow={exportWorkflow.validationChecks.length}
          />
          <ol className="voice-checks">
            {exportWorkflow.validationChecks.map((check) => (
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

        <MainSurface
          className="tts-subpanel"
          ariaLabel="SceneWorks compatibility"
        >
          <SectionHeading
            title="SceneWorks compatibility"
            eyebrow={
              exportWorkflow.sceneWorksHandoff.compatibilityChecks.length
            }
          />
          <ol className="voice-checks">
            {exportWorkflow.sceneWorksHandoff.compatibilityChecks.map(
              (check) => (
                <li
                  className={check.status === "blocked" ? "failed" : "passed"}
                  key={check.id}
                >
                  <ClipboardCheck aria-hidden="true" size={16} />
                  <span>
                    <strong>{statusLabel(check.status)}</strong> {check.summary}
                  </span>
                </li>
              ),
            )}
          </ol>
        </MainSurface>

        <MainSurface
          className="tts-subpanel"
          ariaLabel="SceneWorks attachment steps"
        >
          <SectionHeading
            title="SceneWorks attachment"
            eyebrow={exportWorkflow.sceneWorksHandoff.attachmentSteps.length}
          />
          <ol className="version-list">
            {exportWorkflow.sceneWorksHandoff.attachmentSteps.map((step) => (
              <li key={step.id}>
                <CircleCheck aria-hidden="true" size={16} />
                <div>
                  <strong>{step.label}</strong>
                  <small>
                    {step.source}
                    {" -> "}
                    {step.target}
                  </small>
                </div>
              </li>
            ))}
          </ol>
        </MainSurface>
      </div>

      <ConfirmDialog
        open={confirming}
        title="Export this composition?"
        message={`Export “${chosenPreset?.preset.name ?? selected.presetId}” (${(
          chosenPreset?.formats ?? selected.formats
        ).join(", ")}) for ${selected.sourceId}. ${selected.outputPaths.length} file(s) will be written.`}
        confirmLabel="Export"
        onCancel={() => setConfirming(false)}
        onConfirm={() => {
          exportSelectedLibraryItem(
            chosenPreset
              ? { presetId: chosenPreset.preset.id, formats: chosenPreset.formats }
              : undefined,
          );
          setConfirming(false);
        }}
      />
    </section>
  );
}
