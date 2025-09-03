import init from "./pkg/customizer_wasm.js"

try {
  await init();
} catch (errorMessage) {
  document.getElementById("error").textContent = "Error: " + errorMessage;
}
