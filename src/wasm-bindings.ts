
interface IBindingsJs {
    log(message: string): void;
}

declare global {
    var BindingsJs: IBindingsJs;
}

export type WasmListener = (line: string) => void;

let listeners: WasmListener[] = [];

export function listen(listener: WasmListener, callback: () => void): void {
    listeners.push(listener);
    callback();
    listeners.pop();
}

globalThis.BindingsJs = {
    log: (message: string): void => {
        console.log(message);
        message.split('\n').forEach((line) => {
            listeners.forEach((listener) => listener(line));
        });
    }
}

import * as wasm from 'crab-chess'

export let greet = wasm.greet
export let process = wasm.process

export function currentFen(): string {
    let fenLine: string = ''
    listen((line: string) => {
        if (line.indexOf('Fen: ') >= 0) {
            fenLine = line
        }
    }, () => {
        wasm.process('d')
    })
    return fenLine.split('Fen: ')[1].trim()
}

export function setPosition(position: string, moves: string[]) {
    wasm.process(`position ${position} moves ${moves.join(' ')}`)
}