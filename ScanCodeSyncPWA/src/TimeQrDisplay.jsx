import { useEffect, useMemo, useState } from "react";
import { createPortal } from "react-dom";
import QRCode from "react-qr-code";
import { getOrCreateDeviceId } from "./device_id";

const DEVICE_ID = getOrCreateDeviceId();
const QR_COUNT = 3;
const UPDATE_INTERVAL_MS = 33;
const QR_UPDATE_STEPS_MS = [1, 173, 500];
const QR_UPDATE_PHASE_MS = [0, 41, 137];
const QR_HORIZONTAL_OFFSETS_PCT = [-6, 7, -5];
const OVERLAY_PADDING_PX = 18;
const GRID_GAP_PX = 18;
const TILE_BORDER_PX = 0;
const TILE_INNER_RATIO = 0.82;

const packTimeAndDevice = (timeMs, deviceId) => {
  const t64 = BigInt.asUintN(64, BigInt(timeMs));
  const d16 = BigInt.asUintN(16, BigInt(deviceId));
  return `${t64.toString(10)}-${d16.toString(10)}`;
};

const quantizeTime = (nowMs, stepMs, phaseMs = 0) => {
  const step = BigInt(stepMs);
  const now = BigInt(nowMs);
  const phase = BigInt(phaseMs);
  return ((now - phase) / step) * step + phase;
};

const TimeQrDisplay = () => {
  const [visible, setVisible] = useState(false);
  const [nowMs, setNowMs] = useState(Date.now());
  const [viewport, setViewport] = useState({
    width: typeof window !== "undefined" ? window.innerWidth : 1280,
    height: typeof window !== "undefined" ? window.innerHeight : 720,
  });

  useEffect(() => {
    if (typeof window === "undefined") return;

    const onResize = () => {
      setViewport({ width: window.innerWidth, height: window.innerHeight });
    };

    onResize();
    window.addEventListener("resize", onResize);
    return () => window.removeEventListener("resize", onResize);
  }, []);

  useEffect(() => {
    if (!visible) return;

    const tick = () => setNowMs(Date.now());
    tick();
    const id = setInterval(tick, UPDATE_INTERVAL_MS);
    return () => clearInterval(id);
  }, [visible]);

  const qrPayloads = useMemo(
    () =>
      Array.from({ length: QR_COUNT }, (_, index) => {
        const stepMs = QR_UPDATE_STEPS_MS[index];
        const phaseMs = QR_UPDATE_PHASE_MS[index];
        const quantized = quantizeTime(nowMs, stepMs, phaseMs);
        return packTimeAndDevice(quantized, DEVICE_ID);
      }),
    [nowMs],
  );

  const { tileSizePx, innerSizePx } = useMemo(() => {
    const contentWidth = Math.max(1, viewport.width - OVERLAY_PADDING_PX * 2);
    const contentHeight = Math.max(1, viewport.height - OVERLAY_PADDING_PX * 2);

    const widthPerTile = contentWidth;
    const heightPerTile =
      (contentHeight - GRID_GAP_PX * (QR_COUNT - 1)) / QR_COUNT;

    const candidateTile = Math.floor(
      Math.min(widthPerTile, heightPerTile),
    );
    const safeTile = Math.max(1, candidateTile);
    const inner = Math.max(64, Math.floor((safeTile - TILE_BORDER_PX * 2) * TILE_INNER_RATIO));

    return { tileSizePx: safeTile, innerSizePx: inner };
  }, [viewport]);

  return (
    <>
      {visible &&
        createPortal(
          <div
            onClick={() => setVisible(false)}
            className="fixed inset-0 z-[9999] cursor-pointer"
            style={{ backgroundColor: "#F3F4F6", padding: `${OVERLAY_PADDING_PX}px` }}
          >
            <div
              className="grid h-full w-full"
              style={{
                gridTemplateColumns: "minmax(0, 1fr)",
                gridTemplateRows: `repeat(${QR_COUNT}, minmax(0, 1fr))`,
                gap: `${GRID_GAP_PX}px`,
              }}
            >
              {qrPayloads.map((payload, index) => (
                <div
                  key={`qr-${index}`}
                  className="flex min-h-0 flex-1 items-center justify-center"
                >
                  <div
                    style={{
                      width: `${tileSizePx}px`,
                      height: `${tileSizePx}px`,
                      backgroundColor: "#FFFFFF",
                      boxSizing: "border-box",
                      display: "flex",
                      alignItems: "center",
                      justifyContent: "center",
                      transform: `translateX(${QR_HORIZONTAL_OFFSETS_PCT[index]}%)`,
                    }}
                  >
                    <QRCode
                      value={payload}
                      level="H"
                      bgColor="#FFFFFF"
                      fgColor="#000000"
                      size={innerSizePx}
                      style={{
                        width: `${innerSizePx}px`,
                        height: `${innerSizePx}px`,
                        display: "block",
                        shapeRendering: "crispEdges",
                      }}
                    />
                  </div>
                </div>
              ))}
            </div>
          </div>,
          document.body,
        )}

      <button
        className="scs-button-secondary w-full"
        onClick={() => setVisible(true)}
      >
        Fullscreen Time QR
      </button>
    </>
  );
};

export default TimeQrDisplay;
