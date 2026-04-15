import { useRef, useState } from "react";
import { useCookies } from "react-cookie";
import "./App.css";
// import BarcodeDisplay from "./Barcode";
import { getOrCreateDeviceId } from "./device_id";
import { fskTransmit } from "./play_audio";
import InstallPWA from "./Pwa";
import QrMetadataDisplay from "./QRCode";
import SaveWithRollbackModal from "./SaveWithRollbackModal";
import TakePhotoScan from "./Scan";
import TimeQrDisplay from "./TimeQrDisplay";
const DEVICE_ID = getOrCreateDeviceId();

function App() {
  const [mobilePanel, setMobilePanel] = useState("controls");
  const [showRollbackPopup, setShowRollbackPopup] = useState(false);

  const cookieKeys = [
    // this key is deprecated
    "isMaster",

    "isDirector",
    "isOperator",
    "productionName",
    "enableOperatorName",
    "operatorName",
    "enableSceneName",
    "sceneName",
    "enableTakeNumber",
    "takeNumber",
    // above is the values that will later become a part of the database
    "pendingChanges",
    "changeLog",
    "renameDevice",
    "saveRollbackSeconds",
  ];

  const [cookies, setCookie] = useCookies(cookieKeys);
  const audioCtxRef = useRef(null);

  const cookieOptions = { path: "/", maxAge: 60 * 60 * 24 * 365 };

  const set = (key, value) => {
    const pending = cookies.pendingChanges ?? {};
    setCookie("pendingChanges", { ...pending, [key]: value }, cookieOptions);
    setCookie(key, value, cookieOptions);
  };

  const setLocal = (key, value) => {
    setCookie(key, value, cookieOptions);
  };

  const dismissKeyboard = () => {
    const active = document.activeElement;
    if (active instanceof HTMLElement) {
      active.blur();
    }
    if (navigator.virtualKeyboard?.hide) {
      navigator.virtualKeyboard.hide();
    }
  };

  const savePendingChanges = (rollbackSecondsInput = 0) => {
    const pending = cookies.pendingChanges ?? {};
    if (Object.keys(pending).length === 0) return;

    const rollbackSecondsRaw = Number(rollbackSecondsInput);
    const rollbackSeconds = Number.isFinite(rollbackSecondsRaw)
      ? Math.max(0, rollbackSecondsRaw)
      : 0;
    const rollbackMs = Math.trunc(rollbackSeconds * 1000);
    const effectiveNowMs = Math.max(0, Date.now() - rollbackMs);
    const timestamp = String(BigInt(effectiveNowMs));
    const newRows = Object.entries(pending)
      .map(([key, value]) => `${DEVICE_ID},${timestamp},${key},${value}`)
      .join("\n");

    const existing = cookies.changeLog ?? "";
    const updated = existing ? `${existing}\n${newRows}` : newRows;
    setCookie("changeLog", updated, cookieOptions);
    setCookie("pendingChanges", {}, cookieOptions);
  };

  const handleSave = () => {
    dismissKeyboard();
    savePendingChanges(0);
  };

  const handleSaveWithRollback = () => {
    dismissKeyboard();
    const rollbackSecondsRaw = Number(cookies.saveRollbackSeconds ?? 0);
    const rollbackSeconds = Number.isFinite(rollbackSecondsRaw)
      ? Math.max(0, rollbackSecondsRaw)
      : 0;
    setLocal("saveRollbackSeconds", String(rollbackSeconds));
    savePendingChanges(rollbackSeconds);
    setShowRollbackPopup(false);
  };

  const openRollbackPopup = () => {
    dismissKeyboard();
    setShowRollbackPopup(true);
  };

  const closeRollbackPopup = () => {
    dismissKeyboard();
    setShowRollbackPopup(false);
  };

  const handleDownloadCSV = () => {
    const log = cookies.changeLog ?? "";
    const csv = `${log}`;
    const blob = new Blob([csv], { type: "text/csv" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = "changes.csv";
    a.click();
    URL.revokeObjectURL(url);
  };

  const getBool = (key) => cookies[key] === true || cookies[key] === "true";

  // const isMaster = getBool("isMaster");
  const isDirector = getBool("isDirector");
  const isOperator = getBool("isOperator");
  const enableOperatorName = getBool("enableOperatorName");
  const enableSceneName = getBool("enableSceneName");
  const enableTakeNumber = getBool("enableTakeNumber");
  const productionName = cookies.productionName ?? "Production Name";
  const sceneName = cookies.sceneName ?? "Scene Name";
  const takeNumber = cookies.takeNumber ?? 1;

  const renameDevice = cookies.renameDevice ?? "";
  const saveRollbackSeconds = cookies.saveRollbackSeconds ?? "30";

  const hasPending = Object.keys(cookies.pendingChanges ?? {}).length > 0;
  const hasLog = !!(cookies.changeLog ?? "");

  console.log(DEVICE_ID);

  return (
    <main className="mx-auto flex h-dvh w-full max-w-6xl flex-col gap-3 overflow-hidden p-3 md:min-h-screen md:gap-6 md:overflow-visible md:p-8">
      <header className="scs-card p-3 md:p-6">
        <p className="mb-2 text-xs font-semibold uppercase tracking-[0.2em] text-brand-700">
          ScanCodeSync
        </p>
        <h1 className="text-xl font-semibold text-slate-900 md:text-3xl">
          Production Metadata Console
        </h1>
        <p>device id: {DEVICE_ID}</p>
      </header>

      <section className="flex min-h-0 flex-1 flex-col gap-3 md:grid md:grid-cols-2 md:gap-6">
        <div className="grid grid-cols-2 gap-2 md:hidden">
          <button
            className={`rounded-xl px-3 py-2 text-sm font-semibold transition ${
              mobilePanel === "controls"
                ? "bg-slate-900 text-white"
                : "bg-white text-slate-700"
            }`}
            onClick={() => setMobilePanel("controls")}
          >
            Controls
          </button>
          <button
            className={`rounded-xl px-3 py-2 text-sm font-semibold transition ${
              mobilePanel === "actions"
                ? "bg-slate-900 text-white"
                : "bg-white text-slate-700"
            }`}
            onClick={() => setMobilePanel("actions")}
          >
            Actions
          </button>
        </div>

        <article
          className={`scs-card scs-controls-panel min-h-0 space-y-3 overflow-auto p-3 md:block md:space-y-4 md:p-6 ${
            mobilePanel === "controls" ? "block" : "hidden"
          }`}
        >
          <h2 className="text-lg font-semibold text-slate-900">Device Role</h2>
          {/* <label id="is_master_label" className="flex items-start gap-3">
            <input
              type="checkbox"
              id="is_master_checkbox"
              className="mt-1 h-4 w-4 accent-brand-500"
              checked={isMaster}
              onChange={(e) => set("isMaster", e.target.checked)}
            />
            <span>
              <span className="block font-medium text-slate-900">
                Master Clock
              </span>
              <span className="text-sm text-slate-600">
                Enable only on one trusted timing source.
              </span>
            </span>
          </label>*/}

          {/* {isMaster && (
            <p
              id="is_master_warning"
              className="rounded-lg bg-amber-50 p-3 text-sm text-amber-800"
            >
              Caution: only one master clock should be active.
            </p>
          )}*/}

          <label id="is_director_label" className="flex items-start gap-3">
            <input
              id="is_director_checkbox"
              type="checkbox"
              className="mt-1 h-4 w-4 accent-brand-500"
              checked={isDirector}
              onChange={(e) => set("isDirector", e.target.checked)}
            />
            <span>
              <span className="block font-medium text-slate-900">
                Director Controls
              </span>
              <span className="text-sm text-slate-600">
                Manage project-level metadata fields.
              </span>
            </span>
          </label>

          {isDirector && (
            <div className="space-y-3 rounded-lg border border-slate-200 bg-slate-50 p-3">
              <p
                id="multi_directors_warning"
                className="text-sm text-amber-800"
              >
                Caution: more than one director device can conflict.
              </p>
              <label
                id="production_name_label"
                className="block text-sm font-medium text-slate-700"
              >
                Production Name
                <input
                  id="production_name_input"
                  className="scs-input"
                  type="text"
                  value={productionName}
                  onChange={(e) => set("productionName", e.target.value)}
                />
              </label>

              <label className="block rounded-lg border border-slate-200 bg-white p-3">
                <span className="mb-2 flex items-center gap-2 text-sm font-medium text-slate-700">
                  <input
                    id="scene_name_toggle"
                    type="checkbox"
                    className="h-4 w-4 accent-brand-500"
                    checked={enableSceneName}
                    onChange={(e) => set("enableSceneName", e.target.checked)}
                  />
                  Enable Scene Name
                </span>
                <input
                  id="scene_name_input"
                  className="scs-input mt-0"
                  type="text"
                  disabled={!enableSceneName}
                  value={sceneName}
                  onChange={(e) => set("sceneName", e.target.value)}
                />
              </label>

              <label className="block rounded-lg border border-slate-200 bg-white p-3">
                <span className="mb-2 flex items-center gap-2 text-sm font-medium text-slate-700">
                  <input
                    type="checkbox"
                    className="h-4 w-4 accent-brand-500"
                    checked={enableTakeNumber}
                    onChange={(e) => set("enableTakeNumber", e.target.checked)}
                  />
                  Enable Take Number
                </span>
                <input
                  id="take_number_input"
                  className="scs-input mt-0"
                  type="number"
                  disabled={!enableTakeNumber}
                  value={takeNumber}
                  onChange={(e) => set("takeNumber", e.target.value)}
                />
              </label>
            </div>
          )}

          <label className="flex items-start gap-3">
            <input
              type="checkbox"
              className="mt-1 h-4 w-4 accent-brand-500"
              checked={isOperator}
              onChange={(e) => set("isOperator", e.target.checked)}
            />
            <span>
              <span className="block font-medium text-slate-900">
                Operator Controls
              </span>
              <span className="text-sm text-slate-600">
                Add per-operator metadata for each device.
              </span>
            </span>
          </label>

          {isOperator && (
            <label className="block rounded-lg border border-slate-200 bg-slate-50 p-3 text-sm font-medium text-slate-700">
              <p>
                make sure that if you are renaming a camera using this field,
                that the same camera is used to scan the qr codes at the end,
                and that the sync codes for this device are also scanned
              </p>
              <br></br>
              <span className="mb-2 flex items-center gap-2">
                <input
                  id="is_operator_checkbox"
                  type="checkbox"
                  className="h-4 w-4 accent-brand-500"
                  checked={enableOperatorName}
                  onChange={(e) => set("enableOperatorName", e.target.checked)}
                />
                Rename Camera
              </span>
              <input
                type="text"
                id="operator_name_input"
                className="scs-input mt-0"
                disabled={!enableOperatorName}
                value={renameDevice}
                onChange={(e) => set("renameDevice", e.target.value)}
              />
            </label>
          )}

          <div className="border-t border-slate-200 pt-3">
            <div
              className={`scs-save-bar flex flex-wrap items-center gap-2 ${
                showRollbackPopup ? "scs-save-bar-hidden" : ""
              }`}
            >
              <button
                id="save_changes_button"
                className="scs-button"
                onClick={handleSave}
                disabled={!hasPending}
              >
                Save Changes
              </button>
              <button
                id="save_with_rollback_button"
                className="scs-button-secondary"
                onClick={openRollbackPopup}
                disabled={!hasPending}
              >
                Save with Rollback
              </button>
              {hasPending ? (
                <p
                  id="unsaved_changes_warning"
                  className="rounded-lg bg-amber-50 px-3 py-2 text-sm text-red-500 animate-bounce"
                >
                  Unsaved changes, your changes do not have an effect until you
                  save them.
                </p>
              ) : (
                <p className="rounded-lg bg-emerald-50 px-3 py-2 text-sm text-emerald-800">
                  All saved
                </p>
              )}
            </div>

            <SaveWithRollbackModal
              isOpen={showRollbackPopup}
              rollbackSeconds={saveRollbackSeconds}
              onClose={closeRollbackPopup}
              onConfirm={handleSaveWithRollback}
              onRollbackChange={(value) =>
                setLocal("saveRollbackSeconds", value)
              }
            />
          </div>
        </article>

        <article
          className={`scs-card min-h-0 space-y-3 overflow-auto p-3 md:block md:space-y-4 md:p-6 ${
            mobilePanel === "actions" ? "block" : "hidden"
          }`}
        >
          <h2 className="text-lg font-semibold text-slate-900">Actions</h2>
          <div className="flex flex-wrap gap-2">
            {hasLog && (
              <button
                id="download_csv_button"
                className="scs-button-secondary"
                onClick={handleDownloadCSV}
              >
                Download CSV
              </button>
            )}
            <button
              className="scs-button-secondary border-rose-300 text-rose-700 hover:bg-rose-50"
              onClick={() => {
                if (
                  !confirm(
                    "warning this will delete all previously saved metadata, please ensure it has been downlaoded/recorded",
                  )
                ) {
                  return;
                }
                const savedProductionName = productionName;

                cookieKeys.forEach((a) => {
                  setCookie(a, undefined, cookieOptions);
                });

                set("productionName", savedProductionName);
              }}
            >
              Clear Metadata
            </button>
          </div>

          <div className="grid gap-2 sm:grid-cols-2">
            <p>
              Take a photo or video of these qr codes on each camera at the
              start of each day.
            </p>

            <TimeQrDisplay />

            {/* <BarcodeDisplay />*/}
            <p>
              Take a recording of the sound generated by this button on all of
              your audio recording devices at the start of each day.
            </p>
            <button
              className="scs-button-secondary w-full"
              onClick={async () => {
                console.log(DEVICE_ID);
                const { audioContext } = await fskTransmit(
                  Number(DEVICE_ID),
                  120,
                  400,
                  audioCtxRef.current,
                );
                audioCtxRef.current = audioContext;
              }}
            >
              Play Audio Signature
            </button>
            <p>
              Take a photo or video of these qr codes after each day of
              recording
            </p>
            <QrMetadataDisplay />

            <TakePhotoScan />
            <InstallPWA />
          </div>
        </article>
      </section>
    </main>
  );
}

export default App;
