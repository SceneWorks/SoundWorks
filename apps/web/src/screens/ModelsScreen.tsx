// DR-02: Models (Model Manager) screen. Extracted from App.tsx to complete the
// App.tsx decomposition (F-010). For this slice it is a faithful extraction of
// the existing model-manager panel (outer .panel -> MainSurface + SectionHeading;
// install/revalidate wiring preserved verbatim). DR-03 rebuilds the internals
// onto the model-type-group + model-card + WorkerProgressCard grammar with an
// honest install state. The Provider Coverage + Evaluation Scorecard panels that
// previously shared this surface now live on Settings (DR-03 "move off Models").
import {
  CircleAlert,
  CircleCheck,
  Download,
  HardDrive,
  PackageCheck,
} from "lucide-react";
import { MainSurface, SectionHeading } from "../components";
import { statusLabel, workflowLabel } from "../viewModel";
import { useAppContext } from "./context";

export function ModelsScreen() {
  const { modelManager, modelManagerOperation, runModelManagerAction } =
    useAppContext();

  return (
    <section className="system-grid" aria-label="Model manager">
      <MainSurface className="model-manager-panel">
        <SectionHeading
          title="Model Manager"
          eyebrow={`${modelManager.summary.candidateCount} candidates`}
        />
        <div className="runtime-summary" aria-label="Model manager status">
          <div>
            <PackageCheck aria-hidden="true" size={18} />
            <strong>{modelManager.summary.verifiedInstalledCount}</strong>
            <span>verified</span>
          </div>
          <div>
            <Download aria-hidden="true" size={18} />
            <strong>{modelManager.summary.installableCount}</strong>
            <span>installable</span>
          </div>
          <div>
            <HardDrive aria-hidden="true" size={18} />
            <strong>{modelManager.summary.missingCacheCount}</strong>
            <span>missing cache</span>
          </div>
          <div>
            <CircleAlert aria-hidden="true" size={18} />
            <strong>{modelManager.summary.failedOperationCount}</strong>
            <span>failed ops</span>
          </div>
        </div>

        <div className="runtime-policy">
          <strong>{modelManager.cacheRoot}</strong>
          <span>
            No model is installed until required files verify on disk.
          </span>
          {modelManagerOperation ? (
            <small>
              {statusLabel(modelManagerOperation.action)}:{" "}
              {statusLabel(modelManagerOperation.status)}
            </small>
          ) : null}
        </div>

        <div className="model-manager-grid">
          <div className="runtime-stack">
            <h3>Lane readiness</h3>
            <ol className="runtime-list">
              {modelManager.laneReadiness.map((lane) => (
                <li key={`${lane.lane}-${lane.recommendedCandidateId}`}>
                  <span className={`runtime-dot ${lane.state}`} />
                  <div>
                    <strong>{workflowLabel(lane.lane)}</strong>
                    <small>
                      {lane.recommendedCandidateId} / {statusLabel(lane.state)}
                    </small>
                    <em>{lane.summary}</em>
                    {lane.blocker ? <em>{lane.blocker}</em> : null}
                  </div>
                </li>
              ))}
            </ol>
          </div>

          <div className="runtime-stack">
            <h3>Candidate cache</h3>
            <ol className="runtime-list model-cache-list">
              {modelManager.candidates.slice(0, 10).map((candidate) => (
                <li key={candidate.candidateId}>
                  <span className={`runtime-dot ${candidate.installState}`} />
                  <div>
                    <strong>{candidate.name}</strong>
                    <small>
                      {candidate.candidateId} /{" "}
                      {statusLabel(candidate.installState)} /{" "}
                      {candidate.cache.presentFileCount} of{" "}
                      {candidate.cache.expectedFileCount}
                    </small>
                    <em>{candidate.cache.evidence}</em>
                    {candidate.cache.missingRequiredFiles[0] ? (
                      <em>
                        missing{" "}
                        {candidate.cache.missingRequiredFiles.join(", ")}
                      </em>
                    ) : null}
                    <div className="model-manager-actions">
                      <button
                        className="icon-button small"
                        disabled={!candidate.actions.includes("install")}
                        onClick={() =>
                          runModelManagerAction(candidate.candidateId, "install")
                        }
                        title={`Install ${candidate.name}`}
                        type="button"
                      >
                        <Download aria-hidden="true" size={15} />
                      </button>
                      <button
                        className="icon-button small"
                        onClick={() =>
                          runModelManagerAction(
                            candidate.candidateId,
                            "revalidate",
                          )
                        }
                        title={`Revalidate ${candidate.name}`}
                        type="button"
                      >
                        <CircleCheck aria-hidden="true" size={15} />
                      </button>
                    </div>
                  </div>
                </li>
              ))}
            </ol>
          </div>
        </div>

        {modelManagerOperation ? (
          <div className={`operation-banner ${modelManagerOperation.status}`}>
            <strong>{modelManagerOperation.summary}</strong>
            {modelManagerOperation.recovery ? (
              <span>{modelManagerOperation.recovery}</span>
            ) : null}
          </div>
        ) : null}

        <ol className="validation-list" aria-label="Model manager checks">
          {modelManager.validationChecks.map((check) => (
            <li className={check.passed ? "passed" : "failed"} key={check.id}>
              <CircleCheck aria-hidden="true" size={16} />
              <span>
                {check.summary}
                {check.recovery ? <em>{check.recovery}</em> : null}
              </span>
            </li>
          ))}
        </ol>
      </MainSurface>
    </section>
  );
}
