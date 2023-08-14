
import * as bindings from './wasm-bindings'

describe('wasm-bindings.test.ts', function () {

    beforeAll(async function () {
        await bindings.loadWasmBindgen()
    })

    it('d', function () {
        let uci = bindings.syncWasmUci()
        uci.setPosition('startpos', [])
        expect(uci.currentFen()).toBe('rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1')
    })
    it('e2e4', function () {
        let uci = bindings.syncWasmUci()
        let start = 'startpos'
        let moves = ['e2e4']
        uci.setPosition(start, moves)
        expect(uci.currentFen()).toBe('rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 1 1')
    })
    it('possibleMoves`', function () {
        let uci = bindings.syncWasmUci()
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

    describe('wasmWorkerForTesting()', function () {
        let uciWorker: Awaited<ReturnType<typeof bindings.searchWorker>>
        beforeEach(async function () {
            uciWorker = await bindings.searchWorker()
        })

        it('search via uci commands', async function () {
            let result1 = uciWorker.search('startpos', ['e2e4'])
            let result2 = uciWorker.search('startpos', ['e2e4'])
            let result3 = uciWorker.search('startpos', ['e2e4'])
            expect((await result1).length).toBe(4)
            expect(await result2).toEqual(await result1)
            expect(await result3).toEqual(await result1)
        })
    })
})
