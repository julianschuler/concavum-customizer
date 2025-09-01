import init, { initThreadPool, worker_entry_point } from "./pkg/customizer_wasm.js"

self.onmessage = async (event) => {
  let worker = await init(
    "./pkg/customizer_wasm_bg.wasm",
    event.data[0],
  ).catch((err) => {
    setTimeout(() => {
      throw err;
    });
    throw err;
  });

  await initThreadPool(navigator.hardwareConcurrency);
  worker_entry_point(event.data[1]);

  worker.__wbindgen_thread_destroy();
  close();
};
