import { useRef } from "react";
import { getOrCreateDeviceId } from "./device_id";
import { downloadFile, saveFile } from "./storage";
const TakePhotoScan = () => {
  const inputRef = useRef(null);

  const handleScan = async (e) => {
    const now = BigInt(Date.now());
    const file = e.target.files?.[0];
    if (!file) return;

    await saveFile(
      `DEVICE_ID:${getOrCreateDeviceId()}TIMESTAMP:${now}-${file.name}`,
      file,
    );

    const url = URL.createObjectURL(file);
    const a = document.createElement("a");
    a.href = url;
    a.download = file.name; // or `filename` if you want the full stored name
    a.click();
    URL.revokeObjectURL(url);
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
      <button
        className="scs-button w-full"
        onClick={() => inputRef.current.click()}
      >
        Scan Image
      </button>
    </>
  );
};

export default TakePhotoScan;
