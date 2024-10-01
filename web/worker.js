importScripts("./pkg/customizer_wasm.js");

const { worker_entry_point } = wasm_bindgen;

self.onmessage = async (event) => {
  let init = await wasm_bindgen(
    "./pkg/customizer_wasm_bg.wasm",
    event.data[0],
  ).catch((err) => {
    setTimeout(() => {
      throw err;
    });
    throw err;
  });

  worker_entry_point(event.data[1]);

  init.__wbindgen_thread_destroy();
  close();
};
