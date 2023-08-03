
interface IBindingsJs {
    log(message: string): void;
}

declare global {
    var BindingsJs: IBindingsJs;
}

globalThis.BindingsJs = {
    log: (message: string): void => {
        console.log(message);
    }
}

import * as wasm from 'crab-chess'

export function loadWasm() {
    wasm.greet()
    wasm.process('d\n');
    wasm.process('go perft 1\n');
}