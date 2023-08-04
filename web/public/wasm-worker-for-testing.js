
importScripts('lib/wasm-pack/crab_chess.js');

globalThis.BindingsJs = {
    log: function (msg) {
        self.postMessage(`${msg}`);
    }
};

async function init_wasm_in_worker() {
    // load the wasm
    await wasm_bindgen('./lib/wasm-pack/crab_chess_bg.wasm');

    let asyncCounter = await wasm_bindgen.AsyncCounter.new();

    self.postMessage(`ready`);

    // handle messages passed to the worker
    self.onmessage = async event => {
        if (event.data === 'go') {
            asyncCounter.start()
        } else if (event.data === 'stop') {
            asyncCounter.stop()
        } else if (event.data === 'count') {
            self.postMessage(asyncCounter.count())
        } else if (event.data === 'flush') {
            self.postMessage(`flush: ${asyncCounter.flush()}`)
        } else {
            self.postMessage(`error: unknown message ${event.data}`)
        }
    };
};

init_wasm_in_worker();