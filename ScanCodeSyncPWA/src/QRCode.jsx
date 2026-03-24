import { useEffect, useState } from "react";
import { getOrCreateDeviceId } from "./device_id";
import { useCookies } from "react-cookie";
import QRCode from "react-qr-code";

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
    }, 200);

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
      {visible && (
        <div
          onTouchStart={handleTouchStart}
          onTouchEnd={handleTouchEnd}
          style={{
            position: "fixed",
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            background: "white",
            display: "flex",
            flexDirection: "column",
            alignItems: "center",
            justifyContent: "center",
            zIndex: 1000,
          }}
        >
          <button
            onClick={() => setVisible(false)}
            style={{ position: "absolute", top: 12, right: 12 }}
          >
            Close
          </button>

          <div style={{ marginBottom: 12, fontFamily: "monospace" }}>
            <h1>
              {index + 1} / {chunks.length}
            </h1>
          </div>

          {chunks.length === 0 ? (
            <p>No data found in changeLog cookie.</p>
          ) : (
            <QRCode
              value={chunks[index]}
              size={Math.min(window.innerWidth, window.innerHeight) - 120}
              level="M"
            />
          )}

          <div style={{ display: "flex", gap: 24, marginTop: 24 }}>
            <button
              onClick={() => setIndex((i) => Math.max(i - 1, 0))}
              disabled={index === 0}
            >
              ← Prev
            </button>
            <button
              onClick={() =>
                setIndex((i) => Math.min(i + 1, chunks.length - 1))
              }
              disabled={index === chunks.length - 1}
            >
              Next →
            </button>
            <button onClick={() => setSlideShow(!slideShow)}>
              Play Slideshow
            </button>
          </div>
        </div>
      )}
      <br />
      <button onClick={() => setVisible(true)}>Metadata QR code</button>
    </>
  );
};

export default QrMetadataDisplay;
