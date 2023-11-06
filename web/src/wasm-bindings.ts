
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
        // console.log('wasm-bindings.ts, syncWasmUci, log_to_js:', message)
        message.split('\n').forEach((line) => {
            listeners.forEach((listener) => listener(line))
        })
    }
}

// This loads the .d.ts type definitions. However, because web worker support
// for modules isn't very mature, the wasm bindings are instead imported via a
// <script> tag which sets a global variable called `wasm_bindgen`.
import '../public/lib/wasm-pack/wasm_chess'
import { resolveable } from './helpers'
import { createWorker } from './worker/worker-wrapper'


export function jsWorkerForTesting() {
    const worker = new Worker('build/worker/js-worker-for-testing.js')

    return {
        echo: (message: string) => {
            const result = new Promise<string>((resolve) => {
                worker.onmessage = (e) => {
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

export async function newUciWasmWorker() {
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

let _workerUci: ReturnType<typeof newUciWasmWorker> | undefined = undefined
async function singletonUciWorker(): ReturnType<typeof newUciWasmWorker> {
    if (!_workerUci) {
        _workerUci = newUciWasmWorker()
        _workerUci = Promise.resolve(await _workerUci)
    }

    return await Promise.resolve(_workerUci)
}

let _searchResults: Map<string, Promise<string>> = new Map()

export async function searchWorker(): Promise<{
    search: (start: string, moves: string[]) => Promise<string>,
    terminate: () => void
}> {
    let worker = await singletonUciWorker()

    return {
        search: async (start: string, moves: string[]) => {
            let key = `${start}-${moves.join('-')}`

            let result = _searchResults.get(key)
            if (result) {
                return await result;
            }

            let [promise, resolve] = resolveable<string>()
            _searchResults.set(key, promise)

            // cancel what we're currently doing
            await worker.handle_line('stop')

            let output = []
            if (start === 'startpos') {
                output.push(await worker.handle_line(`position ${start} moves ${moves.join(' ')}`))
            } else {
                output.push(await worker.handle_line(`position fen ${start} moves ${moves.join(' ')}`))
            }
            output.push(await worker.handle_line('go'))
            await new Promise(resolve => setTimeout(resolve, 1000))
            output.push(await worker.handle_line('stop'))

            let reversed = output
                .flatMap(line => line.split('\n'))
                .map(line => line.trim())
                .filter(line => line !== '')
                .reverse()

            for (let line of reversed) {
                if (line.startsWith('bestmove')) {
                    let result = line.split(' ')[1]
                    resolve(result)
                    return result
                }
            }

            throw new Error('no bestmove found')
        },
        terminate: () => worker.terminate()
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

export function syncWasmUci() {
    let uci = wasm_bindgen.UciForJs.new()

    function handleLineAndLog(line: string) {
        let result = uci.handle_line(line)
        // console.log('wasm-bindings.ts, syncWasmUci <=', line)
        // console.log('wasm-bindings.ts, syncWasmUci =>', result)
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

            // console.log('wasm-bindings.ts, syncWasmUci, currentFen() =>', result)
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
            // console.log('wasm-bindings.ts, syncWasmUci, possibleMoves() =>', moves)
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
