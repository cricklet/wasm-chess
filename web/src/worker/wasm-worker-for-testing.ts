import { WorkerToWeb, decodeWebToWorker, encodeWorkerToWeb } from "./worker-types";

importScripts('/lib/wasm-pack/wasm_chess.js');

function send(msg: WorkerToWeb) {
    self.postMessage(encodeWorkerToWeb(msg));
}

globalThis.BindingsJs = {
    log_to_js: function (msg) {
        send({ kind: 'log', msg: [msg] });
    }
};

async function init_wasm_in_worker() {
    // load the wasm
    await wasm_bindgen('/lib/wasm-pack/wasm_chess_bg.wasm');

    let asyncCounter = await wasm_bindgen.AsyncCounterForJs.new();
    let asyncPerft = await wasm_bindgen.PerftForJs.new();

    // handle messages passed to the worker
    self.onmessage = async e => {
        let data = decodeWebToWorker(e.data);

        if (data.kind === 'counter-go') {
            await asyncCounter.start()
        } else if (data.kind === 'counter-stop') {
            asyncCounter.stop()
        } else if (data.kind === 'counter-count') {
            send({ kind: 'counter-count', count: await asyncCounter.count() });
        } else {
            send({ kind: 'error', msg: `unknown message: ${data}` });
        }
    };

    send({ kind: 'ready' });
};

init_wasm_in_worker();