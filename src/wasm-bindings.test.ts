
console.log('loading wasm tests')

import * as wasm from './wasm-bindings'

describe('wasm-bindings.test.ts', function () {
    it('d', function () {
        wasm.setPosition('startpos', [])
        expect(wasm.currentFen()).toBe('rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1')
    })
    it('e2e4', function () {
        let start = 'startpos'
        let moves = ['e2e4']
        wasm.setPosition(start, moves)
        expect(wasm.currentFen()).toBe('rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 1 1')
    })
})
