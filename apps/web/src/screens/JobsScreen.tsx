// DR-02: Runtime Jobs screen (Worker Runtime). Extracted from App.tsx and
// rebuilt on the shared grammar (MainSurface + SectionHeading) in place of the
// bespoke .panel / .panel-heading wrappers. Cancel/Retry wiring + gating and all
// data bindings are preserved verbatim.
import { useState } from "react";
import {
  CircleAlert,
  CircleCheck,
  Cpu,
  HardDrive,
  PackageCheck,
} from "lucide-react";
import { MainSurface, SectionHeading, SegmentedControl } from "../components";
import { formatMb, statusLabel } from "../viewModel";
import { useAppContext } from "./context";

type JobFilter = "all" | "active" | "succeeded" | "failed";

const JOB_FILTERS: ReadonlyArray<{ value: JobFilter; label: string }> = [
  { value: "all", label: "All" },
  { value: "active", label: "Active" },
  { value: "succeeded", label: "Succeeded" },
  { value: "failed", label: "Failed" },
];

export function JobsScreen() {
  const { runtime, cancelRuntimeOperation, retryRuntimeOperation } =
    useAppContext();
  const [filter, setFilter] = useState<JobFilter>("all");

  const visibleJobs = runtime.jobs.filter((job) => {
    if (filter === "all") return true;
    if (filter === "active")
      return job.status === "queued" || job.status === "running";
    if (filter === "succeeded") return job.status === "succeeded";
    return job.status === "failed" || job.status === "cancelled";
  });

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
            <div className="runtime-jobs-head">
              <h3>Jobs</h3>
              <SegmentedControl
                ariaLabel="Filter jobs by status"
                compact
                value={filter}
                onChange={setFilter}
                options={JOB_FILTERS}
              />
            </div>
            <ol className="runtime-list">
              {runtime.jobs.length === 0 ? (
                <li>
                  <span className="runtime-dot unavailable" />
                  <div>
                    <strong>No runtime jobs</strong>
                    <small>
                      Queue a generation from any studio and it will appear here
                      with live progress.
                    </small>
                  </div>
                </li>
              ) : visibleJobs.length === 0 ? (
                <li>
                  <span className="runtime-dot unavailable" />
                  <div>
                    <strong>No {filter} jobs</strong>
                    <small>Adjust the filter to see other jobs.</small>
                  </div>
                </li>
              ) : null}
              {visibleJobs.map((job) => (
                <li key={job.id}>
                  <span className={`runtime-dot ${job.status}`} />
                  <div>
                    <strong>{statusLabel(job.kind)}</strong>
                    <small>
                      {statusLabel(job.status)} /{" "}
                      {Math.round(job.progress?.percent ?? 0)}% /{" "}
                      {statusLabel(job.cancellation)}
                    </small>
                    {job.progress?.message ? (
                      <em>{job.progress.message}</em>
                    ) : null}
                    <em>{job.recordRoot}</em>
                    {job.artifacts.length > 0 ? (
                      <ul className="runtime-artifacts">
                        {job.artifacts.map((artifact, index) => (
                          <li key={index}>
                            {artifact.summary}: <code>{artifact.path}</code>
                          </li>
                        ))}
                      </ul>
                    ) : null}
                    {job.actionableError ? (
                      <em className="runtime-job-error">
                        {job.actionableError.summary}:{" "}
                        {job.actionableError.recovery}
                      </em>
                    ) : null}
                    {job.logTail.length > 0 ? (
                      <details className="runtime-log">
                        <summary>Log ({job.logTail.length})</summary>
                        <pre>{job.logTail.join("\n")}</pre>
                      </details>
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
