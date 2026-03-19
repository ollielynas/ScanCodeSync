import { useState } from "react";
import "./App.css";
import BarcodeDisplay from "./Barcode";
import TakePhotoScan from "./Scan";

function App() {
  // control buttons
  let [isMaster, setIsMaster] = useState(false);
  let [isDirector, setIsDirector] = useState(false);
  let [isOperator, setIsOperator] = useState(false);

  let [productionName, setProductionName] = useState("Production Name");

  let [enableOperatorName, setEnableOperatorName] = useState(false);
  let [operatorName, setOperatorName] = useState("Operator Name");

  let [enableSceneName, setEnableSceneName] = useState(false);
  let [sceneName, setSceneName] = useState("Scene Name");

  let [enableTakeNumber, setEnableTakeNumber] = useState(false);
  let [takeNumber, setTakeNumber] = useState(1);

  const handleIsMasterChange = (event) => {
    setIsMaster(event.target.checked);
  };
  const handleIsDirectorChange = (event) => {
    setIsDirector(event.target.checked);
  };
  const handleIsOperatorChange = (event) => {
    setIsOperator(event.target.checked);
  };

  return (
    <>
      <label>
        {/* The checked prop controls the checkbox state */}
        <input
          type="checkbox"
          checked={isMaster}
          onChange={handleIsMasterChange} // The onChange handler updates the state
        />
        Master Clock
      </label>

      {isMaster ? (
        <p>
          Caution! There should only ever be one master clock and it should not
          change. Please ensure that this device is intneded to be the master
          device
        </p>
      ) : (
        <p></p>
      )}

      <label>
        {/* The checked prop controls the checkbox state */}
        <input
          type="checkbox"
          checked={isDirector}
          onChange={handleIsDirectorChange} // The onChange handler updates the state
        />
        Director Controls
      </label>

      {isDirector ? (
        <>
          <p>
            Caution, having more than one director can resault in unexpected
            behavior
          </p>
          <label>
            Production Name:
            <input
              type={"text"}
              value={productionName}
              onChange={(e) => {
                setProductionName(e.target.value);
              }}
            ></input>
          </label>
          <br></br>
          <label>
            <input
              checked={enableSceneName}
              onChange={(e) => {
                setEnableSceneName(e.target.checked);
              }}
              type="checkbox"
            ></input>
            Scene Name:
            <input
              type={"text"}
              disabled={!enableSceneName}
              value={sceneName}
              onChange={(e) => {
                setSceneName(e.target.value);
              }}
            ></input>
          </label>
          <br></br>
          <label>
            <input
              checked={enableTakeNumber}
              onChange={(e) => {
                setEnableTakeNumber(e.target.checked);
              }}
              type="checkbox"
            ></input>
            Take Number:
            <input
              type={"number"}
              disabled={!enableTakeNumber}
              value={takeNumber}
              onChange={(e) => {
                setTakeNumber(e.target.value);
              }}
            ></input>
          </label>
        </>
      ) : (
        <p></p>
      )}
      <br></br>
      <label>
        {/* The checked prop controls the checkbox state */}
        <input
          type="checkbox"
          checked={isOperator}
          onChange={handleIsOperatorChange} // The onChange handler updates the state
        />
        Operator Controls
      </label>

      {isOperator ? (
        <>
          <br></br>
          <label>
            <input
              checked={enableOperatorName}
              onChange={(e) => {
                setEnableOperatorName(e.target.checked);
              }}
              type="checkbox"
            ></input>
            Operator Name:
            <input
              type={"text"}
              disabled={!enableOperatorName}
              value={operatorName}
              onChange={(e) => {
                setOperatorName(e.target.value);
              }}
            ></input>
          </label>
          <br></br>
        </>
      ) : (
        <></>
      )}

      <BarcodeDisplay />
      <TakePhotoScan />
    </>
  );
}

export default App;
