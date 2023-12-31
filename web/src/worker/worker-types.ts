
export type SendToWorker = {
    name: 'counter-go',
} | {
    name: 'perft-setup',
    fen: string,
    depth: number,
} | SendToWorkerWithResponse;

export type SendToWorkerWithResponse = ({
    name: 'counter-count',
} | {
    name: 'perft-count',
} | {
    name: 'uci',
    line: string,
} | {
    name: 'uci-flush-output',
}) & { id: number };

export function encodeSendToWorker(msg: SendToWorker): string {
    return JSON.stringify(msg);
}

export function decodeSendToWorker(msg: string): SendToWorker {
    return JSON.parse(msg);
}

export type ReceiveFromWorkerMessage = ({
    name: 'ready',
} | {
    name: 'log',
    msg: Array<any>,
} | {
    name: 'error',
    msg: string,
}) & {
    kind: 'message',
};

export type ReceiveFromWorker = ReceiveFromWorkerMessage | ReceiveFromWorkerResponse;

export type ReceiveFromWorkerResponse = ({
    name: 'counter-count',
    counterResult: number,
} | {
    name: 'perft-count',
    perftResult: number,
} | {
    name: 'uci',
    output: string,
} | {
    name: 'uci-flush-output',
    output: string,
}) & {
    kind: 'response',
    id: number,
};

export function isEmpty(msg: ReceiveFromWorker): boolean {
    if (msg.name === 'log') {
        return msg.msg.length === 0;
    } else if (msg.name === 'uci') {
        return msg.output.length === 0;
    } else if (msg.name === 'uci-flush-output') {
        return msg.output.length === 0;
    }
    return false
}

export function isResponse(msg: ReceiveFromWorker): msg is ReceiveFromWorkerResponse {
    return msg.kind === 'response' && msg.id !== undefined;
}

export function responseMatchesRequest<
    S extends SendToWorkerWithResponse,
    R extends ReceiveFromWorkerResponse & Pick<S, "name">
>(msg: SendToWorkerWithResponse, response: ReceiveFromWorker): response is R {
    return isResponse(response) && response.id === msg.id && response.name === msg.name;
}

export function encodeReceiveFromWorker(msg: ReceiveFromWorker): string {
    return JSON.stringify(msg);
}

export function decodeReceiveFromWorker(msg: string): ReceiveFromWorker {
    return JSON.parse(msg);
}
