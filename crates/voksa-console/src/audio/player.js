// voksa console — player-only AudioWorklet (ADR 0003). The engine renders on
// the main thread; this worklet only plays a Float32Array, 128 frames at a
// time, and posts 'done' when exhausted. No wasm runs here.
class VoksaPlayer extends AudioWorkletProcessor {
  constructor(options) {
    super();
    this._samples = options.processorOptions.pcm;
    this._cursor = 0;
  }
  process(_inputs, outputs) {
    const out = outputs[0][0];
    const s = this._samples;
    let c = this._cursor;
    const remaining = s.length - c;
    if (remaining <= 0) {
      this.port.postMessage({ type: 'done' });
      return false;
    }
    const n = remaining < out.length ? remaining : out.length;
    for (let i = 0; i < n; i++) out[i] = s[c + i];
    for (let i = n; i < out.length; i++) out[i] = 0;
    this._cursor = c + n;
    return true;
  }
}
registerProcessor('voksa-player', VoksaPlayer);
