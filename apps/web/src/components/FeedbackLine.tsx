// UX-F2: renders an ActionFeedback tri-state as a status line. Maps each kind to
// a StatusBadge tone so pending/success/error are visually distinct, and is an
// aria-live region so the outcome is announced. Replaces the bare
// `<p className="action-status">{string}</p>` every screen used.
import type { ActionFeedback, ActionFeedbackKind } from "../viewModel";
import { StatusBadge } from "./layout";
import type { StatusTone } from "./layout";

const TONE: Record<ActionFeedbackKind, StatusTone> = {
  idle: "neutral",
  pending: "warning",
  success: "completed",
  error: "failed",
};

const BADGE_LABEL: Record<ActionFeedbackKind, string> = {
  idle: "ready",
  pending: "working",
  success: "done",
  error: "error",
};

export function FeedbackLine({
  feedback,
  className,
}: {
  feedback: ActionFeedback;
  className?: string;
}) {
  if (!feedback.message) {
    return null;
  }
  return (
    <p
      className={["action-status", `action-status-${feedback.kind}`, className]
        .filter(Boolean)
        .join(" ")}
      role="status"
      aria-live="polite"
    >
      {feedback.kind !== "idle" ? (
        <StatusBadge tone={TONE[feedback.kind]}>
          {BADGE_LABEL[feedback.kind]}
        </StatusBadge>
      ) : null}
      <span>{feedback.message}</span>
    </p>
  );
}
