
interface IBindingsJs {
    log_to_js(message: string): void
}

declare global {
    var BindingsJs: IBindingsJs
}

export type WasmListener = (line: string) => void

let listeners: WasmListener[] = []

export function listenScope(listener: WasmListener, callback: () => void): void {
    listeners.push(listener)
    callback()
    listeners = listeners.filter((l) => l !== listener)
}

export function listen(listener: WasmListener): () => void {
    listeners.push(listener)
    return () => {
        listeners = listeners.filter((l) => l !== listener)
    }
}

globalThis.BindingsJs = {
    log_to_js: (message: string): void => {
        console.log('> (wasm) log:', message)
        message.split('\n').forEach((line) => {
            listeners.forEach((listener) => listener(line))
        })
    }
}

// This loads the .d.ts type definitions. However, because web worker support
// for modules isn't very mature, the wasm bindings are instead imported via a
// <script> tag which sets a global variable called `wasm_bindgen`.
import '../public/lib/wasm-pack/wasm_chess'
import { createWorker } from './worker/worker-wrapper'


export function jsWorkerForTesting() {
    const worker = new Worker('build/worker/js-worker-for-testing.js')

    return {
        echo: (message: string) => {
            const result = new Promise<string>((resolve) => {
                worker.onmessage = (e) => {
                    console.log('message received by typescript:', e.data)
                    resolve(e.data)
                    worker.onmessage = null
                }
            })
            worker.postMessage(message)
            return result
        },
        terminate: () => {
            worker.terminate()
        }
    }
}


export async function wasmWorkerForTesting() {
    let worker = await createWorker('build/worker/wasm-worker-for-testing.js')

    return {
        counter: {
            go: function () {
                worker.send({ name: 'counter-go' })
            },

            stop: async function (): Promise<number> {
                let response = await worker.sendWithResponse({ name: 'counter-count' })
                return response.counterResult
            },
        },

        perft: {
            start: function (fen: string, depth: number) {
                worker.send({ name: 'perft-setup', fen, depth })
            },

            stop: async function (): Promise<number> {
                let response = await worker.sendWithResponse({ name: 'perft-count' })
                return response.perftResult
            }
        },

        terminate: () => worker.terminate
    }
}

export async function loadUciWasmWorker() {
    let worker = await createWorker('build/worker/uci-wasm-worker.js')

    return {
        handle_line: async function (line: string): Promise<string> {
            let response = await worker.sendWithResponse({
                name: 'uci',
                line
            })
            return response.output
        },
        terminate: () => worker.terminate
    }
}

let wasmLoaded = false
export async function loadWasmBindgen(): Promise<void> {
    if (wasmLoaded) {
        return
    }

    try {
        wasm_bindgen
    } catch (e) {
        if (e instanceof ReferenceError) {
            throw new Error('wasm_bindgen is undefined, please include via <script> tag')
        }
    }

    await wasm_bindgen()
    wasmLoaded = true
}

export function newUciForJs() {
    let uci = wasm_bindgen.UciForJs.new()

    function handleLineAndLog(line: string) {
        let result = uci.handle_line(line)
        console.log('> (wasm) returned:', result)
        return result
    }

    return {
        currentFen: (): string => {
            let fen = ''
            let result = handleLineAndLog('d')

            for (let line of result.split('\n')) {
                if (line.indexOf('Fen: ') >= 0) {
                    fen = line.split('Fen: ')[1].trim()
                }
            }

            console.log('> (wasm) returned currentFen:', fen)
            return fen
        },
        possibleMoves: (): string[] => {
            let moves: string[] = []
            let result = handleLineAndLog('go perft 1')

            for (let line of result.split('\n')) {
                if (line.indexOf(':') === -1) {
                    continue
                }
                let split = line.split(':').filter(v => v !== '').map(v => v.trim())
                if (split.length !== 2) {
                    continue
                }
                let [move, perft] = split
                if (perft !== '1') {
                    continue
                }

                if (move.length !== 4 && move.length !== 5) {
                    continue
                }
                moves.push(move)
            }
            console.log('> (wasm) returned moves:', moves)
            return moves
        },
        setPosition: (position: string, moves: string[]) => {
            if (position === 'startpos') {
                handleLineAndLog(`position ${position} moves ${moves.join(' ')}`)
            } else {
                handleLineAndLog(`position fen ${position} moves ${moves.join(' ')}`)
            }
        }
    }
}
