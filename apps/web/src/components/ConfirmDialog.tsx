// UX-F2/UX-S2: a small confirmation modal for destructive actions (reject,
// archive). Built on the shared grammar (MainSurface + SectionHeading + Toolbar).
// Renders nothing when closed; traps the action behind an explicit confirm so a
// stray click can't reject/archive an asset.
import type { ReactNode } from "react";
import { MainSurface, SectionHeading, Toolbar } from "./layout";

export function ConfirmDialog({
  open,
  title,
  message,
  confirmLabel = "Confirm",
  cancelLabel = "Cancel",
  onConfirm,
  onCancel,
}: {
  open: boolean;
  title: ReactNode;
  message: ReactNode;
  confirmLabel?: string;
  cancelLabel?: string;
  onConfirm: () => void;
  onCancel: () => void;
}) {
  if (!open) {
    return null;
  }
  return (
    <div
      className="confirm-overlay"
      role="dialog"
      aria-modal="true"
      onKeyDown={(event) => {
        if (event.key === "Escape") {
          onCancel();
        }
      }}
    >
      <MainSurface className="confirm-dialog">
        <SectionHeading title={title} />
        <p className="confirm-message">{message}</p>
        <Toolbar ariaLabel="Confirm action">
          <button type="button" className="secondary-action" onClick={onCancel}>
            {cancelLabel}
          </button>
          <button
            type="button"
            className="primary-action"
            onClick={onConfirm}
            autoFocus
          >
            {confirmLabel}
          </button>
        </Toolbar>
      </MainSurface>
    </div>
  );
}
