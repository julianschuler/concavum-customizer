const { start } = wasm_bindgen;

async function run() {
    await wasm_bindgen();

    start();
}

run();
