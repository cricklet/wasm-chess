
interface IBindingsJs {
    log(message: string): void
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
    log: (message: string): void => {
        console.log(message)
        message.split('\n').forEach((line) => {
            listeners.forEach((listener) => listener(line))
        })
    }
}

// This loads the .d.ts type definitions. However, because web worker support
// for modules isn't very mature, the wasm bindings are instead imported via a
// <script> tag which sets a global variable called `wasm_bindgen`.
import '../public/lib/wasm-pack/crab_chess'

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

    wasm_bindgen.hello()
}

export function currentFen(): string {
    let fenLine: string = ''
    listenScope((line: string) => {
        if (line.indexOf('Fen: ') >= 0) {
            fenLine = line
        }
    }, () => {
        wasm_bindgen.process('d')
    })
    return fenLine.split('Fen: ')[1].trim()
}

export function possibleMoves(): string[] {
    let moves: string[] = []
    listenScope((line: string) => {
        if (line.indexOf(':') === -1) {
            return
        }
        let split = line.split(':').filter(v => v !== '').map(v => v.trim())
        if (split.length !== 2) {
            return
        }
        let [move, perft] = split
        if (perft !== '1') {
            return
        }

        if (move.length !== 4 && move.length !== 5) {
            return
        }
        moves.push(move)
    }, () => {
        wasm_bindgen.process('go perft 1')
    })

    return moves
}

export function setPosition(position: string, moves: string[]) {
    if (position === 'startpos') {
        wasm_bindgen.process(`position ${position} moves ${moves.join(' ')}`)
    } else {
        wasm_bindgen.process(`position fen ${position} moves ${moves.join(' ')}`)
    }
}