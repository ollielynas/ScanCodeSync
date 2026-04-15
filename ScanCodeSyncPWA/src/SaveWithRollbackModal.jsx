import { useEffect } from "react";
import { createPortal } from "react-dom";

function SaveWithRollbackModal({
  isOpen,
  rollbackSeconds,
  onClose,
  onConfirm,
  onRollbackChange,
}) {
  const rollbackSecondsRaw = Number(rollbackSeconds);
  const rollbackSecondsSafe = Number.isFinite(rollbackSecondsRaw)
    ? Math.max(0, rollbackSecondsRaw)
    : 0;
  const rollbackLandingMs = Math.max(
    0,
    Date.now() - Math.trunc(rollbackSecondsSafe * 1000),
  );
  const rollbackLandingText = new Date(rollbackLandingMs).toLocaleString();

  const addRollbackSeconds = (deltaSeconds) => {
    const currentRaw = Number(rollbackSeconds);
    const current = Number.isFinite(currentRaw) ? Math.max(0, currentRaw) : 0;
    const next = Math.max(0, current + deltaSeconds);
    onRollbackChange(String(next));
  };

  useEffect(() => {
    if (!isOpen) return;

    const originalBodyOverflow = document.body.style.overflow;
    const originalHtmlOverflow = document.documentElement.style.overflow;
    document.body.style.overflow = "hidden";
    document.documentElement.style.overflow = "hidden";

    const handleKeyDown = (event) => {
      if (event.key === "Escape") {
        onClose();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
      document.body.style.overflow = originalBodyOverflow;
      document.documentElement.style.overflow = originalHtmlOverflow;
    };
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  return createPortal(
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4"
      onClick={onClose}
    >
      <div
        className="max-h-[90dvh] w-full max-w-sm overflow-y-auto rounded-xl bg-white p-4 shadow-xl"
        onClick={(e) => e.stopPropagation()}
      >
        <h3 className="text-base font-semibold text-slate-900">
          Save with Rollback
        </h3>
        <p className="mt-1 text-sm text-slate-600">
          Apply pending metadata as if it was saved in the past.
        </p>
        <label className="mt-3 block text-sm font-medium text-slate-700">
          Rollback Seconds
          <input
            id="save_rollback_seconds_input"
            className="scs-input mt-2"
            type="number"
            min="0"
            step="1"
            value={rollbackSeconds}
            onChange={(e) => onRollbackChange(e.target.value)}
          />
        </label>
        <div className="mt-2 flex gap-2">
          <button
            className="scs-button-secondary"
            type="button"
            onClick={() => addRollbackSeconds(30)}
          >
            +30 sec
          </button>
          <button
            className="scs-button-secondary"
            type="button"
            onClick={() => addRollbackSeconds(60)}
          >
            +1 min
          </button>
        </div>
        <p className="mt-3 rounded-lg bg-slate-50 px-3 py-2 text-sm text-slate-700">
          Rollback lands on: <strong>{rollbackLandingText}</strong>
        </p>
        <div className="mt-4 flex justify-end gap-2">
          <button className="scs-button-secondary" onClick={onClose}>
            Cancel
          </button>
          <button className="scs-button" onClick={onConfirm}>
            Save
          </button>
        </div>
      </div>
    </div>,
    document.body,
  );
}

export default SaveWithRollbackModal;
