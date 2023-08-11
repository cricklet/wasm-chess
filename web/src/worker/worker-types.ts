
export type SendToWorker = {
    name: 'counter-go',
} | {
    name: 'perft-setup',
    fen: string,
    depth: number,
} | {
    name: 'perft-count',
} | {
    name: 'perft-think',
} | SendToWorkerWithResponse;

export type SendToWorkerWithResponse = ({
    name: 'counter-stop',
} | {
    name: 'perft-count',
} | {
    name: 'perft-think',
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
    name: 'counter-stop',
    counterResult: number,
} | {
    name: 'perft-count',
    perftResult: number,
} | {
    name: 'perft-think',
    id: number,
    done: boolean,
}) & {
    kind: 'response',
    id: number,
};

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
