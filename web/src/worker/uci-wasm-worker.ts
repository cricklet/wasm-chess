import { ReceiveFromWorker, SendToWorkerWithResponse, decodeSendToWorker, encodeReceiveFromWorker } from "./worker-types";

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

    let output: string[] = [];
    uciThinkLoop = setInterval(() => {
        let out = uciForJs.think().trim();
        if (out.length > 0) {
            output.push(out);
        }
    }, 1);

    function flush(request: SendToWorkerWithResponse & { name: 'uci-flush-output' | 'uci' }) {
        send({
            kind: 'response',
            name: request.name,
            id: request.id,
            output:
                output.map(v => v.trim()).filter(line => line.length > 0).join('\n')
        });
        output = [];
    }

    // handle messages passed to the worker
    return {
        handleEvent: (e: MessageEvent) => {
            let data = decodeSendToWorker(e.data);

            switch (data.name) {
                case 'uci': {
                    let out = uciForJs.handle_line(data.line).trim();
                    if (out.length > 0) {
                        output.push(out);
                    }
                    flush(data);
                    return true;
                }
                case 'uci-flush-output': {
                    flush(data);
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