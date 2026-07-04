// voksa AudioWorkletProcessor. The WASM Module (compiled on the main thread)
// arrives via processorOptions and is instantiated HERE, in the worklet scope
// (CLAUDE.md web-audio rule). The module has zero imports, so a synchronous
// `new WebAssembly.Instance(module, {})` is all that's needed — no wasm-bindgen
// glue. We render the whole utterance once in the constructor, copy the PCM out
// of wasm memory, post it back to the main thread (for WAV download / waveform),
// and play it 128 frames at a time in process().
//
// IMPORTANT: wasm linear memory can grow (and detach its ArrayBuffer) on any
// alloc/render call, so every typed-array view is built against the LIVE
// `wasm.memory.buffer` immediately after the call that produced the data, and
// the samples are copied into a plain owned Float32Array before playback.
//
// Text is UTF-8 encoded on the MAIN thread (no TextEncoder in the worklet
// scope on some browsers); `params` is a Float32Array in the fixed voksa layout.

class VoksaProcessor extends AudioWorkletProcessor {
  constructor(options) {
    super();
    const { module, textBytes, params, flags, sampleRate } = options.processorOptions;
    const wasm = new WebAssembly.Instance(module, {}).exports;

    this._samples = new Float32Array(0);
    this._cursor = 0;

    // Marshal the UTF-8 text.
    const inPtr = wasm.voksa_alloc(textBytes.length);
    new Uint8Array(wasm.memory.buffer, inPtr, textBytes.length).set(textBytes);

    // Marshal the f32 param block (4 bytes each).
    const p = params instanceof Float32Array ? params : new Float32Array(params || []);
    const pBytes = p.length * 4;
    const paramsPtr = pBytes > 0 ? wasm.voksa_alloc(pBytes) : 0;
    if (pBytes > 0) {
      new Float32Array(wasm.memory.buffer, paramsPtr, p.length).set(p);
    }

    const outPtr = wasm.voksa_render_params(
      inPtr, textBytes.length, flags >>> 0, sampleRate >>> 0, paramsPtr, p.length);

    wasm.voksa_dealloc(inPtr, textBytes.length);
    if (pBytes > 0) wasm.voksa_dealloc(paramsPtr, pBytes);

    if (outPtr === 0) {
      this.port.postMessage({ type: 'error' });
    } else {
      const n = wasm.voksa_out_len();
      // Re-read .buffer: render may have grown memory. Copy OUT of wasm memory.
      this._samples = new Float32Array(n);
      this._samples.set(new Float32Array(wasm.memory.buffer, outPtr, n));
      wasm.voksa_free_f32(outPtr, n);
      // Hand a copy to the main thread for download + waveform.
      this.port.postMessage({ type: 'samples', pcm: this._samples.slice(), sampleRate });
    }
  }

  process(_inputs, outputs) {
    const out = outputs[0][0];
    const s = this._samples;
    let c = this._cursor;
    const remaining = s.length - c;
    if (remaining <= 0) {
      this.port.postMessage({ type: 'done' });
      return false; // let the host release the node
    }
    const n = remaining < out.length ? remaining : out.length;
    for (let i = 0; i < n; i++) out[i] = s[c + i];
    for (let i = n; i < out.length; i++) out[i] = 0;
    this._cursor = c + n;
    return true;
  }
}

registerProcessor('voksa-processor', VoksaProcessor);
