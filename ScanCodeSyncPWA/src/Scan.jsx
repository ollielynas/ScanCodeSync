import { useRef } from "react";
import { saveFile } from "./storage";
import { getOrCreateDeviceId } from "./device_id";
const TakePhotoScan = () => {
  const inputRef = useRef(null);

  const handleScan = async (e) => {
    const file = e.target.files?.[0];
    if (!file) return;

    await saveFile(`DEVICE_ID:${getOrCreateDeviceId()}-${file.name}`, file);
  };

  return (
    <>
      <input
        ref={inputRef}
        type="file"
        accept="image/*"
        capture="environment"
        style={{ display: "none" }}
        onChange={handleScan}
      />
      <button onClick={() => inputRef.current.click()}>Scan</button>
    </>
  );
};

export default TakePhotoScan;
