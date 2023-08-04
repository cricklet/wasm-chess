
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
        message.split('\n').forEach((line) => {
            listeners.forEach((listener) => listener(line))
        })
    }
}

export async function loadWasm(): Promise<void> {
    console.log('loading wasm')
    console.log(window.wasm_bindgen)
    await wasm_bindgen()
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