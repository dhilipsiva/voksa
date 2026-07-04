// voksa AudioWorkletProcessor. The WASM Module (compiled on the main thread)
// arrives via processorOptions and is instantiated HERE, in the worklet scope
// (CLAUDE.md web-audio rule). The module has zero imports, so a synchronous
// `new WebAssembly.Instance(module, {})` is all that's needed — no wasm-bindgen
// glue. We render the whole utterance once in the constructor, copy the PCM out
// of wasm memory, and play it 128 frames at a time in process().
//
// IMPORTANT: wasm linear memory can grow (and detach its ArrayBuffer) on any
// alloc/render call, so every typed-array view is built against the LIVE
// `wasm.memory.buffer` immediately after the call that produced the data, and
// the samples are copied into a plain owned Float32Array before playback.

class VoksaProcessor extends AudioWorkletProcessor {
  constructor(options) {
    super();
    // `textBytes` is a UTF-8 Uint8Array encoded on the MAIN thread: the
    // AudioWorkletGlobalScope has no TextEncoder in some browsers (Firefox), so
    // we never encode here.
    const { module, textBytes, flags, sampleRate } = options.processorOptions;
    const wasm = new WebAssembly.Instance(module, {}).exports;

    this._samples = new Float32Array(0);
    this._cursor = 0;

    const inPtr = wasm.voksa_alloc(textBytes.length);
    // Fresh view on the live buffer, then write the UTF-8 bytes.
    new Uint8Array(wasm.memory.buffer, inPtr, textBytes.length).set(textBytes);
    const outPtr = wasm.voksa_render(inPtr, textBytes.length, flags >>> 0, sampleRate >>> 0);
    wasm.voksa_dealloc(inPtr, textBytes.length);

    if (outPtr === 0) {
      this.port.postMessage('error');
    } else {
      const n = wasm.voksa_out_len();
      // Re-read .buffer: render may have grown memory. Copy OUT of wasm memory.
      this._samples = new Float32Array(n);
      this._samples.set(new Float32Array(wasm.memory.buffer, outPtr, n));
      wasm.voksa_free_f32(outPtr, n);
    }
  }

  process(_inputs, outputs) {
    const out = outputs[0][0];
    const s = this._samples;
    let c = this._cursor;
    const remaining = s.length - c;
    if (remaining <= 0) {
      this.port.postMessage('done');
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
