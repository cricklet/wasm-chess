
importScripts('lib/wasm-pack/wasm_chess.js');

globalThis.BindingsJs = {
    log_to_js: function (msg) {
        self.postMessage(`${msg}`);
    }
};

async function init_wasm_in_worker() {
    // load the wasm
    await wasm_bindgen('./lib/wasm-pack/wasm_chess_bg.wasm');

    let asyncCounter = await wasm_bindgen.AsyncCounter.new();
    let asyncPerft = await wasm_bindgen.AsyncPerft.new();

    self.postMessage(`ready`);

    // handle messages passed to the worker
    self.onmessage = async event => {
        if (event.data === 'counter-go') {
            asyncCounter.start()
        } else if (event.data === 'counter-stop') {
            asyncCounter.stop()
        } else if (event.data === 'counter-count') {
            self.postMessage(asyncCounter.count())
        } else if (event.data === 'perft-go') {
            asyncPerft.start()
        } else if (event.data === 'perft-stop') {
            asyncPerft.stop()
        } else if (event.data === 'perft-count') {
            self.postMessage(asyncPerft.count())
        } else {
            self.postMessage(`error: unknown message ${event.data}`)
        }
    };
};

init_wasm_in_worker();