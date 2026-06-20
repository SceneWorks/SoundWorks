// DR-02: SoundWorks' equivalent of SceneWorks' shared <WorkerProgressCard>. It
// renders one progress/cancel/retry/error skeleton reused by every studio and
// the Models page, driven by a normalized shape (callers map their
// RuntimeJobSnapshot / ModelManagerOperation onto these props). A failed
// operation surfaces an actionable error + a caller-provided Retry/Resume
// affordance instead of a bare red banner.
import type { ReactNode } from "react";
import { StatusBadge } from "./layout";
import type { StatusTone } from "./layout";

function ProgressBar({
  percent,
  running,
}: {
  percent?: number | null;
  running?: boolean;
}) {
  const hasValue = typeof percent === "number" && Number.isFinite(percent);
  if (!hasValue && running) {
    return (
      <div
        className="progress-track indeterminate"
        role="progressbar"
        aria-label="Working"
      />
    );
  }
  const clamped = Math.max(0, Math.min(100, hasValue ? (percent as number) : 0));
  return (
    <div
      className="progress-track"
      role="progressbar"
      aria-valuenow={Math.round(clamped)}
      aria-valuemin={0}
      aria-valuemax={100}
    >
      <span style={{ width: `${clamped}%` }} />
    </div>
  );
}

export interface WorkerProgressError {
  summary: ReactNode;
  recovery?: ReactNode;
}

export function WorkerProgressCard({
  title,
  typeLabel,
  statusLabel,
  tone,
  percent,
  running = false,
  message,
  logTail,
  error,
  actions,
  className,
}: {
  title: ReactNode;
  typeLabel?: ReactNode;
  statusLabel: ReactNode;
  tone: StatusTone;
  percent?: number | null;
  running?: boolean;
  message?: ReactNode;
  logTail?: readonly string[];
  error?: WorkerProgressError | null;
  actions?: ReactNode;
  className?: string;
}) {
  return (
    <article
      className={`main-surface worker-progress-card${className ? ` ${className}` : ""}`}
    >
      <div className="worker-progress-card__head">
        <div className="worker-progress-card__title">
          {typeLabel ? (
            <span className="worker-progress-card__type">{typeLabel}</span>
          ) : null}
          <h3>{title}</h3>
        </div>
        <StatusBadge tone={tone}>{statusLabel}</StatusBadge>
      </div>

      <ProgressBar percent={percent} running={running} />

      {message ? (
        <p className="worker-progress-card__message">{message}</p>
      ) : null}

      {error ? (
        <div className="actionable-error" role="alert">
          <strong>{error.summary}</strong>
          {error.recovery ? <span>{error.recovery}</span> : null}
        </div>
      ) : null}

      {logTail && logTail.length > 0 ? (
        <pre className="worker-progress-card__log">{logTail.join("\n")}</pre>
      ) : null}

      {actions ? (
        <div className="worker-progress-card__actions">{actions}</div>
      ) : null}
    </article>
  );
}
