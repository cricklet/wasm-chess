import { ReceiveFromWorker, decodeSendToWorker, encodeReceiveFromWorker } from "./worker-types";

importScripts('/lib/wasm-pack/wasm_chess.js');

function send(msg: ReceiveFromWorker) {
    self.postMessage(encodeReceiveFromWorker(msg));
}


globalThis.BindingsJs = {
    log_to_js: function (msg) {
        send({ kind: 'message', name: 'log', msg: [msg] });
    }
};

async function setupUciForJs(): Promise<{ handleEvent: (MessageEvent: any) => boolean }> {
    let uciForJs = await wasm_bindgen.UciForJs.new();
    let uciThinkLoop: ReturnType<typeof setInterval> | undefined = undefined;

    uciThinkLoop = setInterval(() => {
        uciForJs.think();
    }, 1);

    // handle messages passed to the worker
    return {
        handleEvent: (e: MessageEvent) => {
            let data = decodeSendToWorker(e.data);

            switch (data.name) {
                case 'uci': {
                    let output = uciForJs.handle_line(data.line);
                    send({ kind: 'response', name: 'uci', id: data.id, output });
                    return true;
                }
                default:
                    return false;
            }
        }
    }
}


async function init_wasm_in_worker() {
    // load the wasm
    await wasm_bindgen('/lib/wasm-pack/wasm_chess_bg.wasm');

    let uci = await setupUciForJs();

    // handle messages passed to the worker
    self.onmessage = async e => {
        let data = decodeSendToWorker(e.data);

        if (uci.handleEvent(e)) {
            return;
        }

        send({ kind: 'message', name: 'error', msg: `unknown message: ${data}` });
    };

    send({ kind: 'message', name: 'ready' });
};

init_wasm_in_worker();