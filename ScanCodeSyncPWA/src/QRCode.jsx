import { useEffect, useState } from "react";
import { useCookies } from "react-cookie";
import { createPortal } from "react-dom";
import QRCode from "react-qr-code";
import { getOrCreateDeviceId } from "./device_id";

const DEVICE_ID = getOrCreateDeviceId();
const CHUNK_SIZE = 200;

function chunkString(str, size) {
  if (!str) return [];
  const lines = str.split("\n");
  const chunks = [];
  let current = "";

  for (const line of lines) {
    const addition = current ? "\n" + line : line;
    if (current && (current + addition).length > size) {
      chunks.push(current);
      current = line;
    } else {
      current += addition;
    }
  }

  if (current) chunks.push(current);
  return chunks;
}

const QrMetadataDisplay = () => {
  const [visible, setVisible] = useState(false);
  const [cookies] = useCookies(["changeLog"]);
  const [chunks, setChunks] = useState([]);
  const [index, setIndex] = useState(0);
  const [touchStartX, setTouchStartX] = useState(null);

  const [slideShow, setSlideShow] = useState(false);

  useEffect(() => {
    const interval = setInterval(() => {
      if (slideShow) {
        setIndex((index + 1) % chunks.length);
      }
    }, 500);

    return () => clearInterval(interval);
  }, [slideShow, index, chunks]);

  useEffect(() => {
    if (!visible) return;
    const data = cookies.changeLog || "";
    setChunks(chunkString(data, CHUNK_SIZE));
    setIndex(0);
  }, [visible, cookies.changeLog]);

  useEffect(() => {
    if (!visible) return;
    const onKey = (e) => {
      if (e.key === "ArrowRight")
        setIndex((i) => Math.min(i + 1, chunks.length - 1));
      if (e.key === "ArrowLeft") setIndex((i) => Math.max(i - 1, 0));
      if (e.key === "Escape") setVisible(false);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [visible, chunks.length]);

  const handleTouchStart = (e) => setTouchStartX(e.touches[0].clientX);
  const handleTouchEnd = (e) => {
    if (touchStartX === null) return;
    const diff = touchStartX - e.changedTouches[0].clientX;
    if (Math.abs(diff) > 50) {
      if (diff > 0) setIndex((i) => Math.min(i + 1, chunks.length - 1));
      else setIndex((i) => Math.max(i - 1, 0));
    }
    setTouchStartX(null);
  };

  return (
    <>
      {visible &&
        createPortal(
          <div
            onTouchStart={handleTouchStart}
            onTouchEnd={handleTouchEnd}
            className="fixed inset-0 z-[9999] overflow-hidden bg-white"
          >
            <button
              onClick={() => setVisible(false)}
              className="absolute right-3 top-3 rounded-lg border border-slate-300 bg-white/90 px-3 py-2 text-sm font-medium text-slate-800"
            >
              Close
            </button>

            <div className="absolute left-3 top-3 z-10 font-mono text-sm text-slate-700">
              <h1>
                {index + 1} / {chunks.length}
              </h1>
            </div>

            {chunks.length === 0 ? (
              <p className="rounded-lg bg-slate-100 px-4 py-2 text-sm text-slate-800">
                No data found in changeLog cookie.
              </p>
            ) : (
              <div className="flex h-[100dvh] w-[100vw] items-center justify-center bg-white">
                <QRCode
                  value={chunks[index]}
                  size={1024}
                  style={{
                    width: "min(85vw, 85dvh)",
                    height: "min(85vw, 85dvh)",
                    display: "block",
                  }}
                  level="M"
                />
              </div>
            )}

            <div className="absolute bottom-3 left-1/2 z-10 flex -translate-x-1/2 flex-wrap items-center justify-center gap-2 rounded-xl bg-white/90 p-2 shadow">
              <button
                onClick={() => setIndex((i) => Math.max(i - 1, 0))}
                disabled={index === 0}
                className="scs-button-secondary"
              >
                ← Prev
              </button>
              <button
                onClick={() =>
                  setIndex((i) => Math.min(i + 1, chunks.length - 1))
                }
                disabled={index === chunks.length - 1}
                className="scs-button-secondary"
              >
                Next →
              </button>
              <button
                onClick={() => setSlideShow(!slideShow)}
                className="scs-button-secondary"
              >
                {slideShow ? "Pause Slideshow" : "Play Slideshow"}
              </button>
            </div>
          </div>,
          document.body,
        )}
      <button
        className="scs-button-secondary w-full"
        onClick={() => setVisible(true)}
      >
        Metadata Code
      </button>
    </>
  );
};

export default QrMetadataDisplay;
