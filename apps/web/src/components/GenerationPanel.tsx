// UX-S1a: shared studio generation panel. Renders the active runtime job for a
// studio — live progress (from the UX-F1 poll), cancel/retry, the actionable
// error on failure, and an on-success result slot (inline playback) — by mapping
// the polled RuntimeJobSnapshot onto the shared WorkerProgressCard. Every studio
// (TTS/SFX/Samples/Songs/Voice/Video) renders this with the workflow(s) it owns,
// so the queued->running->done loop looks identical everywhere.
import type { ReactNode } from "react";
import type { CapabilityWorkflow, RuntimeJobSnapshot } from "../types";
import { workflowLabel } from "../viewModel";
import { WorkerProgressCard } from "./WorkerProgressCard";
import type { StatusTone } from "./layout";

const JOB_TONE: Record<string, StatusTone> = {
  queued: "neutral",
  running: "warning",
  succeeded: "completed",
  failed: "failed",
  cancelled: "canceled",
};

export function GenerationPanel({
  job,
  workflows,
  typeLabel,
  onCancel,
  onRetry,
  children,
}: {
  job: RuntimeJobSnapshot | null;
  workflows: readonly CapabilityWorkflow[];
  typeLabel?: ReactNode;
  onCancel: (jobId: string) => void;
  onRetry: (jobId: string) => void;
  /** Inline result (e.g. <PlaybackControl/>) rendered only on success. */
  children?: ReactNode;
}) {
  if (!job || !workflows.includes(job.workflow)) {
    return null;
  }
  const running = job.status === "queued" || job.status === "running";
  const failed = job.status === "failed";
  const succeeded = job.status === "succeeded";
  return (
    <div className="generation-panel">
      <WorkerProgressCard
        title={`${workflowLabel(job.workflow)} job`}
        typeLabel={typeLabel}
        statusLabel={job.status}
        tone={JOB_TONE[job.status] ?? "neutral"}
        percent={job.progress?.percent ?? null}
        running={running}
        message={job.progress?.message ?? undefined}
        logTail={job.logTail}
        error={
          failed && job.actionableError
            ? {
                summary: job.actionableError.summary,
                recovery: job.actionableError.recovery,
              }
            : null
        }
        actions={
          running || failed ? (
            <>
              {running ? (
                <button
                  type="button"
                  className="secondary-action"
                  onClick={() => onCancel(job.id)}
                >
                  Cancel
                </button>
              ) : null}
              {failed ? (
                <button
                  type="button"
                  className="secondary-action"
                  onClick={() => onRetry(job.id)}
                >
                  Retry
                </button>
              ) : null}
            </>
          ) : null
        }
      />
      {succeeded && children ? (
        <div className="generation-result">{children}</div>
      ) : null}
    </div>
  );
}
