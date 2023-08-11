
importScripts('/lib/wasm-pack/wasm_chess.js');

globalThis.BindingsJs = {
    log_to_js: function (msg) {
        self.postMessage(`${msg}`);
    }
};

async function init_wasm_in_worker() {
    // load the wasm
    await wasm_bindgen('/lib/wasm-pack/wasm_chess_bg.wasm');

    let asyncCounter = await wasm_bindgen.AsyncCounterForJs.new();
    let asyncPerft = await wasm_bindgen.PerftForJs.new();

    // handle messages passed to the worker
    self.onmessage = async event => {
        self.postMessage(`worker recv: ${event.data}`)
        if (event.data === 'counter-go') {
            await asyncCounter.start()
        } else if (event.data === 'counter-stop') {
            asyncCounter.stop()
        } else if (event.data === 'counter-count') {
            self.postMessage(asyncCounter.count())
        } else {
            self.postMessage(`error: unknown message ${event.data}`)
        }
    };

    self.postMessage(`ready`);
};

init_wasm_in_worker();
self.postMessage("wasm-worker-for-testing.js loading")