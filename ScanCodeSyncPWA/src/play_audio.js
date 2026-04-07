const BARKER_CODE = [1, 0, 1, 1, 0, 1, 1, 1, 0, 0, 0];
const FREQ_ZERO = 1200;
const FREQ_ONE = 2200;
let activePlaybackSource = null;

function bigIntTo64Bits(n) {
  const bits = [];
  for (let i = 63n; i >= 0n; i--) bits.push(Number((n >> i) & 1n));
  return bits;
}

function u16ToBits(n) {
  const bits = [];
  for (let i = 15; i >= 0; i--) bits.push((n >> i) & 1);
  return bits;
}

const GF_EXP = new Uint8Array(512);
const GF_LOG = new Uint8Array(256);
(function buildTables() {
  let x = 1;
  for (let i = 0; i < 255; i++) {
    GF_EXP[i] = x;
    GF_LOG[x] = i;
    x <<= 1;
    if (x & 0x100) x ^= 0x11d;
  }
  for (let i = 255; i < 512; i++) GF_EXP[i] = GF_EXP[i - 255];
})();
function gfMul(a, b) {
  if (a === 0 || b === 0) return 0;
  return GF_EXP[GF_LOG[a] + GF_LOG[b]];
}
function gfPow(x, power) {
  return GF_EXP[(GF_LOG[x] * power) % 255];
}
function rsEncode(msg, nsym) {
  let g = [1];
  for (let i = 0; i < nsym; i++) {
    const root = gfPow(2, i);
    const ng = new Array(g.length + 1).fill(0);
    for (let j = 0; j < g.length; j++) {
      ng[j] ^= g[j];
      ng[j + 1] ^= gfMul(g[j], root);
    }
    g = ng;
  }

  const parity = new Array(nsym).fill(0);
  for (const m of msg) {
    const feedback = m ^ parity[0];
    for (let i = 0; i < nsym - 1; i++) {
      parity[i] = parity[i + 1] ^ gfMul(g[i + 1], feedback);
    }
    parity[nsym - 1] = gfMul(g[nsym], feedback);
  }

  return parity;
}

function buildBitstream(epochMs, deviceId) {
  const epochBits = bigIntTo64Bits(BigInt(epochMs));
  const idBits = u16ToBits(deviceId);

  const payloadBits = [...epochBits, ...idBits];
  const payloadBytes = [];
  for (let i = 0; i < payloadBits.length; i += 8) {
    let byte = 0;
    for (let j = 0; j < 8; j++) byte = (byte << 1) | (payloadBits[i + j] || 0);
    payloadBytes.push(byte);
  }

  const rsBytes = rsEncode(payloadBytes, 8);
  const rsBits = [];
  for (const b of rsBytes)
    for (let i = 7; i >= 0; i--) rsBits.push((b >> i) & 1);

  return [...BARKER_CODE, ...epochBits, ...idBits, ...rsBits];
}

function stopActivePlayback() {
  if (!activePlaybackSource) return;

  try {
    activePlaybackSource.onended = null;
    activePlaybackSource.stop();
  } catch {
    // Ignore sources that have already ended or were never started.
  }

  try {
    activePlaybackSource.disconnect();
  } catch {
    // Ignore already-disconnected nodes.
  }

  activePlaybackSource = null;
}

/**
 * Encodes and plays an FSK audio signal.
 *
 * @param {object} options
 * @param {number} options.deviceId  - u16 device identifier (0–65535)
 * @param {number} [options.baudRate=1200] - Baud rate (bits per second)
 * @param {number} [options.delayMs=15]   - How far in the future to schedule
 *                                          the transmission (ms). The epoch
 *                                          encoded in the signal matches this
 *                                          scheduled start time.
 * @param {AudioContext} [options.audioContext] - Optional existing AudioContext
 *                                               to reuse (avoids hitting the
 *                                               browser's context limit).
 * @returns {Promise<{ epochMs, bitCount, durationMs, audioContext }>}
 */
export async function fskTransmit(
  deviceId,
  baudRate = 120,
  delayMs = 15,
  audioContext,
) {
  if (deviceId == null || deviceId < 0 || deviceId > 65535)
    throw new RangeError(`deviceId must be a u16 (0–65535), got ${deviceId}`);

  const ctx =
    audioContext ?? new (window.AudioContext || window.webkitAudioContext)();
  if (ctx.state === "suspended") await ctx.resume();

  const epochMs = Date.now() + delayMs;
  const bits = buildBitstream(epochMs, deviceId);

  const sampleRate = ctx.sampleRate;
  const samplesPerBit = Math.floor(sampleRate / baudRate);
  const buffer = ctx.createBuffer(1, bits.length * samplesPerBit, sampleRate);
  const data = buffer.getChannelData(0);

  let phase = 0;
  for (let i = 0; i < bits.length; i++) {
    const freq = bits[i] === 0 ? FREQ_ZERO : FREQ_ONE;
    for (let s = 0; s < samplesPerBit; s++) {
      data[i * samplesPerBit + s] = Math.sin(phase) * 0.7;
      phase += (2 * Math.PI * freq) / sampleRate;
    }
  }

  stopActivePlayback();

  const source = ctx.createBufferSource();
  source.buffer = buffer;
  source.connect(ctx.destination);
  source.onended = () => {
    if (activePlaybackSource === source) activePlaybackSource = null;
    try {
      source.disconnect();
    } catch {
      // Ignore already-disconnected nodes.
    }
  };
  activePlaybackSource = source;
  source.start(ctx.currentTime + delayMs / 1000);

  const durationMs = Math.round((buffer.length / sampleRate) * 1000);

  return { epochMs, bitCount: bits.length, durationMs, audioContext: ctx };
}
