import { useEffect, useRef, useState } from "react";
import { getOrCreateDeviceId } from "./device_id";
const TIME_BITS = 64;
const DEVICE_BITS = 16;
const TOTAL_COLS = TIME_BITS + DEVICE_BITS + 8;

const DEVICE_ID = getOrCreateDeviceId();
const DEVICE_BITS_PRECOMPUTED = Array.from({ length: 16 }, (_, i) =>
  (DEVICE_ID >> i) & 1 ? "#ffffff" : "#000000",
);

const BarcodeDisplay = () => {
  const canvasRef = useRef(null);
  const animRef = useRef(null);
  const [visible, setVisible] = useState(false);

  useEffect(() => {
    if (!visible) return;

    const canvas = canvasRef.current;
    const ctx = canvas.getContext("2d");

    const resize = () => {
      canvas.width = window.innerWidth;
      canvas.height = window.innerHeight;
    };
    resize();
    window.addEventListener("resize", resize);

    const colColors = new Array(TOTAL_COLS);
    colColors[0] = "#ff0000";
    colColors[1] = "#ff0000";
    colColors[2] = "#ff0000";
    colColors[3] = "#ff0000";
    colColors[TOTAL_COLS - 1] = "#00ff00";
    colColors[TOTAL_COLS - 2] = "#00ff00";
    colColors[TOTAL_COLS - 3] = "#00ff00";
    colColors[TOTAL_COLS - 4] = "#00ff00";
    for (let x = 0; x < 16; x++) {
      colColors[TIME_BITS + 4 + x] = DEVICE_BITS_PRECOMPUTED[x];
    }

    const draw = () => {
      const now = BigInt(Date.now());
      const w = canvas.width;
      const h = canvas.height;
      const colW = w / TOTAL_COLS;

      for (let x = 4; x < TIME_BITS + 4; x++) {
        const bit = x - 4;
        colColors[x] =
          ((now >> BigInt(bit)) & 1n) === 1n ? "#ffffff" : "#000000";
      }

      for (let x = 0; x < TOTAL_COLS; x++) {
        ctx.fillStyle = colColors[x];
        ctx.fillRect(Math.floor(x * colW), 0, Math.ceil(colW), h);
      }

      animRef.current = requestAnimationFrame(draw);
    };

    animRef.current = requestAnimationFrame(draw);

    return () => {
      cancelAnimationFrame(animRef.current);
      window.removeEventListener("resize", resize);
    };
  }, [visible]);

  return (
    <>
      {visible && (
        <canvas
          ref={canvasRef}
          onClick={() => setVisible(false)}
          style={{
            display: "block",
            position: "fixed",
            inset: 0,
            cursor: "pointer",
          }}
        />
      )}
      <br></br>
      <button onClick={() => setVisible(true)}>Barcode</button>
    </>
  );
};

export default BarcodeDisplay;
