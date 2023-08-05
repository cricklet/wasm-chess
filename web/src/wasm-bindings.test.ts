
import * as bindings from './wasm-bindings'

describe('wasm-bindings.test.ts', function () {
    beforeAll(async function () {
        await bindings.loadWasmBindgen()
    })

    it('d', function () {
        bindings.setPosition('startpos', [])
        expect(bindings.currentFen()).toBe('rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1')
    })
    it('e2e4', function () {
        let start = 'startpos'
        let moves = ['e2e4']
        bindings.setPosition(start, moves)
        expect(bindings.currentFen()).toBe('rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 1 1')
    })
    it('possibleMoves`', function () {
        let start = 'startpos'
        let moves = ['e2e4']
        bindings.setPosition(start, moves)

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

        const actualMoves = bindings.possibleMoves()
        actualMoves.sort()

        expect(expectedMoves).toEqual(actualMoves)
    })

    it('echoWorkerForTesting()', async function () {
        const worker = bindings.jsWorkerForTesting()

        expect(await worker.echo('hello'))
            .toBe('message received by worker: hello')

        expect(await worker.echo('bye'))
            .toBe('message received by worker: bye')

        worker.terminate()
    })

    it('wasmWorkerForTesting()', async function () {
        const worker = await bindings.wasmWorkerForTesting()

        worker.go()
        await new Promise(resolve => setTimeout(resolve, 400))
        worker.stop()

        const result = await worker.count()
        expect(result).toBeGreaterThan(2)
        worker.terminate()
    })
})
