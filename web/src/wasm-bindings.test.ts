
import * as wasm from './wasm-bindings'

describe('wasm-bindings.test.ts', function () {
    beforeAll(async function () {
        await wasm.loadWasmBindgen()
    })

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
    it('possibleMoves`', function () {
        let start = 'startpos'
        let moves = ['e2e4']
        wasm.setPosition(start, moves)

        const expectedMoves = [
            'a7a6',
            'b7b6',
            'c7c6',
            'd7d6',
            'e7e6',
            'f7f6',
            'g7g6',
            'h7h6',
            'a7a5',
            'b7b5',
            'c7c5',
            'd7d5',
            'e7e5',
            'f7f5',
            'g7g5',
            'h7h5',
            'b8a6',
            'b8c6',
            'g8f6',
            'g8h6',
        ]
        expectedMoves.sort()

        const actualMoves = wasm.possibleMoves()
        actualMoves.sort()

        expect(expectedMoves).toEqual(actualMoves)
    })

    it('testWorker()', async function () {
        const result = await wasm.testWorker()
        expect(result).toBe('message received by worker: hello')
    })
})
