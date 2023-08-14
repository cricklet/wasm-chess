
import * as bindings from './wasm-bindings'

describe('wasm-bindings.test.ts', function () {
    let uciWorker: { handle_line: any; terminate: any }

    beforeAll(async function () {
        await bindings.loadWasmBindgen()
        uciWorker = await bindings.uciWasmWorker()
    })

    it('d', function () {
        let uci = bindings.newUciForJs()
        uci.setPosition('startpos', [])
        expect(uci.currentFen()).toBe('rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1')
    })
    it('e2e4', function () {
        let uci = bindings.newUciForJs()
        let start = 'startpos'
        let moves = ['e2e4']
        uci.setPosition(start, moves)
        expect(uci.currentFen()).toBe('rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 1 1')
    })
    it('possibleMoves`', function () {
        let uci = bindings.newUciForJs()
        let start = 'startpos'
        let moves = ['e2e4']
        uci.setPosition(start, moves)

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

        const actualMoves = uci.possibleMoves()
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

    it('wasmWorkerForTesting() counter', async function () {
        const worker = await bindings.wasmWorkerForTesting()

        worker.counter.go()
        await new Promise(resolve => setTimeout(resolve, 400))
        const result = await worker.counter.stop()
    
        expect(result).toBeGreaterThan(2)
        worker.terminate()
    })

    it('wasmWorkerForTesting() perft very short', async function () {
        const worker = await bindings.wasmWorkerForTesting()

        worker.perft.start('startpos', 2)
        await new Promise(resolve => setTimeout(resolve, 1000))
        const result = await worker.perft.stop()
    
        expect(result).toBe(20)
        worker.terminate()
    })

    it('wasmWorkerForTesting() perft short', async function () {
        const worker = await bindings.wasmWorkerForTesting()

        worker.perft.start('startpos', 5)
        await new Promise(resolve => setTimeout(resolve, 1000))
        const result = await worker.perft.stop()
    
        expect(result).toBe(197281)
        worker.terminate()
    })

    it('wasmWorkerForTesting() perft', async function () {
        const worker = await bindings.wasmWorkerForTesting()

        worker.perft.start('startpos', 7)
        await new Promise(resolve => setTimeout(resolve, 1000))
        const result = await worker.perft.stop()
    
        expect(result).toBeGreaterThan(197281)
        worker.terminate()
    })

    it('wasmWorkerForTesting() search', async function () {
        let result = ''
        result += await uciWorker.handle_line('go')
        await new Promise(resolve => setTimeout(resolve, 3000))
        result += await uciWorker.handle_line('stop')

        console.log(result)
        expect(result).toContain('bestmove')
        uciWorker.terminate()
    })
})
