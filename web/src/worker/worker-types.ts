
export type WebToWorker = {
    kind: 'counter-go',
} | {
    kind: 'counter-stop',
} | {
    kind: 'counter-count',
} | {
    kind: 'perft-setup',
    fen: string,
    depth: number,
} | {
    kind: 'perft-count',
} | {
    kind: 'perft-think',
};

export function encodeWebToWorker(msg: WebToWorker): string {
    return JSON.stringify(msg);
}

export function decodeWebToWorker(msg: string): WebToWorker {
    return JSON.parse(msg);
}

export type WorkerToWeb = {
    kind: 'ready',
} | {
    kind: 'log',
    msg: Array<any>,
} | {
    kind: 'counter-count',
    count: number,
} | {
    kind: 'perft-count',
    count: number,
} | {
    kind: 'perft-done',
    count: number,
} | {
    kind: 'error',
    msg: string,
};

export function encodeWorkerToWeb(msg: WorkerToWeb): string {
    return JSON.stringify(msg);
}

export function decodeWorkerToWeb(msg: string): WorkerToWeb {
    return JSON.parse(msg);
}