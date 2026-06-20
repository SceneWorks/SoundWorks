// DR-03: SoundWorks' equivalent of SceneWorks' ModelAvailabilityGate. When a
// studio has no installed+verified runtime model, instead of leaving a dead
// disabled "Blocked" control with no path forward, it shows an actionable offer
// that routes the user to the Model Manager. Renders nothing when a model is
// available, so the studio's normal controls take over.
import type { ReactNode } from "react";

export function ModelAvailabilityGate({
  installed,
  label,
  onOpenModelManager,
}: {
  installed: boolean;
  label: ReactNode;
  onOpenModelManager: () => void;
}) {
  if (installed) {
    return null;
  }

  return (
    <div className="model-availability-gate" role="status">
      <div className="model-availability-gate__copy">
        <strong>No verified {label} model installed</strong>
        <span>
          Generation is blocked until a model is installed and its cache verifies.
        </span>
      </div>
      <button
        className="primary-action"
        onClick={onOpenModelManager}
        type="button"
      >
        Open Model Manager
      </button>
    </div>
  );
}
