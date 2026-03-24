import { useCookies } from "react-cookie";
import "./App.css";
import BarcodeDisplay from "./Barcode";
import TakePhotoScan from "./Scan";
import { getOrCreateDeviceId } from "./device_id";
import QrMetadataDisplay from "./QRCode";
const DEVICE_ID = getOrCreateDeviceId();

function App() {
  const [cookies, setCookie] = useCookies([
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
    "pendingChanges",
    "changeLog",
  ]);

  const cookieOptions = { path: "/", maxAge: 60 * 60 * 24 * 365 };

  const set = (key, value) => {
    const pending = cookies.pendingChanges ?? {};
    setCookie("pendingChanges", { ...pending, [key]: value }, cookieOptions);
    setCookie(key, value, cookieOptions);
  };

  const handleSave = () => {
    const pending = cookies.pendingChanges ?? {};
    if (Object.keys(pending).length === 0) return;

    const timestamp = String(BigInt(Date.now()));
    const newRows = Object.entries(pending)
      .map(([key, value]) => `${DEVICE_ID},${timestamp},${key},${value}`)
      .join("\n");

    const existing = cookies.changeLog ?? "";
    const updated = existing ? `${existing}\n${newRows}` : newRows;
    setCookie("changeLog", updated, cookieOptions);
    setCookie("pendingChanges", {}, cookieOptions);
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

  const isMaster = getBool("isMaster");
  const isDirector = getBool("isDirector");
  const isOperator = getBool("isOperator");
  const enableOperatorName = getBool("enableOperatorName");
  const enableSceneName = getBool("enableSceneName");
  const enableTakeNumber = getBool("enableTakeNumber");
  const productionName = cookies.productionName ?? "Production Name";
  const operatorName = cookies.operatorName ?? "Operator Name";
  const sceneName = cookies.sceneName ?? "Scene Name";
  const takeNumber = cookies.takeNumber ?? 1;

  const hasPending = Object.keys(cookies.pendingChanges ?? {}).length > 0;
  const hasLog = !!(cookies.changeLog ?? "");

  return (
    <>
      <label>
        <input
          type="checkbox"
          checked={isMaster}
          onChange={(e) => set("isMaster", e.target.checked)}
        />
        Master Clock
      </label>
      <br />
      {isMaster && (
        <p>
          Caution! There should only ever be one master clock and it should not
          change. Please ensure that this device is intended to be the master
          device.
        </p>
      )}

      <label>
        <input
          type="checkbox"
          checked={isDirector}
          onChange={(e) => set("isDirector", e.target.checked)}
        />
        Director Controls
      </label>
      <br />
      {isDirector && (
        <>
          <p>
            Caution, having more than one director can result in unexpected
            behavior.
          </p>
          <label>
            Production Name:
            <input
              type="text"
              value={productionName}
              onChange={(e) => set("productionName", e.target.value)}
            />
          </label>
          <br />
          <label>
            <input
              type="checkbox"
              checked={enableSceneName}
              onChange={(e) => set("enableSceneName", e.target.checked)}
            />
            Scene Name:
            <input
              type="text"
              disabled={!enableSceneName}
              value={sceneName}
              onChange={(e) => set("sceneName", e.target.value)}
            />
          </label>
          <br />
          <label>
            <input
              type="checkbox"
              checked={enableTakeNumber}
              onChange={(e) => set("enableTakeNumber", e.target.checked)}
            />
            Take Number:
            <input
              type="number"
              disabled={!enableTakeNumber}
              value={takeNumber}
              onChange={(e) => set("takeNumber", e.target.value)}
            />
          </label>
        </>
      )}

      <br />

      <label>
        <input
          type="checkbox"
          checked={isOperator}
          onChange={(e) => set("isOperator", e.target.checked)}
        />
        Operator Controls
      </label>

      {isOperator && (
        <>
          <br />
          <label>
            <input
              type="checkbox"
              checked={enableOperatorName}
              onChange={(e) => set("enableOperatorName", e.target.checked)}
            />
            Operator Name:
            <input
              type="text"
              disabled={!enableOperatorName}
              value={operatorName}
              onChange={(e) => set("operatorName", e.target.value)}
            />
          </label>
          <br />
        </>
      )}

      <br />
      <button onClick={handleSave} disabled={!hasPending}>
        Save Changes
      </button>
      {hasPending && <span> (unsaved changes)</span>}
      {hasLog && (
        <button onClick={handleDownloadCSV} style={{ marginLeft: "8px" }}>
          Download CSV
        </button>
      )}

      <QrMetadataDisplay />
      <BarcodeDisplay />
      <TakePhotoScan />
    </>
  );
}

export default App;
