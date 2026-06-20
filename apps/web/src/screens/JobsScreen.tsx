// DR-02: Runtime Jobs screen (Worker Runtime). Extracted from App.tsx and
// rebuilt on the shared grammar (MainSurface + SectionHeading) in place of the
// bespoke .panel / .panel-heading wrappers. Cancel/Retry wiring + gating and all
// data bindings are preserved verbatim.
import {
  CircleAlert,
  CircleCheck,
  Cpu,
  HardDrive,
  PackageCheck,
} from "lucide-react";
import { MainSurface, SectionHeading } from "../components";
import { formatMb, statusLabel } from "../viewModel";
import { useAppContext } from "./context";

export function JobsScreen() {
  const { runtime, cancelRuntimeOperation, retryRuntimeOperation } =
    useAppContext();

  return (
    <section className="system-grid" aria-label="Runtime jobs">
      <MainSurface className="runtime-panel">
        <SectionHeading
          title="Worker Runtime"
          eyebrow={runtime.schemaVersion}
        />
        <div className="runtime-summary" aria-label="Runtime status">
          <div>
            <PackageCheck aria-hidden="true" size={18} />
            <strong>{runtime.statusCounts.installed}</strong>
            <span>verified installs</span>
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
                    <em>{model.cache.evidence}</em>
                    {model.reasons[0] ? <em>{model.reasons[0]}</em> : null}
                  </div>
                </li>
              ))}
            </ol>
          </div>

          <div className="runtime-stack">
            <h3>Jobs</h3>
            <ol className="runtime-list">
              {runtime.jobs.length === 0 ? (
                <li>
                  <span className="runtime-dot unavailable" />
                  <div>
                    <strong>No runtime jobs</strong>
                    <small>
                      Fixture/demo actions are blocked until provider execution
                      is wired.
                    </small>
                  </div>
                </li>
              ) : null}
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
                    <em>{job.recordRoot}</em>
                    {job.artifacts[0] ? (
                      <em>
                        {job.artifacts[0].summary}: {job.artifacts[0].path}
                      </em>
                    ) : null}
                    {job.actionableError ? (
                      <em>{job.actionableError.recovery}</em>
                    ) : null}
                    <div className="runtime-job-actions">
                      <button
                        className="secondary-action"
                        disabled={job.cancellation !== "cancellable"}
                        onClick={() => cancelRuntimeOperation(job.id)}
                        type="button"
                      >
                        Cancel
                      </button>
                      <button
                        className="secondary-action"
                        disabled={job.status !== "failed"}
                        onClick={() => retryRuntimeOperation(job.id)}
                        type="button"
                      >
                        Retry
                      </button>
                    </div>
                  </div>
                </li>
              ))}
            </ol>
          </div>
        </div>

        <ol className="validation-list" aria-label="Runtime checks">
          {runtime.validationChecks.map((check) => (
            <li className={check.status} key={check.id}>
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
