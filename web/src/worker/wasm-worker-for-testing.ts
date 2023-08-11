import { ReceiveFromWorker, ReceiveFromWorkerMessage, decodeSendToWorker, encodeReceiveFromWorker } from "./worker-types";

importScripts('/lib/wasm-pack/wasm_chess.js');

function send(msg: ReceiveFromWorker) {
    self.postMessage(encodeReceiveFromWorker(msg));
}

globalThis.BindingsJs = {
    log_to_js: function (msg) {
        send({ kind: 'message', name: 'log', msg: [msg] });
    }
};

async function init_wasm_in_worker() {
    // load the wasm
    await wasm_bindgen('/lib/wasm-pack/wasm_chess_bg.wasm');

    let asyncCounter = await wasm_bindgen.AsyncCounterForJs.new();
    let asyncPerft = await wasm_bindgen.PerftForJs.new();

    // handle messages passed to the worker
    self.onmessage = async e => {
        let data = decodeSendToWorker(e.data);

        switch (data.name) {
            case 'counter-go':
                asyncCounter.start();
                break;
            case 'counter-stop':
                let counterResult = await asyncCounter.stop();
                send({ kind: 'response', name: 'counter-stop', id: data.id, counterResult });
                break;
            default:
                send({ kind: 'message', name: 'error', msg: `unknown message: ${data}` });
        }
    };

    send({ kind: 'message', name: 'ready' });
};

init_wasm_in_worker();